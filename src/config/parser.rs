use crate::config::primitives::{
    AnchorError, parse_align_x, parse_align_y, parse_anchor, parse_color, parse_font_stretch,
    parse_font_style, parse_font_weight, parse_interval, parse_layer, parse_output,
    parse_text_align_x,
};
use crate::config::types::PullDecl;
use crate::config::types::{FieldValue, ParsedConfig, SourceText, Span};
use crate::config::types::{VarDecl, VarValue};
use crate::config::{ConfigError, ConfigErrorKind, Severity};
use iced::Padding;
use iced::alignment::{Horizontal, Vertical};
use iced::border::Radius;
use iced_layershell::reexport::{Anchor, Layer, OutputOption};

pub(crate) fn build_var(
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
    out: &mut ParsedConfig,
) {
    let node_span = Span {
        source: source.clone(),
        span: node.span(),
    };

    let prop_entries: Vec<&kdl::KdlEntry> = node
        .entries()
        .iter()
        .filter(|e| e.name().is_some())
        .collect();

    if prop_entries.is_empty() {
        errs.push(ConfigError {
            kind: ConfigErrorKind::VariableMissingValue,
            span: node_span,
            message: "variable declaration requires a value".into(),
            severity: Severity::Error,
        });
        return;
    }

    for entry in prop_entries {
        let name = entry.name().unwrap().value().to_string();
        let value = match entry.value() {
            kdl::KdlValue::Integer(i) => VarValue::Int(*i),
            kdl::KdlValue::Float(f) => VarValue::Float(*f),
            kdl::KdlValue::Bool(b) => VarValue::Bool(*b),
            kdl::KdlValue::String(s) => VarValue::Str(s.clone()),
            kdl::KdlValue::Null => {
                errs.push(ConfigError {
                    kind: ConfigErrorKind::VariableMissingValue,
                    span: Span {
                        source: source.clone(),
                        span: entry.span(),
                    },
                    message: "variable declaration requires a value".into(),
                    severity: Severity::Error,
                });
                continue;
            }
        };
        let decl = VarDecl {
            value,
            span: Span {
                source: source.clone(),
                span: entry.span(),
            },
        };
        if out.vars.contains_key(&name) {
            errs.push(ConfigError {
                kind: ConfigErrorKind::DuplicateVariable,
                span: Span {
                    source: source.clone(),
                    span: entry.span(),
                },
                message: format!("variable {} is defined twice, using first", name),
                severity: Severity::Warning,
            });
        } else {
            out.vars.insert(name, decl);
        }
    }
}

pub(crate) fn build_pull(
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<(String, PullDecl)> {
    let node_span = Span {
        source: source.clone(),
        span: node.span(),
    };
    let err = |errs: &mut Vec<ConfigError>, msg: &str| {
        errs.push(ConfigError {
            kind: ConfigErrorKind::InvalidFieldType,
            span: node_span.clone(),
            message: msg.into(),
            severity: Severity::Error,
        });
    };

    let mut interval_str: Option<String> = None;
    let mut default = String::new();
    let mut subscription: Option<(String, String)> = None;
    let mut name_count = 0usize;

    for entry in node.entries().iter().filter(|e| e.name().is_some()) {
        let key = entry.name().unwrap().value().to_string();
        match key.as_str() {
            "i" | "interval" => match entry.value().as_string() {
                Some(s) => interval_str = Some(s.to_string()),
                None => {
                    err(errs, "pull interval must be a string");
                    return None;
                }
            },
            "default" => match entry.value().as_string() {
                Some(s) => default = s.to_string(),
                None => {
                    err(errs, "pull default must be a string");
                    return None;
                }
            },
            _ => {
                name_count += 1;
                match entry.value().as_string() {
                    Some(s) => subscription = Some((key, s.to_string())),
                    None => {
                        err(errs, "pull command must be a string");
                        return None;
                    }
                }
            }
        }
    }

    if name_count != 1 {
        err(errs, "a pull must name exactly one variable");
        return None;
    }
    let (name, command) = subscription?;
    let interval = match interval_str {
        Some(s) => match parse_interval(&s) {
            Some(d) => d,
            None => {
                err(errs, &format!("invalid interval \"{}\"", s));
                return None;
            }
        },
        None => {
            err(errs, "pull requires an interval (i= or interval=)");
            return None;
        }
    };
    Some((
        name,
        PullDecl {
            command,
            interval,
            default,
            span: node_span.clone(),
        },
    ))
}

pub(crate) fn parse_document(
    doc: kdl::KdlDocument,
    source: SourceText,
) -> (ParsedConfig, Vec<ConfigError>) {
    let mut out = ParsedConfig::default();
    let mut errs = Vec::new();
    for node in doc.nodes() {
        let name = node.name().value();
        if name != "apptray"
            && matches!(
                name,
                "widget"
                    | "container"
                    | "button"
                    | "row"
                    | "column"
                    | "text"
                    | "style"
                    | "border"
                    | "shadow"
                    | "font"
            )
            && first_positional_string(node).as_deref() == Some("apptray")
        {
            errs.push(ConfigError {
                kind: ConfigErrorKind::DuplicateElement,
                span: Span {
                    source: source.clone(),
                    span: node.span(),
                },
                message: "\"apptray\" is a reserved id".into(),
                severity: Severity::Error,
            });
            continue;
        }
        match name {
            "var" => build_var(node, &source, &mut errs, &mut out),
            "widget" => {
                if let Some((id, w)) = build_widget(node, &source, &mut errs) {
                    if out.widgets.contains_key(&id) {
                        errs.push(dup_id_warning(&id, "widget", node, &source));
                    } else {
                        out.widgets.insert(id, w);
                    }
                }
            }
            "container" => {
                if let Some((id, c)) = build_container(node, &source, &mut errs) {
                    if out.containers.contains_key(&id) {
                        errs.push(dup_id_warning(&id, "container", node, &source));
                    } else {
                        out.containers.insert(id, c);
                    }
                }
            }
            "style" => {
                if let Some((id, s)) = build_style(node, &source, &mut errs) {
                    if out.styles.contains_key(&id) {
                        errs.push(dup_id_warning(&id, "style", node, &source));
                    } else {
                        out.styles.insert(id, s);
                    }
                }
            }
            "border" => {
                if let Some((id, b)) = build_border(node, &source, &mut errs) {
                    if out.borders.contains_key(&id) {
                        errs.push(dup_id_warning(&id, "border", node, &source));
                    } else {
                        out.borders.insert(id, b);
                    }
                }
            }
            "shadow" => {
                if let Some((id, s)) = build_shadow(node, &source, &mut errs) {
                    if out.shadows.contains_key(&id) {
                        errs.push(dup_id_warning(&id, "shadow", node, &source));
                    } else {
                        out.shadows.insert(id, s);
                    }
                }
            }
            "font" => {
                if let Some((id, f)) = build_font(node, &source, &mut errs) {
                    if out.fonts.contains_key(&id) {
                        errs.push(dup_id_warning(&id, "font", node, &source));
                    } else {
                        out.fonts.insert(id, f);
                    }
                }
            }
            "button" => {
                if let Some((id, b)) = build_button(node, &source, &mut errs) {
                    if out.buttons.contains_key(&id) {
                        errs.push(dup_id_warning(&id, "button", node, &source));
                    } else {
                        out.buttons.insert(id, b);
                    }
                }
            }
            "row" => {
                if let Some((id, r)) = build_row(node, &source, &mut errs) {
                    if out.rows.contains_key(&id) {
                        errs.push(dup_id_warning(&id, "row", node, &source));
                    } else {
                        out.rows.insert(id, r);
                    }
                }
            }
            "column" => {
                if let Some((id, c)) = build_column(node, &source, &mut errs) {
                    if out.columns.contains_key(&id) {
                        errs.push(dup_id_warning(&id, "column", node, &source));
                    } else {
                        out.columns.insert(id, c);
                    }
                }
            }
            "text" => {
                if let Some((id, t)) = build_text(node, &source, &mut errs) {
                    if out.texts.contains_key(&id) {
                        errs.push(dup_id_warning(&id, "text", node, &source));
                    } else {
                        out.texts.insert(id, t);
                    }
                }
            }
            "notification" => {
                let ns = build_notification(node, &source, &mut errs);
                if out.notification.is_some() {
                    errs.push(ConfigError {
                        kind: ConfigErrorKind::DuplicateElement,
                        span: Span {
                            source: source.clone(),
                            span: node.span(),
                        },
                        message: "notification block is defined twice, using first".into(),
                        severity: Severity::Warning,
                    });
                } else {
                    out.notification = Some(ns);
                }
            }
            "apptray" => {
                let a = build_apptray_settings(node, &source, &mut errs);
                if out.apptray.is_some() {
                    errs.push(ConfigError {
                        kind: ConfigErrorKind::DuplicateElement,
                        span: Span {
                            source: source.clone(),
                            span: node.span(),
                        },
                        message: "apptray block is defined twice, using first".into(),
                        severity: Severity::Warning,
                    });
                } else {
                    out.apptray = Some(a);
                }
            }
            "icon_theme" => match first_positional_string(node) {
                Some(_) if out.icon_theme.is_some() => errs.push(ConfigError {
                    kind: ConfigErrorKind::DuplicateElement,
                    span: Span {
                        source: source.clone(),
                        span: node.span(),
                    },
                    message: "icon_theme is defined twice, using first".into(),
                    severity: Severity::Warning,
                }),
                Some(theme) => out.icon_theme = Some(theme),
                None => errs.push(ConfigError {
                    kind: ConfigErrorKind::MissingRequiredField,
                    span: Span {
                        source: source.clone(),
                        span: node.span(),
                    },
                    message:
                        "icon_theme requires a string value, e.g. icon_theme \"Gruvbox-Plus-Dark\""
                            .into(),
                    severity: Severity::Warning,
                }),
            },
            "pull" => {
                if let Some((id, decl)) = build_pull(node, &source, &mut errs) {
                    if out.vars.contains_key(&id) || out.pulls.contains_key(&id) {
                        errs.push(ConfigError {
                            kind: ConfigErrorKind::DuplicateVariable,
                            span: Span {
                                source: source.clone(),
                                span: node.span(),
                            },
                            message: format!("variable {} is defined twice, using first", id),
                            severity: Severity::Warning,
                        });
                    } else {
                        out.vars.insert(
                            id.clone(),
                            VarDecl {
                                value: VarValue::Str(decl.default.clone()),
                                span: decl.span.clone(),
                            },
                        );
                        out.pulls.insert(id, decl);
                    }
                }
            }
            _ => errs.push(ConfigError {
                kind: ConfigErrorKind::UnknownNode,
                span: Span {
                    source: source.clone(),
                    span: node.span(),
                },
                message: format!("unknown node \"{}\"", name),
                severity: Severity::Error,
            }),
        }
    }
    (out, errs)
}

fn looks_like_expr(s: &str) -> bool {
    s.contains("${")
}

fn span_of_entry(entry: &kdl::KdlEntry, source: &SourceText) -> Span {
    Span {
        source: source.clone(),
        span: entry.span(),
    }
}

fn prop<'a>(node: &'a kdl::KdlNode, name: &str) -> Option<&'a kdl::KdlEntry> {
    node.entries()
        .iter()
        .find(|e| e.name().map(|n| n.value() == name).unwrap_or(false))
}

