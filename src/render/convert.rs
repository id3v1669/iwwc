use crate::config::types::{AlignX, AlignY, ColAlign, RowAlign};
use iced::alignment;

pub fn align_x(a: AlignX) -> alignment::Horizontal {
    match a {
        AlignX::Left => alignment::Horizontal::Left,
        AlignX::Center => alignment::Horizontal::Center,
        AlignX::Right => alignment::Horizontal::Right,
    }
}

pub fn align_y(a: AlignY) -> alignment::Vertical {
    match a {
        AlignY::Top => alignment::Vertical::Top,
        AlignY::Center => alignment::Vertical::Center,
        AlignY::Bottom => alignment::Vertical::Bottom,
    }
}

pub fn row_align(a: RowAlign) -> alignment::Vertical {
    match a {
        RowAlign::Top => alignment::Vertical::Top,
        RowAlign::Center => alignment::Vertical::Center,
        RowAlign::Bottom => alignment::Vertical::Bottom,
    }
}

pub fn col_align(a: ColAlign) -> alignment::Horizontal {
    match a {
        ColAlign::Left => alignment::Horizontal::Left,
        ColAlign::Center => alignment::Horizontal::Center,
        ColAlign::Right => alignment::Horizontal::Right,
    }
}

pub fn text_align_x(a: AlignX) -> iced::widget::text::Alignment {
    iced::widget::text::Alignment::from(align_x(a))
}

pub fn font(name: &str) -> iced::Font {
    iced::Font::with_name(Box::leak(name.to_string().into_boxed_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_maps_u8_to_f32() {
        let c = color(Color {
            r: 255,
            g: 128,
            b: 0,
            a: 255,
        });
        assert!((c.r - 1.0).abs() < 1e-6);
        assert!((c.g - 128.0 / 255.0).abs() < 1e-6);
        assert!((c.b - 0.0).abs() < 1e-6);
        assert!((c.a - 1.0).abs() < 1e-6);
        let t = color(Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        });
        assert!((t.a - 0.0).abs() < 1e-6);
    }

    #[test]
    fn align_maps() {
        assert!(matches!(
            align_x(AlignX::Center),
            alignment::Horizontal::Center
        ));
        assert!(matches!(
            align_y(AlignY::Bottom),
            alignment::Vertical::Bottom
        ));
        assert!(matches!(row_align(RowAlign::Top), alignment::Vertical::Top));
        assert!(matches!(
            col_align(ColAlign::Right),
            alignment::Horizontal::Right
        ));
    }

    #[test]
    fn text_align_x_from_horizontal() {
        let a = text_align_x(AlignX::Center);
        let expected = iced::widget::text::Alignment::from(alignment::Horizontal::Center);
        assert_eq!(a, expected);
    }

    #[test]
    fn font_uses_given_name() {
        let f = font("Sans");
        assert!(matches!(f.family, iced::font::Family::Name("Sans")));
    }
}
