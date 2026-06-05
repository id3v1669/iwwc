use iced_layershell::reexport::{Anchor, Layer, OutputOption};
use iced::Color;
use iced::alignment::{Horizontal,Vertical};
use iced::advanced::text::Alignment as TextAlignment;
use std::str::FromStr;

pub fn parse_color(input: &str) -> Option<Color> {
    if input == "transparent" {
        return Some(Color::TRANSPARENT);
    }
    Color::from_str(input).ok()
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
    let mut a = Anchor::empty();
    for tok in tokens {
        match tok {
            "t" | "top" => a |= Anchor::Top,
            "b" | "bottom" => a |= Anchor::Bottom,
            "l" | "left" => a |= Anchor::Left,
            "r" | "right" => a |= Anchor::Right,
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

pub fn parse_align_x(s: &str) -> Option<Horizontal> {
    match s {
        "l" | "left" => Some(Horizontal::Left),
        "c" | "center" => Some(Horizontal::Center),
        "r" | "right" => Some(Horizontal::Right),
        _ => None,
    }
}

pub fn parse_align_y(s: &str) -> Option<Vertical> {
    match s {
        "t" | "top" => Some(Vertical::Top),
        "c" | "center" => Some(Vertical::Center),
        "b" | "bottom" => Some(Vertical::Bottom),
        _ => None,
    }
}

pub fn parse_text_align_x(s: &str) -> Option<TextAlignment> {
    match s {
        "l" | "left" => Some(TextAlignment::Left),
        "c" | "center" => Some(TextAlignment::Center),
        "r" | "right" => Some(TextAlignment::Right),
        "j" | "justify" | "justified" => Some(TextAlignment::Justified),
        _ => None,
    }
}

pub fn parse_output(s: &str) -> OutputOption {
    if s == "last" {
        OutputOption::LastOutput
    } else {
        OutputOption::OutputName(s.to_string())
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
