use crate::config::math::{self, value::Value};
use crate::config::resolved::{ ResolvedButton, ResolvedColumn, ResolvedContainer, ResolvedElement,
    ResolvedRow, ResolvedShadow, ResolvedStyle, ResolvedText, ResolvedWidget,
};
use crate::config::resolver::coerce;
use crate::config::resolver::vars::FlatEnv;
use crate::config::types::{
    Button, Column, Container, FieldValue, ParsedConfig, Row, Span, Style, TextEl, Widget,
};
use crate::config::{ConfigError, ConfigErrorKind, Severity};
use std::collections::HashSet;


pub(crate) struct Ctx<'a> {
    pub config: &'a ParsedConfig,
    pub env: &'a FlatEnv,
    pub errs: &'a mut Vec<ConfigError>,
    pub used: &'a mut HashSet<String>,
}

fn mark_expr_vars_used(s: &str, ctx: &mut Ctx) {
    for name in crate::config::resolver::vars::referenced_vars(s) {
        if ctx.config.vars.contains_key(&name) {
            ctx.used.insert(name);
        }
    }
}

pub(crate) fn resolve_field<T: Clone>(
    field: &Option<FieldValue<T>>,
    field_name: &str,
    span: &Span,
    coerce_fn: impl Fn(Value, &str, &Span) -> Result<T, ConfigError>,
    ctx: &mut Ctx,
) -> Option<T> {
    match field {
        None => None,
        Some(FieldValue::Literal(t)) => Some(t.clone()),
        Some(FieldValue::Expr(s)) => {
            mark_expr_vars_used(s, ctx);
            match math::evaluate(s, ctx.env, &span.source, 0) {
                Ok(value) => match coerce_fn(value, field_name, span) {
                    Ok(t) => Some(t),
                    Err(e) => {
                        ctx.errs.push(e);
                        None
                    }
                },
                Err(eval_err) => {
                    ctx.errs.push(ConfigError {
                        kind: ConfigErrorKind::Expression,
                        span: span.clone(),
                        message: eval_err.message,
                        severity: Severity::Error,
                    });
                    None
                }
            }
        }
    }
}

pub(crate) fn resolve_widget(_name: &str, w: &Widget, ctx: &mut Ctx) -> ResolvedWidget {
    let mut visited = HashSet::new();
    let child = match &w.child {
        None => None,
        Some(FieldValue::Literal(id)) => {
            let id = id.clone();
            resolve_ref(&id, &w.span, ctx, &mut visited).map(Box::new)
        }
        Some(FieldValue::Expr(s)) => {
            let s = s.clone();
            let span = w.span.clone();
            mark_expr_vars_used(&s, ctx);
            resolve_string_ref(&s, &span, ctx, &mut visited).map(Box::new)
        }
    };
    ResolvedWidget {
        h: resolve_field(&w.h, "h", &w.span, coerce::coerce_f32, ctx),
        w: resolve_field(&w.w, "w", &w.span, coerce::coerce_f32, ctx),
        layer: resolve_field(&w.layer, "layer", &w.span, coerce::coerce_layer, ctx),
        anchor: resolve_field(&w.anchor, "anchor", &w.span, coerce::coerce_anchor, ctx),
        exclusive: resolve_field(&w.exclusive, "exclusive", &w.span, coerce::coerce_bool, ctx),
        margin: resolve_field(&w.margin, "margin", &w.span, coerce::coerce_margin, ctx),
        output: resolve_field(&w.output, "output", &w.span, coerce::coerce_output, ctx),
        keyboard: resolve_field(&w.keyboard, "keyboard", &w.span, coerce::coerce_bool, ctx),
        transparent: resolve_field(
            &w.transparent,
            "transparent",
            &w.span,
            coerce::coerce_bool,
            ctx,
        ),
        child,
        span: w.span.clone(),
    }
}