pub(crate) fn field_bool(
    name: &str,
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<FieldValue<bool>> {
    let entry = prop(node, name)?;
    match entry.value() {
        kdl::KdlValue::Bool(b) => Some(FieldValue::Literal(*b)),
        kdl::KdlValue::String(s) if looks_like_expr(s) => Some(FieldValue::Expr(s.clone())),
        _ => {
            errs.push(ConfigError {
                kind: ConfigErrorKind::InvalidBool,
                span: span_of_entry(entry, source),
                message: "invalid bool, expected #true or #false".into(),
                severity: Severity::Error,
            });
            None
        }
    }
}

pub(crate) fn field_f32(
    name: &str,
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<FieldValue<f32>> {
    let entry = prop(node, name)?;
    match entry.value() {
        kdl::KdlValue::Integer(i) => Some(FieldValue::Literal(*i as f32)),
        kdl::KdlValue::Float(f) => Some(FieldValue::Literal(*f as f32)),
        kdl::KdlValue::String(s) if looks_like_expr(s) => Some(FieldValue::Expr(s.clone())),
        _ => {
            errs.push(ConfigError {
                kind: ConfigErrorKind::InvalidLengthValue,
                span: span_of_entry(entry, source),
                message: format!("field `{}` expects a number", name),
                severity: Severity::Error,
            });
            None
        }
    }
}

pub(crate) fn field_color(
    name: &str,
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<FieldValue<iced::Color>> {
    let entry = prop(node, name)?;
    let text = match entry.value() {
        kdl::KdlValue::String(s) => s.clone(),
        kdl::KdlValue::Integer(i) => format!("{:06}", i),
        _ => {
            errs.push(invalid_color(span_of_entry(entry, source)));
            return None;
        }
    };
    if looks_like_expr(&text) {
        return Some(FieldValue::Expr(text));
    }
    match parse_color(&text) {
        Some(c) => Some(FieldValue::Literal(c)),
        None => {
            errs.push(invalid_color(span_of_entry(entry, source)));
            None
        }
    }
}

fn invalid_color(span: Span) -> ConfigError {
    ConfigError {
        kind: ConfigErrorKind::InvalidColor, span,
        message: "invalid color format, expected rrggbb, rrggbbaa, #rrggbb, #rrggbbaa, transparent, or int".into(),
        severity: Severity::Error,
    }
}

pub(crate) fn field_id_ref(
    name: &str,
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<FieldValue<String>> {
    let entry = prop(node, name)?;
    match entry.value() {
        kdl::KdlValue::String(s) => Some(if looks_like_expr(s) {
            FieldValue::Expr(s.clone())
        } else {
            FieldValue::Literal(s.clone())
        }),
        kdl::KdlValue::Integer(_)
        | kdl::KdlValue::Float(_)
        | kdl::KdlValue::Bool(_)
        | kdl::KdlValue::Null => {
            errs.push(ConfigError {
                kind: ConfigErrorKind::InvalidFieldType,
                span: span_of_entry(entry, source),
                message: format!("field `{}` expects an id (string)", name),
                severity: Severity::Error,
            });
            None
        }
    }
}

pub(crate) fn field_string(
    name: &str,
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<FieldValue<String>> {
    let entry = prop(node, name)?;
    match entry.value() {
        kdl::KdlValue::String(s) => Some(if looks_like_expr(s) {
            FieldValue::Expr(s.clone())
        } else {
            FieldValue::Literal(s.clone())
        }),
        kdl::KdlValue::Integer(_)
        | kdl::KdlValue::Float(_)
        | kdl::KdlValue::Bool(_)
        | kdl::KdlValue::Null => {
            errs.push(ConfigError {
                kind: ConfigErrorKind::InvalidFieldType,
                span: span_of_entry(entry, source),
                message: format!("field `{}` expects a string", name),
                severity: Severity::Error,
            });
            None
        }
    }
}

pub(crate) fn field_layer(
    name: &str,
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<FieldValue<Layer>> {
    let entry = prop(node, name)?;
    match entry.value() {
        kdl::KdlValue::String(s) if looks_like_expr(s) => Some(FieldValue::Expr(s.clone())),
        kdl::KdlValue::String(s) => match parse_layer(s) {
            Some(l) => Some(FieldValue::Literal(l)),
            None => {
                errs.push(ConfigError {
                    kind: ConfigErrorKind::InvalidEnumValue,
                    span: span_of_entry(entry, source),
                    message: format!("invalid layer value \"{}\", expected one of: top, bottom, background, overlay", s),
                    severity: Severity::Error,
                });
                None
            }
        },
        _ => {
            errs.push(ConfigError {
                kind: ConfigErrorKind::InvalidEnumValue,
                span: span_of_entry(entry, source),
                message: "invalid layer value, expected one of: top, bottom, background, overlay"
                    .into(),
                severity: Severity::Error,
            });
            None
        }
    }
}

pub(crate) fn field_anchor(
    name: &str,
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<FieldValue<Anchor>> {
    let entry = prop(node, name)?;
    let text = match entry.value() {
        kdl::KdlValue::String(s) => s.clone(),
        _ => {
            errs.push(ConfigError {
                kind: ConfigErrorKind::InvalidEnumValue,
                span: span_of_entry(entry, source),
                message: "anchor must be a string".into(),
                severity: Severity::Error,
            });
            return None;
        }
    };
    if looks_like_expr(&text) {
        return Some(FieldValue::Expr(text));
    }
    match parse_anchor(&text) {
        Ok(a) => Some(FieldValue::Literal(a)),
        Err(AnchorError::Unknown(tok)) => push_anchor_err(
            errs,
            entry,
            source,
            ConfigErrorKind::InvalidEnumValue,
            &format!("invalid anchor token \"{}\"", tok),
        ),
    }
}

fn push_anchor_err(
    errs: &mut Vec<ConfigError>,
    entry: &kdl::KdlEntry,
    source: &SourceText,
    kind: ConfigErrorKind,
    msg: &str,
) -> Option<FieldValue<Anchor>> {
    errs.push(ConfigError {
        kind,
        span: span_of_entry(entry, source),
        message: msg.to_string(),
        severity: Severity::Error,
    });
    None
}

pub(crate) fn field_output(
    name: &str,
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<FieldValue<OutputOption>> {
    let entry = prop(node, name)?;
    match entry.value() {
        kdl::KdlValue::String(s) if looks_like_expr(s) => Some(FieldValue::Expr(s.clone())),
        kdl::KdlValue::String(s) => Some(FieldValue::Literal(parse_output(s))),
        _ => {
            errs.push(ConfigError {
                kind: ConfigErrorKind::InvalidEnumValue,
                span: span_of_entry(entry, source),
                message: "invalid output value, expected a string or `last`".into(),
                severity: Severity::Error,
            });
            None
        }
    }
}

fn collect_f32_vals(child: &kdl::KdlNode) -> Vec<f32> {
    child
        .entries()
        .iter()
        .filter(|e| e.name().is_none())
        .filter_map(|e| match e.value() {
            kdl::KdlValue::Integer(i) => Some(*i as f32),
            kdl::KdlValue::Float(f) => Some(*f as f32),
            _ => None,
        })
        .collect()
}

pub(crate) fn field_margin(
    name: &str,
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<FieldValue<(f32, f32, f32, f32)>> {
    if let Some(entry) = prop(node, name) {
        let v = match entry.value() {
            kdl::KdlValue::Integer(i) => *i as f32,
            kdl::KdlValue::Float(f) => *f as f32,
            kdl::KdlValue::String(s) if looks_like_expr(s) => {
                return Some(FieldValue::Expr(s.clone()));
            }
            _ => {
                errs.push(ConfigError {
                    kind: ConfigErrorKind::InvalidMarginArity,
                    span: span_of_entry(entry, source),
                    message: "margin accepts 1, 2, or 4 values".into(),
                    severity: Severity::Error,
                });
                return None;
            }
        };
        return Some(FieldValue::Literal((v, v, v, v)));
    }
    if let Some(children) = node.children() {
        for child in children.nodes() {
            if child.name().value() != name {
                continue;
            }
            let vals = collect_f32_vals(child);
            return match vals.as_slice() {
                [a] => Some(FieldValue::Literal((*a, *a, *a, *a))),
                [v, h] => Some(FieldValue::Literal((*v, *h, *v, *h))),
                [t, r, b, l] => Some(FieldValue::Literal((*t, *r, *b, *l))),
                _ => {
                    errs.push(ConfigError {
                        kind: ConfigErrorKind::InvalidMarginArity,
                        span: Span {
                            source: source.clone(),
                            span: child.span(),
                        },
                        message: "margin accepts 1, 2, or 4 values".into(),
                        severity: Severity::Error,
                    });
                    None
                }
            };
        }
    }
    None
}

pub(crate) fn field_padding(
    name: &str,
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<FieldValue<Padding>> {
    if let Some(entry) = prop(node, name) {
        let v = match entry.value() {
            kdl::KdlValue::Integer(i) => *i as f32,
            kdl::KdlValue::Float(f) => *f as f32,
            kdl::KdlValue::String(s) if looks_like_expr(s) => {
                return Some(FieldValue::Expr(s.clone()));
            }
            _ => {
                errs.push(ConfigError {
                    kind: ConfigErrorKind::InvalidPaddingArity,
                    span: span_of_entry(entry, source),
                    message: "padding accepts 1, 2, or 4 values".into(),
                    severity: Severity::Error,
                });
                return None;
            }
        };
        return Some(FieldValue::Literal(Padding::from(v)));
    }
    if let Some(children) = node.children() {
        for child in children.nodes() {
            if child.name().value() != name {
                continue;
            }
            let vals = collect_f32_vals(child);
            return match vals.as_slice() {
                [a] => Some(FieldValue::Literal(Padding::from(*a))),
                [v, h] => Some(FieldValue::Literal(Padding::from([*v, *h]))),
                [t, r, b, l] => Some(FieldValue::Literal(Padding {
                    top: *t,
                    right: *r,
                    bottom: *b,
                    left: *l,
                })),
                _ => {
                    errs.push(ConfigError {
                        kind: ConfigErrorKind::InvalidPaddingArity,
                        span: Span {
                            source: source.clone(),
                            span: child.span(),
                        },
                        message: "padding accepts 1, 2, or 4 values".into(),
                        severity: Severity::Error,
                    });
                    None
                }
            };
        }
    }
    None
}

pub(crate) fn field_radius(
    name: &str,
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<FieldValue<Radius>> {
    if let Some(entry) = prop(node, name) {
        let v = match entry.value() {
            kdl::KdlValue::Integer(i) => *i as f32,
            kdl::KdlValue::Float(f) => *f as f32,
            kdl::KdlValue::String(s) if looks_like_expr(s) => {
                return Some(FieldValue::Expr(s.clone()));
            }
            _ => {
                errs.push(ConfigError {
                    kind: ConfigErrorKind::InvalidRadiusArity,
                    span: span_of_entry(entry, source),
                    message: "radius accepts 1 or 4 values".into(),
                    severity: Severity::Error,
                });
                return None;
            }
        };
        return Some(FieldValue::Literal(Radius::from(v)));
    }
    if let Some(children) = node.children() {
        for child in children.nodes() {
            if child.name().value() != name {
                continue;
            }
            let vals = collect_f32_vals(child);
            return match vals.as_slice() {
                [a] => Some(FieldValue::Literal(Radius::from(*a))),
                [tl, tr, br, bl] => Some(FieldValue::Literal(Radius {
                    top_left: *tl,
                    top_right: *tr,
                    bottom_right: *br,
                    bottom_left: *bl,
                })),
                _ => {
                    errs.push(ConfigError {
                        kind: ConfigErrorKind::InvalidRadiusArity,
                        span: Span {
                            source: source.clone(),
                            span: child.span(),
                        },
                        message: "radius accepts 1 or 4 values".into(),
                        severity: Severity::Error,
                    });
                    None
                }
            };
        }
    }
    None
}

pub(crate) fn field_length(
    name: &str,
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<FieldValue<iced::Length>> {
    if let Some(entry) = prop(node, name) {
        return match entry.value() {
            kdl::KdlValue::String(s) if looks_like_expr(s) => Some(FieldValue::Expr(s.clone())),
            kdl::KdlValue::String(s) => match s.as_str() {
                "fill" => Some(FieldValue::Literal(iced::Length::Fill)),
                "shrink" => Some(FieldValue::Literal(iced::Length::Shrink)),
                "portion" => {
                    errs.push(ConfigError {
                        kind: ConfigErrorKind::PortionMissingInt,
                        span: span_of_entry(entry, source),
                        message: "portion requires an integer argument".into(),
                        severity: Severity::Error,
                    });
                    None
                }
                _ => {
                    errs.push(ConfigError {
                        kind: ConfigErrorKind::InvalidLengthValue,
                        span: span_of_entry(entry, source),
                        message: format!("invalid length value \"{}\"", s),
                        severity: Severity::Error,
                    });
                    None
                }
            },
            kdl::KdlValue::Integer(i) => Some(FieldValue::Literal(iced::Length::Fixed(*i as f32))),
            kdl::KdlValue::Float(f) => Some(FieldValue::Literal(iced::Length::Fixed(*f as f32))),
            _ => None,
        };
    }
    if let Some(children) = node.children() {
        for child in children.nodes() {
            if child.name().value() != name {
                continue;
            }
            if let Some(p) = prop(child, "portion") {
                if let kdl::KdlValue::Integer(i) = p.value() {
                    return Some(FieldValue::Literal(iced::Length::FillPortion(*i as u16)));
                } else {
                    errs.push(ConfigError {
                        kind: ConfigErrorKind::PortionMissingInt,
                        span: Span {
                            source: source.clone(),
                            span: p.span(),
                        },
                        message: "portion requires an integer argument".into(),
                        severity: Severity::Error,
                    });
                    return None;
                }
            }
        }
    }
    None
}

pub(crate) fn field_align_x(
    name: &str,
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<FieldValue<Horizontal>> {
    let entry = prop(node, name)?;
    match entry.value() {
        kdl::KdlValue::String(s) if looks_like_expr(s) => Some(FieldValue::Expr(s.clone())),
        kdl::KdlValue::String(s) => match parse_align_x(s) {
            Some(a) => Some(FieldValue::Literal(a)),
            None => {
                invalid_align(
                    errs,
                    entry,
                    source,
                    "align_x",
                    "invalid align_x value, expected one of: l, c, r, left, center, right",
                );
                None
            }
        },
        _ => {
            invalid_align(
                errs,
                entry,
                source,
                "align_x",
                "invalid align_x value, expected one of: l, c, r, left, center, right",
            );
            None
        }
    }
}

pub(crate) fn field_text_align_x(
    name: &str,
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<FieldValue<iced::advanced::text::Alignment>> {
    let entry = prop(node, name)?;
    let msg = "invalid align_x value, expected one of: l, c, r, j, left, center, right, justified";
    match entry.value() {
        kdl::KdlValue::String(s) if looks_like_expr(s) => Some(FieldValue::Expr(s.clone())),
        kdl::KdlValue::String(s) => match parse_text_align_x(s) {
            Some(a) => Some(FieldValue::Literal(a)),
            None => {
                invalid_align(errs, entry, source, "align_x", msg);
                None
            }
        },
        _ => {
            invalid_align(errs, entry, source, "align_x", msg);
            None
        }
    }
}

pub(crate) fn field_align_y(
    name: &str,
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<FieldValue<Vertical>> {
    let entry = prop(node, name)?;
    match entry.value() {
        kdl::KdlValue::String(s) if looks_like_expr(s) => Some(FieldValue::Expr(s.clone())),
        kdl::KdlValue::String(s) => match parse_align_y(s) {
            Some(a) => Some(FieldValue::Literal(a)),
            None => {
                invalid_align(
                    errs,
                    entry,
                    source,
                    "align_y",
                    "invalid align_y value, expected one of: t, c, b, top, center, bottom",
                );
                None
            }
        },
        _ => {
            invalid_align(
                errs,
                entry,
                source,
                "align_y",
                "invalid align_y value, expected one of: t, c, b, top, center, bottom",
            );
            None
        }
    }
}

fn invalid_align(
    errs: &mut Vec<ConfigError>,
    entry: &kdl::KdlEntry,
    source: &SourceText,
    _name: &str,
    msg: &str,
) {
    errs.push(ConfigError {
        kind: ConfigErrorKind::InvalidEnumValue,
        span: span_of_entry(entry, source),
        message: msg.into(),
        severity: Severity::Error,
    });
}

use crate::config::types::Widget;

pub(crate) fn build_widget(
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<(String, Widget)> {
    let id = first_positional_string(node)?;
    let w = Widget {
        h: field_f32("h", node, source, errs),
        w: field_f32("w", node, source, errs),
        layer: field_layer("layer", node, source, errs),
        anchor: field_anchor("anchor", node, source, errs),
        exclusive: field_bool("exclusive", node, source, errs),
        margin: field_margin("margin", node, source, errs),
        output: field_output("output", node, source, errs),
        keyboard: field_bool("keyboard", node, source, errs),
        transparent: field_bool("transparent", node, source, errs),
        child: field_id_ref("child", node, source, errs),
        span: Span {
            source: source.clone(),
            span: node.span(),
        },
    };
    Some((id, w))
}

use crate::config::types::Container;

pub(crate) fn build_container(
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<(String, Container)> {
    let id = first_positional_string(node)?;
    let child = field_id_ref("child", node, source, errs);
    if child.is_none() {
        errs.push(ConfigError {
            kind: ConfigErrorKind::MissingRequiredField,
            span: Span {
                source: source.clone(),
                span: node.span(),
            },
            message: "child is required".into(),
            severity: Severity::Error,
        });
    }
    let c = Container {
        w: field_length("w", node, source, errs),
        h: field_length("h", node, source, errs),
        padding: field_padding("padding", node, source, errs),
        align_x: field_align_x("align_x", node, source, errs),
        align_y: field_align_y("align_y", node, source, errs),
        clip: field_bool("clip", node, source, errs),
        style: field_id_ref("style", node, source, errs),
        child,
        span: Span {
            source: source.clone(),
            span: node.span(),
        },
    };
    Some((id, c))
}

use crate::config::types::Style;

pub(crate) fn build_style(
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<(String, Style)> {
    let id = first_positional_string(node)?;
    let s = Style {
        text: field_color("text", node, source, errs),
        bg: field_color("bg", node, source, errs),
        border: field_id_ref("border", node, source, errs),
        shadow: field_id_ref("shadow", node, source, errs),
        snap: field_bool("snap", node, source, errs),
        span: Span {
            source: source.clone(),
            span: node.span(),
        },
    };
    Some((id, s))
}

use crate::config::types::Border;

fn font_enum<T>(
    node: &kdl::KdlNode,
    name: &str,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
    parse: impl Fn(&str) -> Option<T>,
    expected: &str,
) -> Option<T> {
    let entry = prop(node, name)?;
    match entry.value() {
        kdl::KdlValue::String(s) => match parse(s) {
            Some(v) => Some(v),
            None => {
                errs.push(ConfigError {
                    kind: ConfigErrorKind::InvalidEnumValue,
                    span: span_of_entry(entry, source),
                    message: expected.into(),
                    severity: Severity::Error,
                });
                None
            }
        },
        _ => {
            errs.push(ConfigError {
                kind: ConfigErrorKind::InvalidEnumValue,
                span: span_of_entry(entry, source),
                message: expected.into(),
                severity: Severity::Error,
            });
            None
        }
    }
}

pub(crate) fn build_font(
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<(String, iced::Font)> {
    let id = first_positional_string(node)?;
    let mut font = iced::Font::DEFAULT;
    if let Some(entry) = prop(node, "family") {
        match entry.value() {
            kdl::KdlValue::String(s) => {
                font.family = iced::font::Family::Name(Box::leak(s.to_string().into_boxed_str()))
            }
            _ => errs.push(ConfigError {
                kind: ConfigErrorKind::InvalidEnumValue,
                span: span_of_entry(entry, source),
                message: "font family must be a string".into(),
                severity: Severity::Error,
            }),
        }
    }
    if let Some(v) = font_enum(
        node,
        "weight",
        source,
        errs,
        parse_font_weight,
        "invalid weight, expected one of: thin, extra-light, light, normal, medium, semibold, bold, extra-bold, black",
    ) {
        font.weight = v;
    }
    if let Some(v) = font_enum(
        node,
        "stretch",
        source,
        errs,
        parse_font_stretch,
        "invalid stretch, expected one of: ultra-condensed, extra-condensed, condensed, semi-condensed, normal, semi-expanded, expanded, extra-expanded, ultra-expanded",
    ) {
        font.stretch = v;
    }
    if let Some(v) = font_enum(
        node,
        "style",
        source,
        errs,
        parse_font_style,
        "invalid style, expected one of: normal, italic, oblique",
    ) {
        font.style = v;
    }
    Some((id, font))
}

pub(crate) fn build_border(
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<(String, Border)> {
    let id = first_positional_string(node)?;
    let b = Border {
        color: field_color("color", node, source, errs),
        w: field_f32("w", node, source, errs),
        radius: field_radius("radius", node, source, errs),
        span: Span {
            source: source.clone(),
            span: node.span(),
        },
    };
    Some((id, b))
}

fn first_positional_string(node: &kdl::KdlNode) -> Option<String> {
    node.entries()
        .iter()
        .find(|e| e.name().is_none())
        .and_then(|e| e.value().as_string().map(|s| s.to_string()))
}

fn dup_id_warning(id: &str, kind: &str, node: &kdl::KdlNode, source: &SourceText) -> ConfigError {
    ConfigError {
        kind: ConfigErrorKind::DuplicateElement,
        span: Span {
            source: source.clone(),
            span: node.span(),
        },
        message: format!("{} {} is defined twice, using first", kind, id),
        severity: Severity::Warning,
    }
}

pub(crate) fn field_offset(
    name: &str,
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<FieldValue<(f32, f32)>> {
    let children = node.children()?;
    for child in children.nodes() {
        if child.name().value() != name {
            continue;
        }
        let vals: Vec<f32> = child
            .entries()
            .iter()
            .filter(|e| e.name().is_none())
            .filter_map(|e| match e.value() {
                kdl::KdlValue::Integer(i) => Some(*i as f32),
                kdl::KdlValue::Float(f) => Some(*f as f32),
                _ => None,
            })
            .collect();
        return match vals.as_slice() {
            [a, b] => Some(FieldValue::Literal((*a, *b))),
            _ => {
                errs.push(ConfigError {
                    kind: ConfigErrorKind::InvalidOffsetArity,
                    span: Span {
                        source: source.clone(),
                        span: child.span(),
                    },
                    message: "offset requires exactly 2 values".into(),
                    severity: Severity::Error,
                });
                None
            }
        };
    }
    None
}

pub(crate) fn field_children(
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<FieldValue<Vec<String>>> {
    let children = node.children()?;
    for child in children.nodes() {
        if child.name().value() != "children" {
            continue;
        }
        let ids: Vec<String> = child
            .entries()
            .iter()
            .filter(|e| e.name().is_none())
            .filter_map(|e| e.value().as_string().map(|s| s.to_string()))
            .collect();
        if ids.is_empty() {
            errs.push(ConfigError {
                kind: ConfigErrorKind::EmptyChildrenList,
                span: Span {
                    source: source.clone(),
                    span: child.span(),
                },
                message: "children requires at least one id".into(),
                severity: Severity::Error,
            });
            return None;
        }
        return Some(FieldValue::Literal(ids));
    }
    None
}

pub(crate) fn field_col_align(
    name: &str,
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<FieldValue<Horizontal>> {
    let entry = prop(node, name)?;
    match entry.value() {
        kdl::KdlValue::String(s) if looks_like_expr(s) => Some(FieldValue::Expr(s.clone())),
        kdl::KdlValue::String(s) => match parse_align_x(s) {
            Some(a) => Some(FieldValue::Literal(a)),
            None => {
                errs.push(ConfigError {
                    kind: ConfigErrorKind::InvalidEnumValue,
                    span: span_of_entry(entry, source),
                    message: "invalid align value for column, expected one of: l, c, r, left, center, right".into(),
                    severity: Severity::Error,
                });
                None
            }
        },
        _ => None,
    }
}

pub(crate) fn field_row_align(
    name: &str,
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<FieldValue<Vertical>> {
    let entry = prop(node, name)?;
    match entry.value() {
        kdl::KdlValue::String(s) if looks_like_expr(s) => Some(FieldValue::Expr(s.clone())),
        kdl::KdlValue::String(s) => {
            match parse_align_y(s) {
                Some(a) => Some(FieldValue::Literal(a)),
                None => {
                    errs.push(ConfigError {
                    kind: ConfigErrorKind::InvalidEnumValue,
                    span: span_of_entry(entry, source),
                    message: "invalid align value for row, expected one of: t, c, b, top, center, bottom".into(),
                    severity: Severity::Error,
                });
                    None
                }
            }
        }
        _ => None,
    }
}

use crate::config::types::Button;

pub(crate) fn build_button(
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<(String, Button)> {
    let id = first_positional_string(node)?;
    let b = Button {
        w: field_length("w", node, source, errs),
        h: field_length("h", node, source, errs),
        padding: field_padding("padding", node, source, errs),
        action: field_string("action", node, source, errs),
        clip: field_bool("clip", node, source, errs),
        style: field_id_ref("style", node, source, errs),
        style_hover: field_id_ref("style:hover", node, source, errs),
        style_active: field_id_ref("style:active", node, source, errs),
        style_disabled: field_id_ref("style:disabled", node, source, errs),
        text: field_string("text", node, source, errs),
        font: field_string("font", node, source, errs),
        span: Span {
            source: source.clone(),
            span: node.span(),
        },
    };
    Some((id, b))
}

use crate::config::types::Row;

pub(crate) fn build_row(
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<(String, Row)> {
    let id = first_positional_string(node)?;
    let children = field_children(node, source, errs);
    if children.is_none() {
        let has_children_node = node
            .children()
            .map(|d| d.nodes().iter().any(|n| n.name().value() == "children"))
            .unwrap_or(false);
        if !has_children_node {
            errs.push(ConfigError {
                kind: ConfigErrorKind::MissingRequiredField,
                span: Span {
                    source: source.clone(),
                    span: node.span(),
                },
                message: "children is required".into(),
                severity: Severity::Error,
            });
        }
    }
    let r = Row {
        children,
        w: field_length("w", node, source, errs),
        h: field_length("h", node, source, errs),
        padding: field_padding("padding", node, source, errs),
        spacing: field_f32("spacing", node, source, errs),
        clip: field_bool("clip", node, source, errs),
        align: field_row_align("align", node, source, errs),
        span: Span {
            source: source.clone(),
            span: node.span(),
        },
    };
    Some((id, r))
}

use crate::config::types::Column;

pub(crate) fn build_column(
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<(String, Column)> {
    let id = first_positional_string(node)?;
    let children = field_children(node, source, errs);
    if children.is_none() {
        let has_children_node = node
            .children()
            .map(|d| d.nodes().iter().any(|n| n.name().value() == "children"))
            .unwrap_or(false);
        if !has_children_node {
            errs.push(ConfigError {
                kind: ConfigErrorKind::MissingRequiredField,
                span: Span {
                    source: source.clone(),
                    span: node.span(),
                },
                message: "children is required".into(),
                severity: Severity::Error,
            });
        }
    }
    let c = Column {
        children,
        w: field_length("w", node, source, errs),
        h: field_length("h", node, source, errs),
        padding: field_padding("padding", node, source, errs),
        spacing: field_f32("spacing", node, source, errs),
        clip: field_bool("clip", node, source, errs),
        align: field_col_align("align", node, source, errs),
        span: Span {
            source: source.clone(),
            span: node.span(),
        },
    };
    Some((id, c))
}

use crate::config::types::TextEl;

pub(crate) fn build_text(
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<(String, TextEl)> {
    let id = first_positional_string(node)?;
    let t = TextEl {
        w: field_length("w", node, source, errs),
        h: field_length("h", node, source, errs),
        align_x: field_text_align_x("align_x", node, source, errs),
        align_y: field_align_y("align_y", node, source, errs),
        color: field_color("color", node, source, errs),
        font: field_string("font", node, source, errs),
        content: field_string("text", node, source, errs),
        span: Span {
            source: source.clone(),
            span: node.span(),
        },
    };
    Some((id, t))
}

use crate::config::types::Shadow;

pub(crate) fn build_shadow(
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> Option<(String, Shadow)> {
    let id = first_positional_string(node)?;
    let s = Shadow {
        color: field_color("color", node, source, errs),
        offset: field_offset("offset", node, source, errs),
        blur_radius: field_f32("blur_radius", node, source, errs),
        span: Span {
            source: source.clone(),
            span: node.span(),
        },
    };
    Some((id, s))
}

use crate::config::types::ApptraySettings;

pub(crate) fn build_apptray_settings(
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> ApptraySettings {
    ApptraySettings {
        icon_size: field_f32("icon_size", node, source, errs),
        spacing: field_f32("spacing", node, source, errs),
        padding: field_padding("padding", node, source, errs),
        bg: field_color("bg", node, source, errs),
        border: field_id_ref("border", node, source, errs),
        swap_buttons: field_bool("swap_buttons", node, source, errs),
        vertical: field_bool("vertical", node, source, errs),
        menu_bg: field_color("menu_bg", node, source, errs),
        menu_text: field_color("menu_text", node, source, errs),
        menu_disabled: field_color("menu_disabled", node, source, errs),
        span: Span {
            source: source.clone(),
            span: node.span(),
        },
    }
}

use crate::config::types::NotificationSettings;

pub(crate) fn build_notification(
    node: &kdl::KdlNode,
    source: &SourceText,
    errs: &mut Vec<ConfigError>,
) -> NotificationSettings {
    NotificationSettings {
        width: field_f32("width", node, source, errs),
        height: field_f32("height", node, source, errs),
        primary_text: field_color("primary_text", node, source, errs),
        secondary_text: field_color("secondary_text", node, source, errs),
        bg: field_color("bg", node, source, errs),
        border: field_id_ref("border", node, source, errs),
        font: field_string("font", node, source, errs),
        anchor: field_anchor("anchor", node, source, errs),
        margin: field_margin("margin", node, source, errs),
        gap: field_f32("gap", node, source, errs),
        max: field_f32("max", node, source, errs),
        timeout: field_f32("timeout", node, source, errs),
        layer: field_layer("layer", node, source, errs),
        respect_notification_icon: field_bool("respect_notification_icon", node, source, errs),
        freeze_on_hover: field_bool("freeze_on_hover", node, source, errs),
        span: Span {
            source: source.clone(),
            span: node.span(),
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{Severity, parse_str};

    pub(crate) enum Expect {
        Ok,
        Err(&'static str),
        Warn(&'static str),
    }

    pub(crate) struct Case {
        pub label: &'static str,
        pub kdl: &'static str,
        pub expect: Expect,
    }

    pub(crate) fn run_cases(cases: &[Case]) {
        for c in cases {
            let (cfg, msgs) = parse_str(c.kdl, "<test>");
            match &c.expect {
                Expect::Ok => {
                    assert!(
                        cfg.is_some(),
                        "case `{}`: expected Ok, got errors: {:#?}",
                        c.label,
                        msgs
                    );
                    let errs: Vec<_> = msgs
                        .iter()
                        .filter(|m| m.severity == Severity::Error)
                        .collect();
                    assert!(
                        errs.is_empty(),
                        "case `{}`: expected no errors, got: {:#?}",
                        c.label,
                        errs
                    );
                }
                Expect::Err(expected_msg) => {
                    let errs: Vec<_> = msgs
                        .iter()
                        .filter(|m| m.severity == Severity::Error)
                        .collect();
                    assert!(
                        errs.iter().any(|e| e.message == *expected_msg),
                        "case `{}`: expected error message `{}`, got: {:#?}",
                        c.label,
                        expected_msg,
                        msgs
                    );
                }
                Expect::Warn(expected_msg) => {
                    let warns: Vec<_> = msgs
                        .iter()
                        .filter(|m| m.severity == Severity::Warning)
                        .collect();
                    assert!(
                        warns.iter().any(|w| w.message == *expected_msg),
                        "case `{}`: expected warning message `{}`, got: {:#?}",
                        c.label,
                        expected_msg,
                        msgs
                    );
                    let errs: Vec<_> = msgs
                        .iter()
                        .filter(|m| m.severity == Severity::Error)
                        .collect();
                    assert!(
                        errs.is_empty(),
                        "case `{}`: warning case must not also emit errors: {:#?}",
                        c.label,
                        errs
                    );
                }
            }
        }
    }

    #[test]
    fn harness_compiles() {
        run_cases(&[]);
    }

    #[test]
    fn span_line_col() {
        use crate::config::types::{SourceText, Span};
        use std::sync::Arc;

        fn span_at(src: &'static str, offset: usize, len: usize) -> Span {
            Span {
                source: SourceText {
                    label: Arc::from("<t>"),
                    text: Arc::from(src),
                },
                span: miette::SourceSpan::new(offset.into(), len),
            }
        }

        assert_eq!(span_at("", 0, 0).line_col(), (1, 1));
        assert_eq!(span_at("hello", 0, 0).line_col(), (1, 1));
        assert_eq!(span_at("hello", 3, 0).line_col(), (1, 4));
        assert_eq!(span_at("a\nb", 2, 0).line_col(), (2, 1));
        assert_eq!(span_at("a\nbc", 3, 0).line_col(), (2, 2));
        assert_eq!(span_at("é", 2, 0).line_col(), (1, 2));
        assert_eq!(span_at("ab", 999, 0).line_col(), (1, 3));
    }

    use iced_layershell::reexport::Anchor;

    #[test]
    fn length_keyword_parsing() {
        use crate::config::primitives::parse_length_keyword;
        use iced::Length;
        assert!(matches!(parse_length_keyword("fill"), Some(Length::Fill)));
        assert!(matches!(
            parse_length_keyword("shrink"),
            Some(Length::Shrink)
        ));
        assert!(parse_length_keyword("portion").is_none());
        assert!(parse_length_keyword("xyz").is_none());
    }

    #[test]
    fn anchor_parsing() {
        use super::{AnchorError, parse_anchor};

        fn a(t: bool, b: bool, l: bool, r: bool) -> Anchor {
            let mut out = Anchor::empty();
            if t {
                out |= Anchor::Top;
            }
            if b {
                out |= Anchor::Bottom;
            }
            if l {
                out |= Anchor::Left;
            }
            if r {
                out |= Anchor::Right;
            }
            out
        }

        assert_eq!(parse_anchor("t").unwrap(), a(true, false, false, false));
        assert_eq!(parse_anchor("top").unwrap(), a(true, false, false, false));
        assert_eq!(parse_anchor("t | l").unwrap(), a(true, false, true, false));
        assert_eq!(parse_anchor("b | r").unwrap(), a(false, true, false, true));
        assert_eq!(
            parse_anchor("bottom | r").unwrap(),
            a(false, true, false, true)
        );
        assert_eq!(parse_anchor("b|r").unwrap(), a(false, true, false, true));

        assert_eq!(parse_anchor("l | r").unwrap(), a(false, false, true, true));
        assert_eq!(parse_anchor("t | b").unwrap(), a(true, true, false, false));
        assert_eq!(
            parse_anchor("t | l | r").unwrap(),
            a(true, false, true, true)
        );
        assert_eq!(
            parse_anchor("t | b | l | r").unwrap(),
            a(true, true, true, true)
        );
        assert!(matches!(
            parse_anchor("xyz").unwrap_err(),
            AnchorError::Unknown(_)
        ));
    }

    #[test]
    fn enum_parsers() {
        use super::*;

        assert_eq!(parse_layer("top"), Some(Layer::Top));
        assert_eq!(parse_layer("bottom"), Some(Layer::Bottom));
        assert_eq!(parse_layer("background"), Some(Layer::Background));
        assert_eq!(parse_layer("overlay"), Some(Layer::Overlay));
        assert_eq!(parse_layer("middle"), None);

        assert_eq!(parse_align_x("l"), Some(Horizontal::Left));
        assert_eq!(parse_align_x("c"), Some(Horizontal::Center));
        assert_eq!(parse_align_x("r"), Some(Horizontal::Right));
        assert_eq!(parse_align_x("left"), Some(Horizontal::Left));
        assert_eq!(parse_align_x("center"), Some(Horizontal::Center));
        assert_eq!(parse_align_x("right"), Some(Horizontal::Right));
        assert_eq!(parse_align_x("middle"), None);

        assert_eq!(parse_align_y("t"), Some(Vertical::Top));
        assert_eq!(parse_align_y("c"), Some(Vertical::Center));
        assert_eq!(parse_align_y("b"), Some(Vertical::Bottom));
        assert_eq!(parse_align_y("top"), Some(Vertical::Top));
        assert_eq!(parse_align_y("center"), Some(Vertical::Center));
        assert_eq!(parse_align_y("bottom"), Some(Vertical::Bottom));
        assert_eq!(parse_align_y("mid"), None);

        assert_eq!(parse_output("last"), OutputOption::LastOutput);
        assert_eq!(parse_output("active"), OutputOption::Active);
        assert_eq!(
            parse_output("HDMI-A-1"),
            OutputOption::OutputName("HDMI-A-1".into())
        );
    }

    #[test]
    #[allow(clippy::type_complexity)]
    fn field_helpers_compile() {
        let _: fn(
            &str,
            &kdl::KdlNode,
            &super::SourceText,
            &mut Vec<super::ConfigError>,
        ) -> Option<super::FieldValue<f32>> = super::field_f32;
    }

    #[test]
    fn variables() {
        run_cases(&[
            Case {
                label: "single int",
                kdl: "var x=1",
                expect: Expect::Ok,
            },
            Case {
                label: "single float",
                kdl: "var x=1.1",
                expect: Expect::Ok,
            },
            Case {
                label: "unquoted string",
                kdl: "var x=hello",
                expect: Expect::Ok,
            },
            Case {
                label: "quoted string",
                kdl: r#"var x="hello world""#,
                expect: Expect::Ok,
            },
            Case {
                label: "multiline string",
                kdl: "var x=\"\"\"\nhello\nworld\n\"\"\"",
                expect: Expect::Ok,
            },
            Case {
                label: "multiple decl",
                kdl: r#"var a=1 b=2.0 c="text""#,
                expect: Expect::Ok,
            },
            Case {
                label: "var with expr",
                kdl: r#"var y=1 var x="${y/2}""#,
                expect: Expect::Ok,
            },
            Case {
                label: "var with element str",
                kdl: r#"var x="container box1 w=200.5 h=40.0 child=btn1""#,
                expect: Expect::Ok,
            },
            Case {
                label: "missing value",
                kdl: "var x",
                expect: Expect::Err("variable declaration requires a value"),
            },
            Case {
                label: "duplicate var",
                kdl: "var x=1\nvar x=2",
                expect: Expect::Warn("variable x is defined twice, using first"),
            },
        ]);
    }

    #[test]
    fn widget_and_interpolation() {
        run_cases(&[
            Case {
                label: "expr in numeric field",
                kdl: "var x=40\nwidget bar h=\"${x}\" child=box1",
                expect: Expect::Ok,
            },
            Case {
                label: "expr in string field (layer)",
                kdl: "var s=\"top\"\nwidget bar layer=\"${s}\" child=box1",
                expect: Expect::Ok,
            },
            Case {
                label: "layer top",
                kdl: "widget bar layer=top child=box1",
                expect: Expect::Ok,
            },
            Case {
                label: "layer bottom",
                kdl: "widget bar layer=bottom child=box1",
                expect: Expect::Ok,
            },
            Case {
                label: "layer background",
                kdl: "widget bar layer=background child=box1",
                expect: Expect::Ok,
            },
            Case {
                label: "layer overlay",
                kdl: "widget bar layer=overlay child=box1",
                expect: Expect::Ok,
            },
            Case {
                label: "layer invalid",
                kdl: "widget bar layer=middle child=box1",
                expect: Expect::Err(
                    "invalid layer value \"middle\", expected one of: top, bottom, background, overlay",
                ),
            },
            Case {
                label: "anchor short t",
                kdl: r#"widget bar anchor="t" child=box1"#,
                expect: Expect::Ok,
            },
            Case {
                label: "anchor long top",
                kdl: r#"widget bar anchor="top" child=box1"#,
                expect: Expect::Ok,
            },
            Case {
                label: "anchor t|l",
                kdl: r#"widget bar anchor="t | l" child=box1"#,
                expect: Expect::Ok,
            },
            Case {
                label: "anchor b|r",
                kdl: r#"widget bar anchor="b | r" child=box1"#,
                expect: Expect::Ok,
            },
            Case {
                label: "anchor bottom|r",
                kdl: r#"widget bar anchor="bottom | r" child=box1"#,
                expect: Expect::Ok,
            },
            Case {
                label: "anchor b|r no spaces",
                kdl: r#"widget bar anchor="b|r" child=box1"#,
                expect: Expect::Ok,
            },
            Case {
                label: "anchor l|r stretch",
                kdl: r#"widget bar anchor="l | r" child=box1"#,
                expect: Expect::Ok,
            },
            Case {
                label: "anchor t|b stretch",
                kdl: r#"widget bar anchor="t | b" child=box1"#,
                expect: Expect::Ok,
            },
            Case {
                label: "anchor t|l|r full",
                kdl: r#"widget bar anchor="t | l | r" child=box1"#,
                expect: Expect::Ok,
            },
            Case {
                label: "anchor invalid token",
                kdl: r#"widget bar anchor="t | x" child=box1"#,
                expect: Expect::Err("invalid anchor token \"x\""),
            },
            Case {
                label: "margin 1 value",
                kdl: "widget bar margin=5 child=box1",
                expect: Expect::Ok,
            },
            Case {
                label: "margin 2 values",
                kdl: "widget bar child=box1 {\n  margin 5 10\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "margin 4 values",
                kdl: "widget bar child=box1 {\n  margin 5 10 5 10\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "margin 3 values",
                kdl: "widget bar child=box1 {\n  margin 5 10 5\n}",
                expect: Expect::Err("margin accepts 1, 2, or 4 values"),
            },
            Case {
                label: "bool exclusive true",
                kdl: "widget bar exclusive=#true child=box1",
                expect: Expect::Ok,
            },
            Case {
                label: "bool exclusive false",
                kdl: "widget bar exclusive=#false child=box1",
                expect: Expect::Ok,
            },
            Case {
                label: "bool invalid",
                kdl: "widget bar exclusive=yes child=box1",
                expect: Expect::Err("invalid bool, expected #true or #false"),
            },
            Case {
                label: "output arbitrary",
                kdl: r#"widget bar output="HDMI-A-1" child=box1"#,
                expect: Expect::Ok,
            },
            Case {
                label: "output last keyword",
                kdl: "widget bar output=last child=box1",
                expect: Expect::Ok,
            },
            Case {
                label: "output invalid (int)",
                kdl: "widget bar output=123 child=box1",
                expect: Expect::Err("invalid output value, expected a string or `last`"),
            },
        ]);
    }

    #[test]
    fn style() {
        run_cases(&[
            Case {
                label: "color rrggbb",
                kdl: "style s1 text=ffffff",
                expect: Expect::Ok,
            },
            Case {
                label: "color #rrggbb",
                kdl: r##"style s1 text="#ffffff""##,
                expect: Expect::Ok,
            },
            Case {
                label: "color rrggbbaa",
                kdl: "style s1 text=ffffffff",
                expect: Expect::Ok,
            },
            Case {
                label: "color #rrggbbaa",
                kdl: r##"style s1 text="#ffffffff""##,
                expect: Expect::Ok,
            },
            Case {
                label: "color transparent",
                kdl: "style s1 text=transparent",
                expect: Expect::Ok,
            },
            Case {
                label: "color int-as-string",
                kdl: "style s1 bg=000000",
                expect: Expect::Ok,
            },
            Case {
                label: "color bad chars",
                kdl: "style s1 text=xyz",
                expect: Expect::Err(
                    "invalid color format, expected rrggbb, rrggbbaa, #rrggbb, #rrggbbaa, transparent, or int",
                ),
            },
            Case {
                label: "color wrong length",
                kdl: "style s1 text=fffff",
                expect: Expect::Err(
                    "invalid color format, expected rrggbb, rrggbbaa, #rrggbb, #rrggbbaa, transparent, or int",
                ),
            },
        ]);
    }

    #[test]
    fn container() {
        run_cases(&[
            Case {
                label: "minimal valid",
                kdl: "container box1 child=btn1",
                expect: Expect::Ok,
            },
            Case {
                label: "missing child",
                kdl: "container box1 w=200",
                expect: Expect::Err("child is required"),
            },
            Case {
                label: "w/h int",
                kdl: "container box1 w=200 h=40 child=btn1",
                expect: Expect::Ok,
            },
            Case {
                label: "w/h float",
                kdl: "container box1 w=200.5 h=40.0 child=btn1",
                expect: Expect::Ok,
            },
            Case {
                label: "w fill",
                kdl: "container box1 w=fill child=btn1",
                expect: Expect::Ok,
            },
            Case {
                label: "w shrink",
                kdl: "container box1 w=shrink child=btn1",
                expect: Expect::Ok,
            },
            Case {
                label: "w portion block",
                kdl: "container box1 child=btn1 {\n  w portion=2\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "w portion bare",
                kdl: "container box1 w=portion child=btn1",
                expect: Expect::Err("portion requires an integer argument"),
            },
            Case {
                label: "padding 1",
                kdl: "container box1 padding=5 child=btn1",
                expect: Expect::Ok,
            },
            Case {
                label: "padding 2",
                kdl: "container box1 child=btn1 {\n  padding 5 10\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "padding 4",
                kdl: "container box1 child=btn1 {\n  padding 5 10 5 10\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "padding 3 invalid",
                kdl: "container box1 child=btn1 {\n  padding 5 10 5\n}",
                expect: Expect::Err("padding accepts 1, 2, or 4 values"),
            },
            Case {
                label: "align_x short l",
                kdl: "container box1 align_x=l child=btn1",
                expect: Expect::Ok,
            },
            Case {
                label: "align_x short c",
                kdl: "container box1 align_x=c child=btn1",
                expect: Expect::Ok,
            },
            Case {
                label: "align_x short r",
                kdl: "container box1 align_x=r child=btn1",
                expect: Expect::Ok,
            },
            Case {
                label: "align_x long left",
                kdl: "container box1 align_x=left child=btn1",
                expect: Expect::Ok,
            },
            Case {
                label: "align_x long center",
                kdl: "container box1 align_x=center child=btn1",
                expect: Expect::Ok,
            },
            Case {
                label: "align_x long right",
                kdl: "container box1 align_x=right child=btn1",
                expect: Expect::Ok,
            },
            Case {
                label: "align_x invalid",
                kdl: "container box1 align_x=middle child=btn1",
                expect: Expect::Err(
                    "invalid align_x value, expected one of: l, c, r, left, center, right",
                ),
            },
            Case {
                label: "align_y short t",
                kdl: "container box1 align_y=t child=btn1",
                expect: Expect::Ok,
            },
            Case {
                label: "align_y short c",
                kdl: "container box1 align_y=c child=btn1",
                expect: Expect::Ok,
            },
            Case {
                label: "align_y short b",
                kdl: "container box1 align_y=b child=btn1",
                expect: Expect::Ok,
            },
            Case {
                label: "align_y long top",
                kdl: "container box1 align_y=top child=btn1",
                expect: Expect::Ok,
            },
            Case {
                label: "align_y long center",
                kdl: "container box1 align_y=center child=btn1",
                expect: Expect::Ok,
            },
            Case {
                label: "align_y long bottom",
                kdl: "container box1 align_y=bottom child=btn1",
                expect: Expect::Ok,
            },
            Case {
                label: "align_y invalid",
                kdl: "container box1 align_y=mid child=btn1",
                expect: Expect::Err(
                    "invalid align_y value, expected one of: t, c, b, top, center, bottom",
                ),
            },
        ]);
    }

    #[test]
    fn border() {
        run_cases(&[
            Case {
                label: "minimal",
                kdl: "border b1",
                expect: Expect::Ok,
            },
            Case {
                label: "w single",
                kdl: "border b1 w=2.5",
                expect: Expect::Ok,
            },
            Case {
                label: "radius 1",
                kdl: "border b1 radius=5",
                expect: Expect::Ok,
            },
            Case {
                label: "radius 2",
                kdl: "border b1 {\n  radius 5 10\n}",
                expect: Expect::Err("radius accepts 1 or 4 values"),
            },
            Case {
                label: "radius 4",
                kdl: "border b1 {\n  radius 5 10 5 10\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "radius 3 invalid",
                kdl: "border b1 {\n  radius 5 10 5\n}",
                expect: Expect::Err("radius accepts 1 or 4 values"),
            },
        ]);
    }

    #[test]
    fn shadow() {
        run_cases(&[
            Case {
                label: "minimal",
                kdl: "shadow s1",
                expect: Expect::Ok,
            },
            Case {
                label: "offset 2",
                kdl: "shadow s1 {\n  offset 1 2\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "offset 1",
                kdl: "shadow s1 {\n  offset 1\n}",
                expect: Expect::Err("offset requires exactly 2 values"),
            },
            Case {
                label: "offset 3",
                kdl: "shadow s1 {\n  offset 1 2 3\n}",
                expect: Expect::Err("offset requires exactly 2 values"),
            },
        ]);
    }

    #[test]
    fn button() {
        run_cases(&[
            Case {
                label: "minimal — no child required",
                kdl: "button btn1",
                expect: Expect::Ok,
            },
            Case {
                label: "action string",
                kdl: r#"button btn1 action="echo hello""#,
                expect: Expect::Ok,
            },
            Case {
                label: "style variants",
                kdl: "button btn1 style=s1 style:hover=s1 style:active=s1 style:disabled=s1",
                expect: Expect::Ok,
            },
            Case {
                label: "padding 3 invalid",
                kdl: "button btn1 {\n  padding 5 10 5\n}",
                expect: Expect::Err("padding accepts 1, 2, or 4 values"),
            },
            Case {
                label: "action non-string",
                kdl: "button btn1 action=42",
                expect: Expect::Err("field `action` expects a string"),
            },
        ]);
    }

    #[test]
    fn row() {
        run_cases(&[
            Case {
                label: "minimal one child",
                kdl: "row r1 {\n  children btn1\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "missing children",
                kdl: "row r1 w=40",
                expect: Expect::Err("children is required"),
            },
            Case {
                label: "empty children",
                kdl: "row r1 {\n  children\n}",
                expect: Expect::Err("children requires at least one id"),
            },
            Case {
                label: "multiple children",
                kdl: "row r1 {\n  children btn1 btn2 btn3\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "align t",
                kdl: "row r1 align=t {\n  children btn1\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "align c",
                kdl: "row r1 align=c {\n  children btn1\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "align b",
                kdl: "row r1 align=b {\n  children btn1\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "align top",
                kdl: "row r1 align=top {\n  children btn1\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "align bottom",
                kdl: "row r1 align=bottom {\n  children btn1\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "align left invalid",
                kdl: "row r1 align=left {\n  children btn1\n}",
                expect: Expect::Err(
                    "invalid align value for row, expected one of: t, c, b, top, center, bottom",
                ),
            },
            Case {
                label: "spacing int",
                kdl: "row r1 spacing=5 {\n  children btn1\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "spacing float",
                kdl: "row r1 spacing=1.5 {\n  children btn1\n}",
                expect: Expect::Ok,
            },
        ]);
    }

    #[test]
    fn column() {
        run_cases(&[
            Case {
                label: "minimal one child",
                kdl: "column c1 {\n  children btn1\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "missing children",
                kdl: "column c1 w=40",
                expect: Expect::Err("children is required"),
            },
            Case {
                label: "empty children",
                kdl: "column c1 {\n  children\n}",
                expect: Expect::Err("children requires at least one id"),
            },
            Case {
                label: "align l",
                kdl: "column c1 align=l {\n  children btn1\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "align c",
                kdl: "column c1 align=c {\n  children btn1\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "align r",
                kdl: "column c1 align=r {\n  children btn1\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "align left",
                kdl: "column c1 align=left {\n  children btn1\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "align right",
                kdl: "column c1 align=right {\n  children btn1\n}",
                expect: Expect::Ok,
            },
            Case {
                label: "align top invalid",
                kdl: "column c1 align=top {\n  children btn1\n}",
                expect: Expect::Err(
                    "invalid align value for column, expected one of: l, c, r, left, center, right",
                ),
            },
        ]);
    }

    #[test]
    fn text() {
        run_cases(&[
            Case {
                label: "minimal",
                kdl: "text t1",
                expect: Expect::Ok,
            },
            Case {
                label: "font string",
                kdl: r#"text t1 font="Symbols Nerd Font""#,
                expect: Expect::Ok,
            },
            Case {
                label: "color field",
                kdl: "text t1 color=ffffff",
                expect: Expect::Ok,
            },
            Case {
                label: "color invalid",
                kdl: "text t1 color=xyz",
                expect: Expect::Err(
                    "invalid color format, expected rrggbb, rrggbbaa, #rrggbb, #rrggbbaa, transparent, or int",
                ),
            },
            Case {
                label: "align_x invalid",
                kdl: "text t1 align_x=middle",
                expect: Expect::Err(
                    "invalid align_x value, expected one of: l, c, r, j, left, center, right, justified",
                ),
            },
            Case {
                label: "align_y invalid",
                kdl: "text t1 align_y=mid",
                expect: Expect::Err(
                    "invalid align_y value, expected one of: t, c, b, top, center, bottom",
                ),
            },
            Case {
                label: "text non-string",
                kdl: "text t1 text=99",
                expect: Expect::Err("field `text` expects a string"),
            },
        ]);
    }

    #[test]
    fn apptray_block() {
        run_cases(&[
            Case {
                label: "minimal",
                kdl: "apptray icon_size=24 spacing=6",
                expect: Expect::Ok,
            },
            Case {
                label: "ref as child",
                kdl: "widget bar child=apptray\napptray icon_size=20",
                expect: Expect::Ok,
            },
            Case {
                label: "swap",
                kdl: "apptray swap_buttons=#true",
                expect: Expect::Ok,
            },
            Case {
                label: "reserved widget id",
                kdl: "widget apptray child=t1\ntext t1",
                expect: Expect::Err("\"apptray\" is a reserved id"),
            },
            Case {
                label: "reserved text id",
                kdl: "text apptray",
                expect: Expect::Err("\"apptray\" is a reserved id"),
            },
            Case {
                label: "duplicate block",
                kdl: "apptray icon_size=24\napptray icon_size=30",
                expect: Expect::Warn("apptray block is defined twice, using first"),
            },
        ]);
    }

    #[test]
    fn notification_block() {
        run_cases(&[
            Case {
                label: "minimal",
                kdl: "notification width=400 height=110",
                expect: Expect::Ok,
            },
            Case {
                label: "colors",
                kdl: "notification primary_text=ffffff secondary_text=cccccc bg=222222",
                expect: Expect::Ok,
            },
            Case {
                label: "anchor",
                kdl: r#"notification anchor="t | r""#,
                expect: Expect::Ok,
            },
            Case {
                label: "layer",
                kdl: "notification layer=overlay",
                expect: Expect::Ok,
            },
            Case {
                label: "border ref",
                kdl: "notification border=nb\nborder nb color=444444 w=1 radius=8",
                expect: Expect::Ok,
            },
            Case {
                label: "bad color",
                kdl: "notification bg=xyz",
                expect: Expect::Err(
                    "invalid color format, expected rrggbb, rrggbbaa, #rrggbb, #rrggbbaa, transparent, or int",
                ),
            },
            Case {
                label: "bad layer",
                kdl: "notification layer=middle",
                expect: Expect::Err(
                    "invalid layer value \"middle\", expected one of: top, bottom, background, overlay",
                ),
            },
            Case {
                label: "duplicate",
                kdl: "notification width=400\nnotification width=500",
                expect: Expect::Warn("notification block is defined twice, using first"),
            },
        ]);
    }

    #[test]
    fn pull_block() {
        run_cases(&[
            Case {
                label: "basic",
                kdl: r#"pull dt="date" i="1s""#,
                expect: Expect::Ok,
            },
            Case {
                label: "interval alias",
                kdl: r#"pull dt="date" interval="500ms""#,
                expect: Expect::Ok,
            },
            Case {
                label: "with default",
                kdl: r#"pull dt="date" i="1s" default="…""#,
                expect: Expect::Ok,
            },
            Case {
                label: "bad interval",
                kdl: r#"pull dt="date" i="1x""#,
                expect: Expect::Err("invalid interval \"1x\""),
            },
            Case {
                label: "missing interval",
                kdl: r#"pull dt="date""#,
                expect: Expect::Err("pull requires an interval (i= or interval=)"),
            },
            Case {
                label: "no var",
                kdl: r#"pull i="1s""#,
                expect: Expect::Err("a pull must name exactly one variable"),
            },
            Case {
                label: "dup name",
                kdl: "var dt=1\npull dt=\"date\" i=\"1s\"",
                expect: Expect::Warn("variable dt is defined twice, using first"),
            },
        ]);
    }
}
