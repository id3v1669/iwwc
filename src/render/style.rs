use crate::config::resolved::{ResolvedBorder, ResolvedShadow, ResolvedStyle};
use crate::config::types::Color;
use crate::render::convert;
use iced::widget::{button, container};
use iced::{Background, Border, Color as IcedColor, Shadow, Vector};

pub fn background(c: Color) -> Background {
    Background::Color(convert::color(c))
}

pub fn border(b: &ResolvedBorder) -> Border {
    Border {
        color: b
            .color
            .map(convert::color)
            .unwrap_or(IcedColor::TRANSPARENT),
        width: b.w.unwrap_or(0.0),
        radius: b
            .radius
            .map(|e| iced::border::Radius {
                top_left: e.top,
                top_right: e.right,
                bottom_right: e.bottom,
                bottom_left: e.left,
            })
            .unwrap_or_default(),
    }
}

pub fn shadow(s: &ResolvedShadow) -> Shadow {
    Shadow {
        color: s
            .color
            .map(convert::color)
            .unwrap_or(IcedColor::TRANSPARENT),
        offset: s.offset.map(|(x, y)| Vector { x, y }).unwrap_or_default(),
        blur_radius: s.blur_radius.unwrap_or(0.0),
    }
}

pub fn container_style(s: &ResolvedStyle) -> container::Style {
    container::Style {
        text_color: s.text.map(convert::color),
        background: s.bg.map(background),
        border: s.border.as_ref().map(border).unwrap_or_default(),
        shadow: s.shadow.as_ref().map(shadow).unwrap_or_default(),
        snap: s.snap.unwrap_or_default(),
    }
}

pub fn button_style(s: &ResolvedStyle) -> button::Style {
    button::Style {
        background: s.bg.map(background),
        text_color: s.text.map(convert::color).unwrap_or(IcedColor::BLACK),
        border: s.border.as_ref().map(border).unwrap_or_default(),
        shadow: s.shadow.as_ref().map(shadow).unwrap_or_default(),
        snap: s.snap.unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::Edges;

    fn empty_style() -> ResolvedStyle {
        ResolvedStyle {
            text: None,
            bg: None,
            border: None,
            shadow: None,
            snap: None,
        }
    }

    #[test]
    fn border_defaults_when_none() {
        let b = border(&ResolvedBorder {
            color: None,
            w: None,
            radius: None,
        });
        assert_eq!(b.color, IcedColor::TRANSPARENT);
        assert_eq!(b.width, 0.0);
    }

    #[test]
    fn border_maps_radius_corners() {
        let b = border(&ResolvedBorder {
            color: Some(Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            }),
            w: Some(2.0),
            radius: Some(Edges {
                top: 1.0,
                right: 2.0,
                bottom: 3.0,
                left: 4.0,
            }),
        });
        assert_eq!(b.width, 2.0);
        assert_eq!(b.radius.top_left, 1.0);
        assert_eq!(b.radius.top_right, 2.0);
        assert_eq!(b.radius.bottom_right, 3.0);
        assert_eq!(b.radius.bottom_left, 4.0);
    }

    #[test]
    fn shadow_maps_offset() {
        let s = shadow(&ResolvedShadow {
            color: Some(Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            }),
            offset: Some((2.0, 3.0)),
            blur_radius: Some(5.0),
        });
        assert_eq!(s.offset, Vector { x: 2.0, y: 3.0 });
        assert_eq!(s.blur_radius, 5.0);
    }

    #[test]
    fn container_style_none_fields() {
        let cs = container_style(&empty_style());
        assert!(cs.text_color.is_none());
        assert!(cs.background.is_none());
    }

    #[test]
    fn container_style_some_bg() {
        let mut s = empty_style();
        s.bg = Some(Color {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        });
        let cs = container_style(&s);
        assert!(matches!(cs.background, Some(Background::Color(_))));
    }
}