pub(crate) fn resolve_ref(
    reference: &str,
    span: &Span,
    ctx: &mut Ctx,
    visited: &mut HashSet<String>,
) -> Option<ResolvedElement> {
    if visited.contains(reference) {
        ctx.errs.push(ConfigError {
            kind: ConfigErrorKind::CircularReference,
            span: span.clone(),
            message: format!("circular reference detected at \"{}\"", reference),
            severity: Severity::Error,
        });
        return None;
    }

    if reference == "apptray" {
        return Some(ResolvedElement::Apptray(resolve_apptray_settings(ctx)));
    }

    if ctx.config.texts.contains_key(reference) {
        ctx.used.insert(reference.to_string());
        visited.insert(reference.to_string());
        let t = ctx.config.texts.get(reference).unwrap().clone();
        let resolved = resolve_text(&t, ctx);
        visited.remove(reference);
        return Some(ResolvedElement::Text(resolved));
    }

    if ctx.config.containers.contains_key(reference) {
        let c = ctx.config.containers.get(reference).unwrap().clone();
        ctx.used.insert(reference.to_string());
        visited.insert(reference.to_string());
        let resolved = resolve_container(&c, ctx, visited);
        visited.remove(reference);
        return resolved.map(ResolvedElement::Container);
    }

    if ctx.config.buttons.contains_key(reference) {
        let b = ctx.config.buttons.get(reference).unwrap().clone();
        ctx.used.insert(reference.to_string());
        let resolved = resolve_button(&b, ctx);
        return Some(ResolvedElement::Button(Box::new(resolved)));
    }

    if ctx.config.rows.contains_key(reference) {
        let r = ctx.config.rows.get(reference).unwrap().clone();
        ctx.used.insert(reference.to_string());
        visited.insert(reference.to_string());
        let resolved = resolve_row(&r, ctx, visited);
        visited.remove(reference);
        return Some(ResolvedElement::Row(resolved));
    }

    if ctx.config.columns.contains_key(reference) {
        let col = ctx.config.columns.get(reference).unwrap().clone();
        ctx.used.insert(reference.to_string());
        visited.insert(reference.to_string());
        let resolved = resolve_column(&col, ctx, visited);
        visited.remove(reference);
        return Some(ResolvedElement::Column(resolved));
    }

    if let Some(crate::config::types::VarValue::Str(s)) = ctx.env.lookup_value(reference) {
        let s = s.clone();
        ctx.used.insert(reference.to_string());
        if visited.contains(reference) {
            ctx.errs.push(ConfigError {
                kind: ConfigErrorKind::CircularReference,
                span: span.clone(),
                message: format!("circular reference detected at \"{}\"", reference),
                severity: Severity::Error,
            });
            return None;
        }
        visited.insert(reference.to_string());
        let out = resolve_ref(&s, span, ctx, visited);
        visited.remove(reference);
        return out;
    }

    {
        let (frag_cfg, _frag_errs) = crate::config::parse_str(reference, "<fragment>");
        if let Some(frag) = frag_cfg
            && let Some(el) = single_element_owned(&frag)
        {
            return resolve_fragment_element(el, ctx, visited);
        }
    }

    ctx.errs.push(ConfigError {
        kind: ConfigErrorKind::UnresolvedReference,
        span: span.clone(),
        message: format!("unresolved reference \"{}\"", reference),
        severity: Severity::Error,
    });
    None
}

enum OwnedEl {
    Container(Container),
    Button(Button),
    Row(Row),
    Column(Column),
    Text(TextEl),
}

fn single_element_owned(frag: &ParsedConfig) -> Option<OwnedEl> {
    let count = frag.containers.len()
        + frag.buttons.len()
        + frag.rows.len()
        + frag.columns.len()
        + frag.texts.len();
    if count != 1 {
        return None;
    }
    if let Some((_, c)) = frag.containers.iter().next() {
        return Some(OwnedEl::Container(c.clone()));
    }
    if let Some((_, b)) = frag.buttons.iter().next() {
        return Some(OwnedEl::Button(b.clone()));
    }
    if let Some((_, r)) = frag.rows.iter().next() {
        return Some(OwnedEl::Row(r.clone()));
    }
    if let Some((_, c)) = frag.columns.iter().next() {
        return Some(OwnedEl::Column(c.clone()));
    }
    if let Some((_, t)) = frag.texts.iter().next() {
        return Some(OwnedEl::Text(t.clone()));
    }
    None
}

