use crate::config::types::VarValue;
use iced_layershell::reexport::Anchor;

fn parse_value(raw: &str) -> VarValue {
    if let Ok(i) = raw.parse::<i128>() {
        return VarValue::Int(i);
    }
    if let Ok(f) = raw.parse::<f64>() {
        return VarValue::Float(f);
    }
    match raw {
        "#true" | "true" => VarValue::Bool(true),
        "#false" | "false" => VarValue::Bool(false),
        _ => VarValue::Str(raw.to_string()),
    }
}

use crate::config::resolved::ResolvedConfig;
use crate::config::resolver::resolve;
use crate::config::types::ParsedConfig;
use crate::config::{ConfigError, ConfigErrorKind, Severity};
use crate::config::{LoadError, load_from_path};

fn load_error_lines(e: LoadError) -> Vec<String> {
    match e {
        LoadError::PathDiscovery(s) => vec![s],
        LoadError::Io(err, p) => vec![format!("cannot read {}: {}", p.display(), err)],
        LoadError::Syntax(c) => vec![c.to_string()],
        LoadError::Semantic(v) => v.iter().map(|c| c.to_string()).collect(),
    }
}

#[derive(Debug)]
pub enum UpdateError {
    UnknownVariable(String),
    Invalid(Vec<ConfigError>),
}

impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateError::UnknownVariable(name) => write!(f, "variable \"{}\" is not defined", name),
            UpdateError::Invalid(errs) => {
                let joined = errs
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");
                write!(f, "{}", joined)
            }
        }
    }
}

#[derive(Clone)]
pub struct Store {
    config: ParsedConfig,
    resolved: ResolvedConfig,
    warnings: Vec<ConfigError>,
}

impl Store {
    pub fn new(config: ParsedConfig) -> Result<Store, Vec<ConfigError>> {
        let (resolved, msgs) = resolve(&config);
        match resolved {
            Some(r) => Ok(Store {
                config,
                resolved: r,
                warnings: msgs,
            }),
            None => Err(msgs),
        }
    }

    pub fn resolved(&self) -> &ResolvedConfig {
        &self.resolved
    }

    pub fn warnings(&self) -> &[ConfigError] {
        &self.warnings
    }

    pub fn pulls(&self) -> &indexmap::IndexMap<String, crate::config::types::PullDecl> {
        &self.config.pulls
    }

    pub fn var_value(&self, name: &str) -> Option<&VarValue> {
        self.config.vars.get(name).map(|d| &d.value)
    }

    pub fn refresh(&mut self) {
        let (resolved, msgs) = resolve(&self.config);
        if let Some(r) = resolved {
            self.resolved = r;
            self.warnings = msgs;
        }
    }

    pub fn reload(&mut self, path: &std::path::Path) -> Result<Vec<String>, Vec<String>> {
        let load = match load_from_path(path) {
            Ok(ok) => ok,
            Err(e) => return Err(load_error_lines(e)),
        };
        match Store::new(load.config) {
            Ok(candidate) => {
                let serrs = candidate.validate_surfaces();
                if !serrs.is_empty() {
                    return Err(serrs.iter().map(|e| e.to_string()).collect());
                }
                let mut warns: Vec<String> = load.warnings.iter().map(|w| w.to_string()).collect();
                warns.extend(candidate.warnings.iter().map(|w| w.to_string()));
                *self = candidate;
                Ok(warns)
            }
            Err(errs) => Err(errs.iter().map(|e| e.to_string()).collect()),
        }
    }

    pub fn validate_surfaces(&self) -> Vec<ConfigError> {
        let mut errs = Vec::new();
        let mut err = |kind, span: &crate::config::types::Span, message: String| {
            errs.push(ConfigError {
                kind,
                span: span.clone(),
                message,
                severity: Severity::Error,
            });
        };
        for (name, w) in &self.resolved.widgets {
            let ew = w.w.map(|v| v as u32).unwrap_or(0);
            let eh = w.h.map(|v| v as u32).unwrap_or(0);
            let Some(a) = w.anchor else { continue };
            let (l, r) = (a.contains(Anchor::Left), a.contains(Anchor::Right));
            let (t, b) = (a.contains(Anchor::Top), a.contains(Anchor::Bottom));
            if ew == 0 && (l ^ r) {
                err(
                    ConfigErrorKind::MissingSizeAnchor,
                    &w.span,
                    format!(
                        "widget \"{name}\": anchor uses only one of left/right but w is not set; set w, or anchor t/b only to span the full width"
                    ),
                );
            }
            if ew > 0 && l && r {
                err(
                    ConfigErrorKind::AnchorConflict,
                    &w.span,
                    format!(
                        "widget \"{name}\": w is set but anchor spans both left and right; remove w or one of the anchors"
                    ),
                );
            }
            if eh == 0 && (t ^ b) {
                err(
                    ConfigErrorKind::MissingSizeAnchor,
                    &w.span,
                    format!(
                        "widget \"{name}\": anchor uses only one of top/bottom but h is not set; set h, or anchor l/r only to span the full height"
                    ),
                );
            }
            if eh > 0 && t && b {
                err(
                    ConfigErrorKind::AnchorConflict,
                    &w.span,
                    format!(
                        "widget \"{name}\": h is set but anchor spans both top and bottom; remove h or one of the anchors"
                    ),
                );
            }
        }
        errs
    }

