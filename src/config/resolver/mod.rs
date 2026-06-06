pub mod coerce;
pub mod elements;
pub mod vars;

use crate::config::resolved::ResolvedConfig;
use crate::config::resolved::ResolvedNotificationSettings;
use crate::config::types::ParsedConfig;
use crate::config::{ConfigError, Severity};
use indexmap::IndexMap;
use std::collections::HashSet;

pub fn resolve(config: &ParsedConfig) -> (Option<ResolvedConfig>, Vec<ConfigError>) {
    let mut errs = Vec::new();
    let mut used: HashSet<String> = HashSet::new();
    let env = vars::resolve_vars(config, &mut used, &mut errs);

    let mut widgets = IndexMap::new();
    for (name, w) in &config.widgets {
        let mut ctx = elements::Ctx {
            config,
            env: &env,
            errs: &mut errs,
            used: &mut used,
        };
        let rw = elements::resolve_widget(name, w, &mut ctx);
        widgets.insert(name.clone(), rw);
    }

    let notification = resolve_notification(config, &env, &mut used, &mut errs);
    let apptray = {
        let mut ctx = elements::Ctx {
            config,
            env: &env,
            errs: &mut errs,
            used: &mut used,
        };
        elements::resolve_apptray_settings(&mut ctx)
    };

    let smart_polls = env.smart_polls();

    let mut all_ids: Vec<(&str, &crate::config::types::Span, bool)> = Vec::new();
    for (id, d) in &config.vars {
        all_ids.push((id, &d.span, true));
    }
    for (id, e) in &config.containers {
        all_ids.push((id, &e.span, false));
    }
    for (id, e) in &config.buttons {
        all_ids.push((id, &e.span, false));
    }
    for (id, e) in &config.rows {
        all_ids.push((id, &e.span, false));
    }
    for (id, e) in &config.columns {
        all_ids.push((id, &e.span, false));
    }
    for (id, e) in &config.texts {
        all_ids.push((id, &e.span, false));
    }
    for (id, e) in &config.styles {
        all_ids.push((id, &e.span, false));
    }
    for (id, e) in &config.borders {
        all_ids.push((id, &e.span, false));
    }
    for (id, e) in &config.shadows {
        all_ids.push((id, &e.span, false));
    }

    for (id, span, is_var) in all_ids {
        if !used.contains(id) {
            let (kind, message) = if is_var {
                (
                    crate::config::ConfigErrorKind::UnusedVariable,
                    format!("variable \"{}\" is defined but never used", id),
                )
            } else {
                (
                    crate::config::ConfigErrorKind::UnusedElement,
                    format!("element \"{}\" is defined but never used", id),
                )
            };
            errs.push(ConfigError {
                kind,
                span: span.clone(),
                message,
                severity: Severity::Warning,
            });
        }
    }
    let icon_theme = config.icon_theme.clone();

    let has_error = errs.iter().any(|e| e.severity == Severity::Error);
    if has_error {
        (None, errs)
    } else {
        (
            Some(ResolvedConfig {
                widgets,
                notification,
                apptray,
                smart_polls,
                icon_theme,
            }),
            errs,
        )
    }
}

