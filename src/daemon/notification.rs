use crate::config::resolved::ResolvedNotificationSettings;
use crate::config::types::{Anchor as CfgAnchor, Layer as CfgLayer};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};

pub fn margin_for_slot(s: &ResolvedNotificationSettings, slot: usize) -> (i32, i32, i32, i32) {
    let base = s.margin;
    let step = (s.height + s.gap) as i32 * slot as i32;
    let (mut top, right, mut bottom, left) = (
        base.0 as i32,
        base.1 as i32,
        base.2 as i32,
        base.3 as i32,
    );
    if s.anchor.bottom && !s.anchor.top {
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
        layer: to_layer(s.layer),
        anchor: to_anchor(s.anchor),
        exclusive_zone: Some(0),
        margin: Some(margin_for_slot(s, slot)),
        keyboard_interactivity: KeyboardInteractivity::None,
        output_option: OutputOption::LastOutput,
        events_transparent: false,
        namespace: Some("iwwc".to_string()),
    }
}

fn to_layer(l: CfgLayer) -> Layer {
    match l {
        CfgLayer::Background => Layer::Background,
        CfgLayer::Bottom => Layer::Bottom,
        CfgLayer::Overlay => Layer::Overlay,
        CfgLayer::Top => Layer::Top,
    }
}

fn to_anchor(a: CfgAnchor) -> Anchor {
    let mut out = Anchor::empty();
    if a.top {
        out |= Anchor::Top;
    }
    if a.bottom {
        out |= Anchor::Bottom;
    }
    if a.left {
        out |= Anchor::Left;
    }
    if a.right {
        out |= Anchor::Right;
    }
    if out.is_empty() {
        Anchor::Top | Anchor::Right
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::resolved::ResolvedNotificationSettings;
    use crate::config::types::Anchor;

    fn settings(anchor: Anchor) -> ResolvedNotificationSettings {
        let mut s = ResolvedNotificationSettings::default();
        s.height = 100.0;
        s.gap = 10.0;
        s.margin = (12.0, 12.0, 12.0, 12.0);
        s.anchor = anchor;
        s
    }

    #[test]
    fn top_anchor_slot_offsets_top_margin() {
        let s = settings(Anchor {
            top: true,
            bottom: false,
            left: false,
            right: true,
        });
        assert_eq!(margin_for_slot(&s, 0), (12, 12, 12, 12));
        assert_eq!(margin_for_slot(&s, 1), (12 + 110, 12, 12, 12));
        assert_eq!(margin_for_slot(&s, 2), (12 + 220, 12, 12, 12));
    }

    #[test]
    fn bottom_anchor_slot_offsets_bottom_margin() {
        let s = settings(Anchor {
            top: false,
            bottom: true,
            left: false,
            right: true,
        });
        assert_eq!(margin_for_slot(&s, 1), (12, 12, 12 + 110, 12));
    }
}