    pub fn update(&mut self, name: &str, raw_value: &str) -> Result<(), UpdateError> {
        if !self.config.vars.contains_key(name) {
            return Err(UpdateError::UnknownVariable(name.to_string()));
        }
        let parsed = match self.config.vars.get(name).map(|d| &d.value) {
            Some(VarValue::Bool(b)) if raw_value == "toggle" => VarValue::Bool(!b),
            _ => parse_value(raw_value),
        };
        let mut candidate = self.config.clone();
        if let Some(decl) = candidate.vars.get_mut(name) {
            decl.value = parsed;
        }
        let (resolved, msgs) = resolve(&candidate);
        match resolved {
            Some(r) => {
                self.config = candidate;
                self.resolved = r;
                self.warnings = msgs;
                Ok(())
            }
            None => Err(UpdateError::Invalid(msgs)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_str;
    use crate::config::resolved::ResolvedConfig;
    use crate::config::{ConfigError, ConfigErrorKind};

    fn store_from(kdl: &str) -> Result<Store, Vec<ConfigError>> {
        let (cfg, perrs) = parse_str(kdl, "<test>");
        let cfg = cfg.expect("fixture should parse");
        assert!(
            perrs
                .iter()
                .all(|e| e.severity != crate::config::Severity::Error),
            "fixture parse errs: {:?}",
            perrs
        );
        Store::new(cfg)
    }

    #[test]
    fn new_ok_exposes_resolved() {
        let store = store_from("widget bar child=t1\ntext t1 color=ffffff").expect("should build");
        let _: &ResolvedConfig = store.resolved();
        assert!(store.resolved().widgets.contains_key("bar"));
    }

    #[test]
    fn new_ok_collects_warnings() {
        let store = store_from("widget bar child=t1\ntext t1\ntext t2").expect("should build");
        assert!(
            store
                .warnings()
                .iter()
                .any(|w| w.kind == ConfigErrorKind::UnusedElement)
        );
    }

    #[test]
    fn new_err_on_resolution_error() {
        let res = store_from("widget bar child=nope");
        assert!(res.is_err());
        let errs = res.err().unwrap();
        assert!(
            errs.iter()
                .any(|e| e.kind == ConfigErrorKind::UnresolvedReference)
        );
    }

    #[test]
    fn parse_value_int() {
        assert!(matches!(parse_value("5"), VarValue::Int(5)));
        assert!(matches!(parse_value("-3"), VarValue::Int(-3)));
    }
    #[test]
    fn parse_value_float() {
        assert!(matches!(parse_value("2.5"), VarValue::Float(f) if (f - 2.5).abs() < 1e-9));
    }
    #[test]
    fn parse_value_bool() {
        assert!(matches!(parse_value("#true"), VarValue::Bool(true)));
        assert!(matches!(parse_value("true"), VarValue::Bool(true)));
        assert!(matches!(parse_value("#false"), VarValue::Bool(false)));
        assert!(matches!(parse_value("false"), VarValue::Bool(false)));
    }
    #[test]
    fn parse_value_string() {
        assert!(matches!(parse_value("hello"), VarValue::Str(s) if s == "hello"));
        assert!(
            matches!(parse_value("container c1 child=t1"), VarValue::Str(s) if s == "container c1 child=t1")
        );
        assert!(matches!(parse_value("${x/2}"), VarValue::Str(s) if s == "${x/2}"));
    }

    use crate::config::resolved::ResolvedElement;

    #[test]
    fn update_numeric_changes_resolved() {
        let mut store = store_from("var hh=40\nwidget bar h=\"${hh}\" child=t1\ntext t1").unwrap();
        assert_eq!(store.resolved().widgets.get("bar").unwrap().h, Some(40.0));
        store.update("hh", "80").expect("update ok");
        assert_eq!(store.resolved().widgets.get("bar").unwrap().h, Some(80.0));
    }

    #[test]
    fn update_unknown_var_errors() {
        let mut store = store_from("widget bar child=t1\ntext t1").unwrap();
        match store.update("nope", "5") {
            Err(UpdateError::UnknownVariable(n)) => assert_eq!(n, "nope"),
            other => panic!("expected UnknownVariable, got {:?}", other),
        }
        assert!(store.resolved().widgets.contains_key("bar"));
    }

    #[test]
    fn update_invalid_keeps_old_state() {
        let mut store = store_from("var hh=40\nwidget bar h=\"${hh}\" child=t1\ntext t1").unwrap();
        let before = store.resolved().widgets.get("bar").unwrap().h;
        assert_eq!(before, Some(40.0));
        match store.update("hh", "notanumber") {
            Err(UpdateError::Invalid(errs)) => assert!(!errs.is_empty()),
            other => panic!("expected Invalid, got {:?}", other),
        }
        assert_eq!(store.resolved().widgets.get("bar").unwrap().h, Some(40.0));
    }

    #[test]
    fn update_compounds() {
        let mut store =
            store_from("var a=10\nvar b=20\nwidget bar w=\"${a}\" h=\"${b}\" child=t1\ntext t1")
                .unwrap();
        store.update("a", "11").unwrap();
        store.update("b", "22").unwrap();
        let bar = store.resolved().widgets.get("bar").unwrap();
        assert_eq!(bar.w, Some(11.0));
        assert_eq!(bar.h, Some(22.0));
    }

    #[test]
    fn update_struct_var() {
        let mut store =
            store_from("var x=\"container c1 child=t1\"\nwidget bar child=x\ntext t1\ntext t2")
                .unwrap();
        assert!(matches!(
            store
                .resolved()
                .widgets
                .get("bar")
                .unwrap()
                .child
                .as_deref(),
            Some(ResolvedElement::Container(_))
        ));
        store
            .update("x", "container c2 child=t2")
            .expect("valid fragment");
        assert!(matches!(
            store
                .resolved()
                .widgets
                .get("bar")
                .unwrap()
                .child
                .as_deref(),
            Some(ResolvedElement::Container(_))
        ));
        let err = store.update("x", "container c3 child=missing").unwrap_err();
        assert!(matches!(err, UpdateError::Invalid(_)));
    }

    #[test]
    fn update_warning_not_blocking() {
        let mut store =
            store_from("var hh=40\nwidget bar h=\"${hh}\" child=t1\ntext t1\ntext unused").unwrap();
        store
            .update("hh", "50")
            .expect("commit despite unused-element warning");
        assert_eq!(store.resolved().widgets.get("bar").unwrap().h, Some(50.0));
    }

    #[test]
    fn update_error_display() {
        let mut store = store_from("widget bar child=t1\ntext t1").unwrap();
        let err = store.update("nope", "5").unwrap_err();
        let msg = format!("{}", err);
        assert!(
            msg.contains("nope") && msg.contains("not defined"),
            "unexpected display: {}",
            msg
        );

        let mut store2 = store_from("var hh=40\nwidget bar h=\"${hh}\" child=t1\ntext t1").unwrap();
        let err2 = store2.update("hh", "notanumber").unwrap_err();
        let msg2 = format!("{}", err2);
        assert!(!msg2.is_empty(), "invalid display should not be empty");
    }

    #[test]
    fn reload_valid_commits_change() {
        use std::io::Write;
        let mut store =
            store_from("widget bar anchor=\"t | l | r\" h=30 child=t1\ntext t1").unwrap();
        assert_eq!(store.resolved().widgets.get("bar").unwrap().h, Some(30.0));

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("c.kdl");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "widget bar anchor=\"t | l | r\" h=50 child=t1\ntext t1").unwrap();
        drop(f);

        let warns = store.reload(&path).expect("valid reload");
        assert!(warns.is_empty(), "no warnings expected, got {:?}", warns);
        assert_eq!(store.resolved().widgets.get("bar").unwrap().h, Some(50.0));
    }

    #[test]
    fn reload_resolve_error_keeps_state() {
        use std::io::Write;
        let mut store = store_from("widget bar h=30 child=t1\ntext t1").unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("c.kdl");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "widget bar child=missing").unwrap();
        drop(f);

        let errs = store.reload(&path).unwrap_err();
        assert!(!errs.is_empty());
        assert_eq!(store.resolved().widgets.get("bar").unwrap().h, Some(30.0));
    }

    #[test]
    fn reload_syntax_error_keeps_state() {
        use std::io::Write;
        let mut store = store_from("widget bar h=30 child=t1\ntext t1").unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("c.kdl");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "widget bar {{").unwrap();
        drop(f);

        let errs = store.reload(&path).unwrap_err();
        assert!(!errs.is_empty());
        assert_eq!(store.resolved().widgets.get("bar").unwrap().h, Some(30.0));
    }

