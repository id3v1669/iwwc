use iced::Background;

pub fn background(c: iced::Color) -> Background {
    Background::Color(c)
}

#[cfg(test)]
mod tests {
    use super::*;

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
            radius: Some(iced::border::Radius {
                top_left: 1.0,
                top_right: 2.0,
                bottom_right: 3.0,
                bottom_left: 4.0,
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