fn resolve_fragment_element(
    el: OwnedEl,
    ctx: &mut Ctx,
    visited: &mut HashSet<String>,
) -> Option<ResolvedElement> {
    match el {
        OwnedEl::Container(c) => {
            resolve_container(&c, ctx, visited).map(ResolvedElement::Container)
        }
        OwnedEl::Button(b) => Some(ResolvedElement::Button(Box::new(resolve_button(&b, ctx)))),
        OwnedEl::Row(r) => Some(ResolvedElement::Row(resolve_row(&r, ctx, visited))),
        OwnedEl::Column(c) => Some(ResolvedElement::Column(resolve_column(&c, ctx, visited))),
        OwnedEl::Text(t) => Some(ResolvedElement::Text(resolve_text(&t, ctx))),
    }
}

fn resolve_container(
    c: &Container,
    ctx: &mut Ctx,
    visited: &mut HashSet<String>,
) -> Option<ResolvedContainer> {
    let child = match &c.child {
        None => None,
        Some(FieldValue::Literal(id)) => {
            let id = id.clone();
            resolve_ref(&id, &c.span, ctx, visited)
        }
        Some(FieldValue::Expr(s)) => {
            let s = s.clone();
            resolve_string_ref(&s, &c.span, ctx, visited)
        }
    };
    let child = match child {
        Some(el) => Box::new(el),
        None => return None,
    };
    Some(ResolvedContainer {
        w: resolve_field(&c.w, "w", &c.span, coerce::coerce_length, ctx),
        h: resolve_field(&c.h, "h", &c.span, coerce::coerce_length, ctx),
        padding: resolve_field(&c.padding, "padding", &c.span, coerce::coerce_padding, ctx),
        align_x: resolve_field(&c.align_x, "align_x", &c.span, coerce::coerce_align_x, ctx),
        align_y: resolve_field(&c.align_y, "align_y", &c.span, coerce::coerce_align_y, ctx),
        clip: resolve_field(&c.clip, "clip", &c.span, coerce::coerce_bool, ctx),
        style: resolve_style_ref(&c.style, &c.span, ctx),
        child,
        span: c.span.clone(),
    })
}

fn resolve_button(b: &Button, ctx: &mut Ctx) -> ResolvedButton {
    ResolvedButton {
        w: resolve_field(&b.w, "w", &b.span, coerce::coerce_length, ctx),
        h: resolve_field(&b.h, "h", &b.span, coerce::coerce_length, ctx),
        padding: resolve_field(&b.padding, "padding", &b.span, coerce::coerce_padding, ctx),
        action: resolve_field(&b.action, "action", &b.span, coerce::coerce_string, ctx),
        clip: resolve_field(&b.clip, "clip", &b.span, coerce::coerce_bool, ctx),
        style: resolve_style_ref(&b.style, &b.span, ctx),
        style_hover: resolve_style_ref(&b.style_hover, &b.span, ctx),
        style_active: resolve_style_ref(&b.style_active, &b.span, ctx),
        style_disabled: resolve_style_ref(&b.style_disabled, &b.span, ctx),
        text: resolve_field(&b.text, "text", &b.span, coerce::coerce_string, ctx),
        font: resolve_field(&b.font, "font", &b.span, coerce::coerce_string, ctx),
        span: b.span.clone(),
    }
}

fn resolve_children(
    children: &Option<FieldValue<Vec<String>>>,
    span: &Span,
    ctx: &mut Ctx,
    visited: &mut HashSet<String>,
) -> Vec<ResolvedElement> {
    let ids: Vec<String> = match children {
        Some(FieldValue::Literal(ids)) => ids.clone(),
        Some(FieldValue::Expr(_)) | None => Vec::new(),
    };
    let mut out = Vec::new();
    for id in &ids {
        if let Some(el) = resolve_ref(id, span, ctx, visited) {
            out.push(el);
        }
    }
    out
}

fn resolve_row(r: &Row, ctx: &mut Ctx, visited: &mut HashSet<String>) -> ResolvedRow {
    let children = resolve_children(&r.children, &r.span, ctx, visited);
    ResolvedRow {
        children,
        w: resolve_field(&r.w, "w", &r.span, coerce::coerce_length, ctx),
        h: resolve_field(&r.h, "h", &r.span, coerce::coerce_length, ctx),
        padding: resolve_field(&r.padding, "padding", &r.span, coerce::coerce_padding, ctx),
        spacing: resolve_field(&r.spacing, "spacing", &r.span, coerce::coerce_f32, ctx),
        clip: resolve_field(&r.clip, "clip", &r.span, coerce::coerce_bool, ctx),
        align: resolve_field(&r.align, "align", &r.span, coerce::coerce_row_align, ctx),
        span: r.span.clone(),
    }
}

