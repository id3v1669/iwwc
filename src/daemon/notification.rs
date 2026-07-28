use crate::config::resolved::ResolvedNotificationSettings;
use iced_layershell::reexport::{
    Anchor, BlurOption, Extent, KeyboardInteractivity, LayerSize, NewLayerShellSettings,
};

pub fn margin_offset(s: &ResolvedNotificationSettings, offset: f32) -> (i32, i32, i32, i32) {
    let base = s.margin;
    let step = offset as i32;
    let (mut top, right, mut bottom, left) =
        (base.0 as i32, base.1 as i32, base.2 as i32, base.3 as i32);
    if s.anchor.contains(Anchor::Bottom) && !s.anchor.contains(Anchor::Top) {
        bottom += step;
    } else {
        top += step;
    }
    (top, right, bottom, left)
}

pub fn notif_size(s: &ResolvedNotificationSettings, height: f32) -> LayerSize {
    LayerSize {
        width: Extent::try_px(s.width as u32).unwrap_or(Extent::Fill),
        height: Extent::try_px(height as u32).unwrap_or(Extent::Fill),
    }
}

pub fn notif_layer_settings(
    s: &ResolvedNotificationSettings,
    height: f32,
    offset: f32,
) -> NewLayerShellSettings {
    NewLayerShellSettings {
        size: notif_size(s, height),
        layer: s.layer,
        anchor: s.anchor,
        exclusive_zone: Some(0),
        margin: Some(margin_offset(s, offset)),
        keyboard_interactivity: KeyboardInteractivity::None,
        blur_option: BlurOption::None,
        output_option: s.output.clone(),
        events_transparent: false,
        namespace: Some("iwwc".to_string()),
    }
}
