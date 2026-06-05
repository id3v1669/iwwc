use crate::config::types::{
    AlignX, AlignY, Anchor, ColAlign, Color, Layer, Output, RowAlign,
};

pub fn parse_color(input: &str) -> Option<Color> {
    if input == "transparent" {
        return Some(Color::TRANSPARENT);
    }
    let hex = input.strip_prefix('#').unwrap_or(input);
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color { r, g, b, a: 0xff })
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Color { r, g, b, a })
        }
        _ => None,
    }
}

pub fn parse_length_keyword(s: &str) -> Option<iced::Length> {
    match s {
        "fill" => Some(iced::Length::Fill),
        "shrink" => Some(iced::Length::Shrink),
        _ => None,
    }
}

#[derive(Debug)]
pub enum AnchorError {
    Unknown(String),
}

pub fn parse_anchor(input: &str) -> Result<Anchor, AnchorError> {
    let tokens: Vec<&str> = input
        .split('|')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let mut a = Anchor::default();
    for tok in tokens {
        match tok {
            "t" | "top" => a.top = true,
            "b" | "bottom" => a.bottom = true,
            "l" | "left" => a.left = true,
            "r" | "right" => a.right = true,
            other => return Err(AnchorError::Unknown(other.into())),
        }
    }
    Ok(a)
}

pub fn parse_layer(s: &str) -> Option<Layer> {
    match s {
        "top" => Some(Layer::Top),
        "bottom" => Some(Layer::Bottom),
        "background" => Some(Layer::Background),
        "overlay" => Some(Layer::Overlay),
        _ => None,
    }
}

pub fn parse_align_x(s: &str) -> Option<AlignX> {
    match s {
        "l" | "left" => Some(AlignX::Left),
        "c" | "center" => Some(AlignX::Center),
        "r" | "right" => Some(AlignX::Right),
        _ => None,
    }
}

pub fn parse_align_y(s: &str) -> Option<AlignY> {
    match s {
        "t" | "top" => Some(AlignY::Top),
        "c" | "center" => Some(AlignY::Center),
        "b" | "bottom" => Some(AlignY::Bottom),
        _ => None,
    }
}

pub fn parse_row_align(s: &str) -> Option<RowAlign> {
    match s {
        "t" | "top" => Some(RowAlign::Top),
        "c" | "center" => Some(RowAlign::Center),
        "b" | "bottom" => Some(RowAlign::Bottom),
        _ => None,
    }
}

pub fn parse_col_align(s: &str) -> Option<ColAlign> {
    match s {
        "l" | "left" => Some(ColAlign::Left),
        "c" | "center" => Some(ColAlign::Center),
        "r" | "right" => Some(ColAlign::Right),
        _ => None,
    }
}

pub fn parse_output(s: &str) -> Output {
    if s == "last" {
        Output::Last
    } else {
        Output::Specific(s.to_string())
    }
}

pub fn parse_interval(s: &str) -> Option<std::time::Duration> {
    let s = s.trim();
    let (num, mult_ms): (&str, u64) = if let Some(n) = s.strip_suffix("ms") {
        (n, 1)
    } else if let Some(n) = s.strip_suffix('s') {
        (n, 1000)
    } else if let Some(n) = s.strip_suffix('m') {
        (n, 60_000)
    } else if let Some(n) = s.strip_suffix('h') {
        (n, 3_600_000)
    } else {
        return None;
    };
    let val: u64 = num.trim().parse().ok()?;
    Some(std::time::Duration::from_millis(
        val.saturating_mul(mult_ms),
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn interval_parsing() {
        use super::parse_interval;
        use std::time::Duration;
        assert_eq!(parse_interval("1s"), Some(Duration::from_millis(1000)));
        assert_eq!(parse_interval("500ms"), Some(Duration::from_millis(500)));
        assert_eq!(parse_interval("2m"), Some(Duration::from_millis(120_000)));
        assert_eq!(parse_interval("1h"), Some(Duration::from_millis(3_600_000)));
        assert_eq!(parse_interval(" 250ms "), Some(Duration::from_millis(250)));
        assert_eq!(parse_interval("5"), None);
        assert_eq!(parse_interval("1x"), None);
        assert_eq!(parse_interval("ms"), None);
    }
}
