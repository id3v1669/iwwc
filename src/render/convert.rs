pub fn font(name: &str) -> iced::Font {
    iced::Font::with_name(Box::leak(name.to_string().into_boxed_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_uses_given_name() {
        let f = font("Sans");
        assert!(matches!(f.family, iced::font::Family::Name("Sans")));
    }
}