    #[test]
    fn reload_io_error_keeps_state() {
        let mut store = store_from("widget bar h=30 child=t1\ntext t1").unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does-not-exist.kdl");

        let errs = store.reload(&path).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("cannot read")));
        assert_eq!(store.resolved().widgets.get("bar").unwrap().h, Some(30.0));
    }

    #[test]
    fn validate_surfaces_single_top_anchor_full_width_ok() {
        let s = store_from("widget bar anchor=t h=30 child=t1\ntext t1").unwrap();
        assert!(
            s.validate_surfaces().is_empty(),
            "got {:?}",
            s.validate_surfaces()
        );
    }

    #[test]
    fn validate_surfaces_single_left_without_w_errors() {
        let s = store_from("widget bar anchor=l h=30 child=t1\ntext t1").unwrap();
        let errs = s.validate_surfaces();
        assert!(
            errs.iter()
                .any(|e| e.kind == ConfigErrorKind::MissingSizeAnchor),
            "expected MissingSizeAnchor, got {:?}",
            errs
        );
    }

    #[test]
    fn validate_surfaces_size_with_both_edges_conflicts() {
        let s = store_from("widget bar anchor=\"l | r\" w=500 h=30 child=t1\ntext t1").unwrap();
        let errs = s.validate_surfaces();
        assert!(
            errs.iter()
                .any(|e| e.kind == ConfigErrorKind::AnchorConflict),
            "expected AnchorConflict, got {:?}",
            errs
        );
    }

    #[test]
    fn validate_surfaces_full_width_bar_ok() {
        let s = store_from("widget bar anchor=\"t | l | r\" h=30 child=t1\ntext t1").unwrap();
        assert!(
            s.validate_surfaces().is_empty(),
            "got {:?}",
            s.validate_surfaces()
        );
    }

    #[test]
    fn validate_surfaces_explicit_width_ok() {
        let s = store_from("widget bar anchor=\"t\" w=1920 h=30 child=t1\ntext t1").unwrap();
        assert!(s.validate_surfaces().is_empty());
    }

    #[test]
    fn validate_surfaces_left_panel_full_height_ok() {
        let s = store_from("widget side anchor=l w=300 child=t1\ntext t1").unwrap();
        assert!(
            s.validate_surfaces().is_empty(),
            "got {:?}",
            s.validate_surfaces()
        );
    }

    #[test]
    fn validate_surfaces_full_screen_ok() {
        let s = store_from("widget full anchor=\"t | b | l | r\" child=t1\ntext t1").unwrap();
        assert!(s.validate_surfaces().is_empty());
    }

    #[test]
    fn validate_surfaces_no_anchor_no_size_is_fullscreen_ok() {
        let s = store_from("widget bar child=t1\ntext t1").unwrap();
        assert!(
            s.validate_surfaces().is_empty(),
            "got {:?}",
            s.validate_surfaces()
        );
    }

    #[test]
    fn reload_warning_only_commits_and_returns_warnings() {
        use std::io::Write;
        let mut store =
            store_from("widget bar anchor=\"t | l | r\" h=30 child=t1\ntext t1").unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("c.kdl");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "widget bar anchor=\"t | l | r\" h=30 child=t1\ntext t1\ntext unused"
        )
        .unwrap();
        drop(f);

        let warns = store.reload(&path).expect("commits despite warning");
        assert!(
            warns.iter().any(|w| w.contains("warning")),
            "expected a warning line, got {:?}",
            warns
        );
    }

    #[test]
    fn reload_invalid_surface_keeps_state() {
        use std::io::Write;
        let mut store =
            store_from("widget bar anchor=\"t | l | r\" h=30 child=t1\ntext t1").unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("c.kdl");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "widget bar anchor=l h=30 child=t1\ntext t1").unwrap();
        drop(f);

        let errs = store.reload(&path).unwrap_err();
        assert!(
            errs.iter().any(|e| e.contains("only one of left/right")),
            "got {:?}",
            errs
        );
        assert_eq!(
            store.resolved().widgets.get("bar").unwrap().h,
            Some(30.0),
            "state must be kept"
        );
    }
}