fn resolve_column(c: &Column, ctx: &mut Ctx, visited: &mut HashSet<String>) -> ResolvedColumn {
    let children = resolve_children(&c.children, &c.span, ctx, visited);
    ResolvedColumn {
        children,
        w: resolve_field(&c.w, "w", &c.span, coerce::coerce_length, ctx),
        h: resolve_field(&c.h, "h", &c.span, coerce::coerce_length, ctx),
        padding: resolve_field(&c.padding, "padding", &c.span, coerce::coerce_padding, ctx),
        spacing: resolve_field(&c.spacing, "spacing", &c.span, coerce::coerce_f32, ctx),
        clip: resolve_field(&c.clip, "clip", &c.span, coerce::coerce_bool, ctx),
        align: resolve_field(&c.align, "align", &c.span, coerce::coerce_col_align, ctx),
        span: c.span.clone(),
    }
}

fn resolve_string_ref(
    s: &str,
    span: &Span,
    ctx: &mut Ctx,
    visited: &mut HashSet<String>,
) -> Option<ResolvedElement> {
    mark_expr_vars_used(s, ctx);
    match math::evaluate(s, ctx.env, &span.source, 0) {
        Ok(value) => match coerce::coerce_string(value, "child", span) {
            Ok(id) => resolve_ref(&id, span, ctx, visited),
            Err(e) => {
                ctx.errs.push(e);
                None
            }
        },
        Err(eval_err) => {
            ctx.errs.push(ConfigError {
                kind: ConfigErrorKind::Expression,
                span: span.clone(),
                message: eval_err.message,
                severity: Severity::Error,
            });
            None
        }
    }
}

fn literal_or_eval_id(
    id_field: &Option<FieldValue<String>>,
    span: &Span,
    ctx: &mut Ctx,
) -> Option<String> {
    match id_field {
        None => None,
        Some(FieldValue::Literal(id)) => Some(id.clone()),
        Some(FieldValue::Expr(s)) => {
            let s = s.clone();
            mark_expr_vars_used(&s, ctx);
            match math::evaluate(&s, ctx.env, &span.source, 0) {
                Ok(v) => coerce::coerce_string(v, "ref", span).ok(),
                Err(e) => {
                    ctx.errs.push(ConfigError {
                        kind: ConfigErrorKind::Expression,
                        span: span.clone(),
                        message: e.message,
                        severity: Severity::Error,
                    });
                    None
                }
            }
        }
    }
}

fn resolve_style_ref(
    id_field: &Option<FieldValue<String>>,
    span: &Span,
    ctx: &mut Ctx,
) -> Option<ResolvedStyle> {
    let id = literal_or_eval_id(id_field, span, ctx)?;
    if !ctx.config.styles.contains_key(&id) {
        ctx.errs.push(ConfigError {
            kind: ConfigErrorKind::UnresolvedReference,
            span: span.clone(),
            message: format!("unresolved reference \"{}\"", id),
            severity: Severity::Error,
        });
        return None;
    }
    let style = ctx.config.styles.get(&id).unwrap().clone();
    ctx.used.insert(id);
    Some(resolve_style(&style, span, ctx))
}

fn resolve_style(s: &Style, span: &Span, ctx: &mut Ctx) -> ResolvedStyle {
    ResolvedStyle {
        text: resolve_field(&s.text, "text", &s.span, coerce::coerce_color, ctx),
        bg: resolve_field(&s.bg, "bg", &s.span, coerce::coerce_color, ctx),
        border: resolve_border_ref(&s.border, span, ctx),
        shadow: resolve_shadow_ref(&s.shadow, span, ctx),
        snap: resolve_field(&s.snap, "snap", &s.span, coerce::coerce_bool, ctx),
    }
}

