use crate::config::resolved::ResolvedNotificationSettings;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, NewLayerShellSettings};

pub fn margin_for_slot(s: &ResolvedNotificationSettings, slot: usize) -> (i32, i32, i32, i32) {
    let base = s.margin;
    let step = (s.height + s.gap) as i32 * slot as i32;
    let (mut top, right, mut bottom, left) =
        (base.0 as i32, base.1 as i32, base.2 as i32, base.3 as i32);
    if s.anchor.contains(Anchor::Bottom) && !s.anchor.contains(Anchor::Top) {
        bottom += step;
    } else {
        top += step;
    }
    (top, right, bottom, left)
}

pub fn notif_layer_settings(
    s: &ResolvedNotificationSettings,
    slot: usize,
) -> NewLayerShellSettings {
    NewLayerShellSettings {
        size: Some((s.width as u32, s.height as u32)),
        layer: s.layer,
        anchor: s.anchor,
        exclusive_zone: Some(0),
        margin: Some(margin_for_slot(s, slot)),
        keyboard_interactivity: KeyboardInteractivity::None,
        output_option: s.output.clone(),
        events_transparent: false,
        namespace: Some("iwwc".to_string()),
    }
}
