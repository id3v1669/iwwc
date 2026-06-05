use crate::config::resolved::ResolvedWidget;
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings,
};

pub fn layer_settings_for(w: &ResolvedWidget) -> NewLayerShellSettings {
    let width = w.w.map(|v| v as u32).unwrap_or(0);
    let height = w.h.map(|v| v as u32).unwrap_or(0);
    NewLayerShellSettings {
        size: Some((width, height)),
        layer: w.layer.unwrap_or(Layer::Top),
        anchor: w.anchor.unwrap_or(Anchor::Top | Anchor::Left),
        exclusive_zone: Some(exclusive_zone(w)),
        margin: w
            .margin
            .map(|(t, r, b, l)| (t as i32, r as i32, b as i32, l as i32)),
        keyboard_interactivity: match w.keyboard {
            Some(true) => KeyboardInteractivity::Exclusive,
            _ => KeyboardInteractivity::None,
        },
        output_option: w.output.clone(),
        events_transparent: w.transparent.unwrap_or(false),
        namespace: Some("iwwc".to_string()),
    }
}

fn exclusive_zone(w: &ResolvedWidget) -> i32 {
    if w.exclusive != Some(true) {
        return 0;
    }
    let h = w.h.map(|v| v as i32).unwrap_or(0);
    let width = w.w.map(|v| v as i32).unwrap_or(0);
    match w.anchor {
        Some(a) => {
            let vertical = a.contains(Anchor::Top) ^ a.contains(Anchor::Bottom);
            let horizontal = a.contains(Anchor::Left) ^ a.contains(Anchor::Right);
            match (vertical, horizontal) {
                (true, false) => h,
                (false, true) => width,
                _ => 0,
            }
        }
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_str;
    use crate::config::resolved::ResolvedWidget;
    use crate::config::resolver::resolve;

    fn widget(kdl: &str, name: &str) -> ResolvedWidget {
        let (cfg, _) = parse_str(kdl, "<t>");
        let cfg = cfg.expect("parse");
        let (rc, errs) = resolve(&cfg);
        assert!(
            errs.iter()
                .all(|e| e.severity != crate::config::Severity::Error),
            "{:?}",
            errs
        );
        rc.expect("resolve")
            .widgets
            .get(name)
            .cloned()
            .expect("widget present")
    }

    #[test]
    fn top_bar_exclusive_zone_is_height() {
        let w = widget(
            "widget bar layer=top anchor=\"t\" h=30 w=1920 exclusive=#true child=t1\ntext t1",
            "bar",
        );
        let s = layer_settings_for(&w);
        assert_eq!(s.size, Some((1920, 30)));
        assert_eq!(s.exclusive_zone, Some(30));
        assert!(matches!(s.layer, iced_layershell::reexport::Layer::Top));
    }

    #[test]
    fn corner_anchor_reserves_nothing() {
        let w = widget(
            "widget corner anchor=\"t | l\" h=30 w=200 exclusive=#true child=t1\ntext t1",
            "corner",
        );
        assert_eq!(layer_settings_for(&w).exclusive_zone, Some(0));
    }

    #[test]
    fn left_panel_exclusive_zone_is_width() {
        let w = widget(
            "widget side layer=top anchor=\"l\" w=300 h=1080 exclusive=#true child=t1\ntext t1",
            "side",
        );
        let s = layer_settings_for(&w);
        assert_eq!(s.exclusive_zone, Some(300));
    }

    #[test]
    fn non_exclusive_zone_zero() {
        let w = widget(
            "widget bar anchor=\"t\" h=30 w=100 exclusive=#false child=t1\ntext t1",
            "bar",
        );
        assert_eq!(layer_settings_for(&w).exclusive_zone, Some(0));
    }

    #[test]
    fn output_and_transparent_and_keyboard() {
        let w = widget(
            "widget bar anchor=\"t\" h=30 w=100 output=\"HDMI-A-1\" transparent=#true keyboard=#true child=t1\ntext t1",
            "bar",
        );
        let s = layer_settings_for(&w);
        assert!(
            matches!(s.output_option, iced_layershell::reexport::OutputOption::OutputName(ref n) if n == "HDMI-A-1")
        );
        assert!(s.events_transparent);
        assert!(matches!(
            s.keyboard_interactivity,
            iced_layershell::reexport::KeyboardInteractivity::Exclusive
        ));
        assert_eq!(s.namespace.as_deref(), Some("iwwc"));
    }

    #[test]
    fn output_last_default() {
        let w = widget(
            "widget bar anchor=\"t\" h=30 w=100 child=t1\ntext t1",
            "bar",
        );
        assert!(matches!(
            layer_settings_for(&w).output_option,
            iced_layershell::reexport::OutputOption::LastOutput
        ));
    }
}