pub(crate) fn resolve_border_ref(
    id_field: &Option<FieldValue<String>>,
    span: &Span,
    ctx: &mut Ctx,
) -> Option<ResolvedBorder> {
    let id = literal_or_eval_id(id_field, span, ctx)?;
    if !ctx.config.borders.contains_key(&id) {
        ctx.errs.push(ConfigError {
            kind: ConfigErrorKind::UnresolvedReference,
            span: span.clone(),
            message: format!("unresolved reference \"{}\"", id),
            severity: Severity::Error,
        });
        return None;
    }
    let b = ctx.config.borders.get(&id).unwrap().clone();
    ctx.used.insert(id);
    Some(ResolvedBorder {
        color: resolve_field(&b.color, "color", &b.span, coerce::coerce_color, ctx),
        w: resolve_field(&b.w, "w", &b.span, coerce::coerce_f32, ctx),
        radius: resolve_field(&b.radius, "radius", &b.span, coerce::coerce_radius, ctx),
    })
}

fn resolve_shadow_ref(
    id_field: &Option<FieldValue<String>>,
    span: &Span,
    ctx: &mut Ctx,
) -> Option<ResolvedShadow> {
    let id = literal_or_eval_id(id_field, span, ctx)?;
    if !ctx.config.shadows.contains_key(&id) {
        ctx.errs.push(ConfigError {
            kind: ConfigErrorKind::UnresolvedReference,
            span: span.clone(),
            message: format!("unresolved reference \"{}\"", id),
            severity: Severity::Error,
        });
        return None;
    }
    let sh = ctx.config.shadows.get(&id).unwrap().clone();
    ctx.used.insert(id);
    let offset = match &sh.offset {
        Some(FieldValue::Literal(o)) => Some(*o),
        _ => None,
    };
    Some(ResolvedShadow {
        color: resolve_field(&sh.color, "color", &sh.span, coerce::coerce_color, ctx),
        offset,
        blur_radius: resolve_field(
            &sh.blur_radius,
            "blur_radius",
            &sh.span,
            coerce::coerce_f32,
            ctx,
        ),
    })
}

use crate::config::resolved::ResolvedApptraySettings;

pub(crate) fn resolve_apptray_settings(ctx: &mut Ctx) -> ResolvedApptraySettings {
    let mut out = ResolvedApptraySettings::default();
    let Some(a) = ctx.config.apptray.clone() else {
        return out;
    };
    let span = a.span.clone();
    if let Some(v) = resolve_field(&a.icon_size, "icon_size", &span, coerce::coerce_f32, ctx) {
        out.icon_size = v;
    }
    if let Some(v) = resolve_field(&a.spacing, "spacing", &span, coerce::coerce_f32, ctx) {
        out.spacing = v;
    }
    out.padding = resolve_field(&a.padding, "padding", &span, coerce::coerce_padding, ctx);
    out.bg = resolve_field(&a.bg, "bg", &span, coerce::coerce_color, ctx);
    out.border = resolve_border_ref(&a.border, &span, ctx);
    if let Some(v) = resolve_field(
        &a.swap_buttons,
        "swap_buttons",
        &span,
        coerce::coerce_bool,
        ctx,
    ) {
        out.swap_buttons = v;
    }
    if let Some(v) = resolve_field(&a.vertical, "vertical", &span, coerce::coerce_bool, ctx) {
        out.vertical = v;
    }
    if let (Some(v), Some(s)) = (
        resolve_field(&a.menu_bg, "menu_bg", &span, coerce::coerce_color, ctx),
        out.menu.menu_container_style.as_mut(),
    ) {
        s.bg = Some(v);
    }
    // Transitional: menu_text recolors only the normal button text; hover/active
    // keep their defaults. (raw `menu_width` is intentionally no longer consumed.)
    // Full per-state styling comes with the future style-block parser.
    if let (Some(v), Some(s)) = (
        resolve_field(&a.menu_text, "menu_text", &span, coerce::coerce_color, ctx),
        out.menu.button_style.as_mut(),
    ) {
        s.text = Some(v);
    }
    if let (Some(v), Some(s)) = (
        resolve_field(
            &a.menu_disabled,
            "menu_disabled",
            &span,
            coerce::coerce_color,
            ctx,
        ),
        out.menu.button_style_disabled.as_mut(),
    ) {
        s.text = Some(v);
    }
    if let Some(v) = resolve_field(&a.row_height, "row_height", &span, coerce::coerce_f32, ctx) {
        out.menu.row_height = v;
    }
    out
}

