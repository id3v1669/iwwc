pub fn parse_length(length: Option<String>, dimension: &str) -> iced::Length {
    length.map_or(iced::Length::Fill, |s| match s.to_lowercase().as_str() {
        "fill" => iced::Length::Fill,
        "shrink" => iced::Length::Shrink,
        _ => {
            let size: f32 = match s.parse::<f32>() {
                Ok(size) => size,
                Err(_) => {
                    log::warn!("Invalid {dimension} length value: {s}, defaulting to Fill");
                    return iced::Length::Fill;
                }
            };
            iced::Length::Fixed(size)
        }
    })
}

pub fn parse_padding(padding: Option<Vec<f32>>) -> iced::Padding {
    match padding {
        Some(p) => match p.len() {
            4 => iced::Padding {
                top: p[0],
                right: p[1],
                bottom: p[2],
                left: p[3],
            },
            1 => iced::Padding::new(p[0]),
            _ => {
                log::warn!("Invalid padding length, defaulting to (0.0, 0.0, 0.0, 0.0)");
                iced::Padding::new(0.0)
            }
        },
        None => iced::Padding::new(0.0), // Default padding
    }
}

pub fn allinment_vertical(allinment: Option<String>) -> iced::alignment::Vertical {
    allinment
        .map(|a| match a.to_lowercase().as_str() {
            "top" => iced::alignment::Vertical::Top,
            "center" => iced::alignment::Vertical::Center,
            "bottom" => iced::alignment::Vertical::Bottom,
            _ => {
                log::warn!("Unknown vertical alignment: {a}, defaulting to Center");
                iced::alignment::Vertical::Center
            }
        })
        .unwrap_or(iced::alignment::Vertical::Center)
}

pub fn allinment_horizontal(allinment: Option<String>) -> iced::alignment::Horizontal {
    allinment
        .map(|a| match a.to_lowercase().as_str() {
            "left" => iced::alignment::Horizontal::Left,
            "center" => iced::alignment::Horizontal::Center,
            "right" => iced::alignment::Horizontal::Right,
            _ => {
                log::warn!("Unknown horizontal alignment: {a}, defaulting to Center");
                iced::alignment::Horizontal::Center
            }
        })
        .unwrap_or(iced::alignment::Horizontal::Center)
}

pub fn parse_anchor(locations: Option<Vec<String>>) -> iced_layershell::reexport::Anchor {
    let mut anchor =
        iced_layershell::reexport::Anchor::Top | iced_layershell::reexport::Anchor::Right;

    if let Some(locations) = locations {
        anchor = iced_layershell::reexport::Anchor::empty();
        for location_str in locations.iter() {
            anchor |= match location_str.to_lowercase().as_str() {
                "top" => iced_layershell::reexport::Anchor::Top,
                "bottom" => iced_layershell::reexport::Anchor::Bottom,
                "left" => iced_layershell::reexport::Anchor::Left,
                "right" => iced_layershell::reexport::Anchor::Right,
                _ => {
                    log::warn!("Unknown notification anchor: {}, ignoring", location_str);
                    iced_layershell::reexport::Anchor::empty()
                }
            };
        }
        if anchor == iced_layershell::reexport::Anchor::empty() {
            anchor =
                iced_layershell::reexport::Anchor::Top | iced_layershell::reexport::Anchor::Right;
        }
    }
    anchor
}
