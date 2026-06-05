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
    fn font_uses_given_name() {
        let f = font("Sans");
        assert!(matches!(f.family, iced::font::Family::Name("Sans")));
    }
}
