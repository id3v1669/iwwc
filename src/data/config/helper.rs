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

pub fn parse_anchor(locations: Option<Vec<String>>) -> iced::platform_specific::shell::commands::subsurface::Anchor {
    let mut anchor =
        iced::platform_specific::shell::commands::subsurface::Anchor::TOP | iced::platform_specific::shell::commands::subsurface::Anchor::RIGHT;

    if let Some(locations) = locations {
        anchor = iced::platform_specific::shell::commands::subsurface::Anchor::empty();
        for location_str in locations.iter() {
            anchor |= match location_str.to_lowercase().as_str() {
                "top" => iced::platform_specific::shell::commands::subsurface::Anchor::TOP,
                "bottom" => iced::platform_specific::shell::commands::subsurface::Anchor::BOTTOM,
                "left" => iced::platform_specific::shell::commands::subsurface::Anchor::LEFT,
                "right" => iced::platform_specific::shell::commands::subsurface::Anchor::RIGHT,
                _ => {
                    log::warn!("Unknown notification anchor: {location_str}, ignoring");
                    iced::platform_specific::shell::commands::subsurface::Anchor::empty()
                }
            };
        }
        if anchor == iced::platform_specific::shell::commands::subsurface::Anchor::empty() {
            anchor =
                iced::platform_specific::shell::commands::subsurface::Anchor::TOP | iced::platform_specific::shell::commands::subsurface::Anchor::RIGHT;
        }
    }
    anchor
}

pub fn parse_font_weight(weight: Option<String>) -> iced::font::Weight {
    match weight {
        Some(w) => match w.to_lowercase().as_str() {
            "thin" => iced::font::Weight::Thin,
            "extra_light" | "ultra_light" => iced::font::Weight::ExtraLight,
            "light" => iced::font::Weight::Light,
            "normal" | "regular" => iced::font::Weight::Normal,
            "medium" => iced::font::Weight::Medium,
            "semi_bold" | "demi_bold" => iced::font::Weight::Semibold,
            "bold" => iced::font::Weight::Bold,
            "extra_bold" | "ultra_bold" => iced::font::Weight::ExtraBold,
            "black" => iced::font::Weight::Black,
            _ => {
                log::warn!("Unknown font weight: {w}, defaulting to normal");
                iced::font::Weight::Normal
            }
        },
        None => iced::font::Weight::Normal,
    }
}

pub fn parse_font_stretch(stretch: Option<String>) -> iced::font::Stretch {
    match stretch {
        Some(s) => match s.to_lowercase().as_str() {
            "ultra_condensed" => iced::font::Stretch::UltraCondensed,
            "extra_condensed" => iced::font::Stretch::ExtraCondensed,
            "condensed" => iced::font::Stretch::Condensed,
            "semi_condensed" | "demi_condensed" => iced::font::Stretch::SemiCondensed,
            "normal" | "regular" => iced::font::Stretch::Normal,
            "semi_expanded" | "demi_expanded" => iced::font::Stretch::SemiExpanded,
            "expanded" => iced::font::Stretch::Expanded,
            "extra_expanded" => iced::font::Stretch::ExtraExpanded,
            "ultra_expanded" => iced::font::Stretch::UltraExpanded,
            _ => {
                log::warn!("Unknown font stretch: {s}, defaulting to normal");
                iced::font::Stretch::Normal
            }
        },
        None => iced::font::Stretch::Normal,
    }
}

pub fn parse_font_style(style: Option<String>) -> iced::font::Style {
    match style {
        Some(s) => match s.to_lowercase().as_str() {
            "normal" => iced::font::Style::Normal,
            "italic" => iced::font::Style::Italic,
            "oblique" => iced::font::Style::Oblique,
            _ => {
                log::warn!("Unknown font style: {s}, defaulting to normal");
                iced::font::Style::Normal
            }
        },
        None => iced::font::Style::Normal,
    }
}

static FONT_INTERNER: std::sync::OnceLock<
    std::sync::RwLock<std::collections::HashMap<String, &'static str>>,
> = std::sync::OnceLock::new();

pub fn get_font_name_static(name: String) -> &'static str {
    let interner =
        FONT_INTERNER.get_or_init(|| std::sync::RwLock::new(std::collections::HashMap::new()));
    {
        let reader = interner.read().unwrap();
        if let Some(&interned) = reader.get(&name) {
            return interned;
        }
    } // extra scope to release the read lock before acquiring the write lock

    let mut writer = interner.write().unwrap();

    let leaked: &'static str = Box::leak(name.clone().into_boxed_str());
    writer.insert(name, leaked);
    leaked
}