fn resolve_notification(
    config: &ParsedConfig,
    env: &vars::FlatEnv,
    used: &mut HashSet<String>,
    errs: &mut Vec<ConfigError>,
) -> ResolvedNotificationSettings {
    let mut out = ResolvedNotificationSettings::default();
    let Some(ns) = &config.notification else {
        return out;
    };
    let mut ctx = elements::Ctx {
        config,
        env,
        errs,
        used,
    };
    let span = &ns.span;
    if let Some(v) = elements::resolve_field(&ns.width, "width", span, coerce::coerce_f32, &mut ctx)
    {
        out.width = v;
    }
    if let Some(v) =
        elements::resolve_field(&ns.height, "height", span, coerce::coerce_f32, &mut ctx)
    {
        out.height = v;
    }
    if let Some(v) = elements::resolve_field(
        &ns.primary_text,
        "primary_text",
        span,
        coerce::coerce_color,
        &mut ctx,
    ) {
        out.primary_text = v;
    }
    if let Some(v) = elements::resolve_field(
        &ns.secondary_text,
        "secondary_text",
        span,
        coerce::coerce_color,
        &mut ctx,
    ) {
        out.secondary_text = v;
    }
    if let Some(v) = elements::resolve_field(&ns.bg, "bg", span, coerce::coerce_color, &mut ctx) {
        out.bg = v;
    }
    out.border = elements::resolve_border_ref(&ns.border, span, &mut ctx);
    if let Some(v) =
        elements::resolve_field(&ns.font, "font", span, coerce::coerce_string, &mut ctx)
    {
        out.font = Some(v);
    }
    if let Some(v) =
        elements::resolve_field(&ns.anchor, "anchor", span, coerce::coerce_anchor, &mut ctx)
    {
        out.anchor = v;
    }
    if let Some(v) =
        elements::resolve_field(&ns.margin, "margin", span, coerce::coerce_margin, &mut ctx)
    {
        out.margin = v;
    }
    if let Some(v) = elements::resolve_field(&ns.gap, "gap", span, coerce::coerce_f32, &mut ctx) {
        out.gap = v;
    }
    if let Some(v) = elements::resolve_field(&ns.max, "max", span, coerce::coerce_f32, &mut ctx) {
        out.max = v.max(0.0) as u32;
    }
    if let Some(v) =
        elements::resolve_field(&ns.timeout, "timeout", span, coerce::coerce_f32, &mut ctx)
    {
        out.timeout_ms = v as i32;
    }
    if let Some(v) =
        elements::resolve_field(&ns.layer, "layer", span, coerce::coerce_layer, &mut ctx)
    {
        out.layer = v;
    }
    if let Some(v) = elements::resolve_field(
        &ns.respect_notification_icon,
        "respect_notification_icon",
        span,
        coerce::coerce_bool,
        &mut ctx,
    ) {
        out.respect_icon = v;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::resolve;
    use crate::config::parse_str;
    use crate::config::resolved::ResolvedElement;
    use crate::config::{ConfigErrorKind, Severity};

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
    fn unused_element_warns() {
        let (rc, errs) = resolve_kdl("widget bar child=t1\ntext t1\ntext t2");
        assert!(rc.is_some(), "should resolve (warning, not error)");
        assert!(
            errs.iter().any(
                |e| e.kind == ConfigErrorKind::UnusedElement && e.severity == Severity::Warning
            ),
            "expected UnusedElement warning, got {:?}",
            errs
        );
    }

    #[test]
    fn unused_variable_warns() {
        let (rc, errs) = resolve_kdl("var unused=5\nwidget bar child=t1\ntext t1");
        assert!(rc.is_some());
        assert!(
            errs.iter()
                .any(|e| e.kind == ConfigErrorKind::UnusedVariable
                    && e.severity == Severity::Warning),
            "expected UnusedVariable warning, got {:?}",
            errs
        );
    }

    #[test]
    fn used_var_no_warning() {
        // A variable used inside a ${...} expression must be marked used and NOT warn.
        let (rc, errs) = resolve_kdl("var hh=40\nwidget bar h=\"${hh}\" child=t1\ntext t1");
        assert!(rc.is_some());
        assert!(
            !errs
                .iter()
                .any(|e| e.kind == ConfigErrorKind::UnusedVariable),
            "hh is used in an expression, should not warn: {:?}",
            errs
        );
    }

    #[test]
    fn transitive_var_usage_no_warning() {
        let (rc, errs) =
            resolve_kdl("var a=1\nvar b=\"${a}\"\nwidget bar h=\"${b}\" child=t1\ntext t1");
        assert!(rc.is_some());
        assert!(
            !errs
                .iter()
                .any(|e| e.kind == ConfigErrorKind::UnusedVariable),
            "neither a nor b should warn (a used by b, b used in h): {:?}",
            errs
        );
    }

    #[test]
    fn text_interpolation_content() {
        let (rc, errs) = resolve_kdl("var x=5\ntext t1 text=\"hi ${x}\"\nwidget bar child=t1");
        assert!(
            errs.iter().all(|e| e.severity != Severity::Error),
            "errs: {:?}",
            errs
        );
        let rc = rc.unwrap();
        match rc.widgets.get("bar").unwrap().child.as_deref() {
            Some(ResolvedElement::Text(t)) => assert_eq!(t.content.as_deref(), Some("hi 5")),
            other => panic!("expected text, got {:?}", other),
        }
    }

    #[test]
    fn apptray_reference_resolves_to_element() {
        let (cfg, _) = parse_str(
            "widget bar child=apptray\napptray icon_size=30 swap_buttons=#true",
            "<t>",
        );
        let (rc, errs) = resolve(&cfg.unwrap());
        assert!(
            errs.iter().all(|e| e.severity != Severity::Error),
            "{:?}",
            errs
        );
        let bar = rc.unwrap();
        let bar = bar.widgets.get("bar").unwrap();
        match bar.child.as_deref() {
            Some(crate::config::resolved::ResolvedElement::Apptray(s)) => {
                assert_eq!(s.icon_size, 30.0);
                assert!(s.swap_buttons);
            }
            other => panic!("expected apptray element, got {:?}", other),
        }
    }

    #[test]
    fn apptray_reference_defaults_without_block() {
        let (cfg, _) = parse_str("widget bar child=apptray", "<t>");
        let (rc, errs) = resolve(&cfg.unwrap());
        assert!(
            errs.iter().all(|e| e.severity != Severity::Error),
            "{:?}",
            errs
        );
        match rc.unwrap().widgets.get("bar").unwrap().child.as_deref() {
            Some(crate::config::resolved::ResolvedElement::Apptray(s)) => {
                assert_eq!(s.icon_size, 22.0)
            }
            other => panic!("expected apptray, got {:?}", other),
        }
    }

    #[test]
    fn smart_ram_resolves_and_is_polled() {
        let (cfg, _) = parse_str(
            "widget bar child=t1\ntext t1 text=\"${iwwc.ram.used / 1073741824}\"",
            "<t>",
        );
        let (rc, errs) = resolve(&cfg.unwrap());
        assert!(
            errs.iter().all(|e| e.severity != Severity::Error),
            "{:?}",
            errs
        );
        let rc = rc.unwrap();
        assert!(rc.smart_polls.iter().any(|(ns, _)| ns == "iwwc.ram"));
    }

    #[test]
    fn bare_namespace_is_unresolved() {
        let (cfg, _) = parse_str("widget bar child=t1\ntext t1 text=\"${iwwc.ram}\"", "<t>");
        let (rc, errs) = resolve(&cfg.unwrap());
        assert!(rc.is_none());
        assert!(errs.iter().any(|e| e.severity == Severity::Error));
    }

    #[test]
    fn no_smart_use_means_no_polls() {
        let (cfg, _) = parse_str("widget bar child=t1\ntext t1 text=\"hi\"", "<t>");
        let (rc, _) = resolve(&cfg.unwrap());
        assert!(rc.unwrap().smart_polls.is_empty());
    }

    #[test]
    fn notification_defaults_and_override() {
        let (rc, errs) = {
            let (cfg, _) = parse_str("widget bar child=t1\ntext t1", "<t>");
            resolve(&cfg.unwrap())
        };
        assert!(errs.iter().all(|e| e.severity != Severity::Error));
        let n = &rc.unwrap().notification;
        assert_eq!(n.width, 400.0);
        assert_eq!(n.max, 5);
        assert_eq!(n.timeout_ms, 5000);

        let (cfg, _) = parse_str(
            "widget bar child=t1\ntext t1\nnotification width=300 max=3 timeout=2000 bg=000000 border=nb\nborder nb w=2",
            "<t>",
        );
        let (rc, errs) = resolve(&cfg.unwrap());
        assert!(
            errs.iter().all(|e| e.severity != Severity::Error),
            "{:?}",
            errs
        );
        let n = &rc.unwrap().notification;
        assert_eq!(n.width, 300.0);
        assert_eq!(n.max, 3);
        assert_eq!(n.timeout_ms, 2000);
        assert_eq!(n.bg, iced::Color::BLACK);
        assert_eq!(n.border.unwrap().width, 2.0);
    }
}