pub(crate) fn resolve_text(t: &TextEl, ctx: &mut Ctx) -> ResolvedText {
    ResolvedText {
        w: resolve_field(&t.w, "w", &t.span, coerce::coerce_length, ctx),
        h: resolve_field(&t.h, "h", &t.span, coerce::coerce_length, ctx),
        align_x: resolve_field(&t.align_x, "align_x", &t.span, coerce::coerce_align_x, ctx),
        align_y: resolve_field(&t.align_y, "align_y", &t.span, coerce::coerce_align_y, ctx),
        color: resolve_field(&t.color, "color", &t.span, coerce::coerce_color, ctx),
        font: resolve_field(&t.font, "font", &t.span, coerce::coerce_string, ctx),
        content: resolve_field(&t.content, "text", &t.span, coerce::coerce_string, ctx),
        span: t.span.clone(),
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Severity;
    use crate::config::parse_str;
    use crate::config::resolved::ResolvedElement;
    use crate::config::resolver::resolve;

    fn resolve_kdl(
        kdl: &str,
    ) -> (
        Option<crate::config::resolved::ResolvedConfig>,
        Vec<crate::config::ConfigError>,
    ) {
        let (cfg, perrs) = parse_str(kdl, "<test>");
        let cfg = cfg.expect("parse ok");
        assert!(
            perrs.iter().all(|e| e.severity != Severity::Error),
            "fixture parse errs: {:?}",
            perrs
        );
        resolve(&cfg)
    }

    #[test]
    fn widget_with_text_child() {
        let (rc, errs) = resolve_kdl("widget bar child=t1\ntext t1 color=ffffff");
        assert!(
            errs.iter().all(|e| e.severity != Severity::Error),
            "errs: {:?}",
            errs
        );
        let rc = rc.expect("resolved");
        let bar = rc.widgets.get("bar").expect("bar widget");
        match bar.child.as_deref() {
            Some(ResolvedElement::Text(t)) => {
                assert_eq!(
                    t.color,
                    Some(iced::Color::from("ffffff"))
                );
            }
            other => panic!("expected text child, got {:?}", other),
        }
    }

    #[test]
    fn widget_field_expression() {
        let (rc, errs) = resolve_kdl("var hh=40\nwidget bar h=\"${hh}\" child=t1\ntext t1");
        assert!(
            errs.iter().all(|e| e.severity != Severity::Error),
            "errs: {:?}",
            errs
        );
        let rc = rc.unwrap();
        assert_eq!(rc.widgets.get("bar").unwrap().h, Some(40.0));
    }

    #[test]
    fn unresolved_child() {
        let (rc, errs) = resolve_kdl("widget bar child=nope");
        assert!(rc.is_none());
        assert!(
            errs.iter()
                .any(|e| e.kind == crate::config::ConfigErrorKind::UnresolvedReference)
        );
    }

    #[test]
    fn widget_container_text_chain() {
        let (rc, errs) =
            resolve_kdl("widget bar child=box1\ncontainer box1 child=t1\ntext t1 color=ffffff");
        assert!(
            errs.iter().all(|e| e.severity != Severity::Error),
            "errs: {:?}",
            errs
        );
        let rc = rc.unwrap();
        let bar = rc.widgets.get("bar").unwrap();
        match bar.child.as_deref() {
            Some(ResolvedElement::Container(c)) => {
                assert!(matches!(c.child.as_ref(), ResolvedElement::Text(_)));
            }
            other => panic!("expected container, got {:?}", other),
        }
    }

    #[test]
    fn out_of_order_definitions() {
        let (rc, errs) = resolve_kdl("text t1\nwidget bar child=box1\ncontainer box1 child=t1");
        assert!(
            errs.iter().all(|e| e.severity != Severity::Error),
            "errs: {:?}",
            errs
        );
        assert!(rc.is_some());
    }

    #[test]
    fn circular_container_refs() {
        let (rc, errs) =
            resolve_kdl("widget bar child=a\ncontainer a child=b\ncontainer b child=a");
        assert!(rc.is_none());
        assert!(
            errs.iter()
                .any(|e| e.kind == crate::config::ConfigErrorKind::CircularReference),
            "expected CircularReference, got {:?}",
            errs
        );
    }

    #[test]
    fn button_no_child() {
        let (rc, errs) = resolve_kdl("widget bar child=btn\nbutton btn action=\"echo hi\"");
        assert!(
            errs.iter().all(|e| e.severity != Severity::Error),
            "errs: {:?}",
            errs
        );
        let rc = rc.unwrap();
        match rc.widgets.get("bar").unwrap().child.as_deref() {
            Some(ResolvedElement::Button(b)) => assert_eq!(b.action.as_deref(), Some("echo hi")),
            other => panic!("expected button, got {:?}", other),
        }
    }

    #[test]
    fn row_with_multiple_children() {
        let (rc, errs) =
            resolve_kdl("widget bar child=r1\nrow r1 {\n  children a b\n}\nbutton a\nbutton b");
        assert!(
            errs.iter().all(|e| e.severity != Severity::Error),
            "errs: {:?}",
            errs
        );
        let rc = rc.unwrap();
        match rc.widgets.get("bar").unwrap().child.as_deref() {
            Some(ResolvedElement::Row(r)) => assert_eq!(r.children.len(), 2),
            other => panic!("expected row, got {:?}", other),
        }
    }

    #[test]
    fn column_with_shared_child_duplicated() {
        let (rc, errs) =
            resolve_kdl("widget bar child=c1\ncolumn c1 {\n  children b b\n}\nbutton b");
        assert!(
            errs.iter().all(|e| e.severity != Severity::Error),
            "errs: {:?}",
            errs
        );
        let rc = rc.unwrap();
        match rc.widgets.get("bar").unwrap().child.as_deref() {
            Some(ResolvedElement::Column(c)) => assert_eq!(c.children.len(), 2),
            other => panic!("expected column, got {:?}", other),
        }
    }

    #[test]
    fn container_style_inlined_with_border() {
        let (rc, errs) = resolve_kdl(
            "widget bar child=box1\ncontainer box1 style=s1 child=t1\ntext t1\nstyle s1 bg=000000 border=b1\nborder b1 radius=5",
        );
        assert!(
            errs.iter().all(|e| e.severity != Severity::Error),
            "errs: {:?}",
            errs
        );
        let rc = rc.unwrap();
        match rc.widgets.get("bar").unwrap().child.as_deref() {
            Some(ResolvedElement::Container(c)) => {
                let style = c.style.as_ref().expect("style inlined");
                assert_eq!(
                    style.bg,
                    Some(crate::config::types::Color {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: 0xff
                    })
                );
                let border = style.border.as_ref().expect("border inlined");
                assert_eq!(
                    border.radius,
                    Some(iced::border::Radius {
                        top_left: 5.0,
                        top_right: 5.0,
                        bottom_right: 5.0,
                        bottom_left: 5.0
                    })
                );
            }
            other => panic!("expected container, got {:?}", other),
        }
    }

    #[test]
    fn unresolved_style_ref() {
        let (rc, errs) =
            resolve_kdl("widget bar child=box1\ncontainer box1 style=nope child=t1\ntext t1");
        assert!(rc.is_none());
        assert!(
            errs.iter()
                .any(|e| e.kind == crate::config::ConfigErrorKind::UnresolvedReference)
        );
    }

    #[test]
    fn var_redirect_to_element_id() {
        let (rc, errs) = resolve_kdl("var y=\"t1\"\nwidget bar child=y\ntext t1 color=ffffff");
        assert!(
            errs.iter().all(|e| e.severity != Severity::Error),
            "errs: {:?}",
            errs
        );
        let rc = rc.unwrap();
        assert!(matches!(
            rc.widgets.get("bar").unwrap().child.as_deref(),
            Some(ResolvedElement::Text(_))
        ));
    }

    #[test]
    fn var_fragment_expands() {
        let (rc, errs) = resolve_kdl(
            "var x=\"container c1 child=t1\"\nwidget bar child=x\ntext t1 color=ffffff",
        );
        assert!(
            errs.iter().all(|e| e.severity != Severity::Error),
            "errs: {:?}",
            errs
        );
        let rc = rc.unwrap();
        match rc.widgets.get("bar").unwrap().child.as_deref() {
            Some(ResolvedElement::Container(c)) => {
                assert!(matches!(c.child.as_ref(), ResolvedElement::Text(_)))
            }
            other => panic!("expected expanded container, got {:?}", other),
        }
    }
}
