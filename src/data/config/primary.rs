use crate::data::config::wraper::ConfigRead;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use iced::platform_specific::shell::commands::layer_surface::{
    Anchor, KeyboardInteractivity, Layer,
};

#[derive(Debug, Clone)]
pub struct WidgetWindow {
    //pub size: Option<(Option<u32>, Option<u32>)>,
    pub size: (u32, u32),
    pub layer: Layer,
    pub anchor: Anchor,
    pub exclusive_zone: i32,
    //pub output: IcedOutput,
    //pub margin: (i32, i32, i32, i32),
    pub keyboard_interactivity: KeyboardInteractivity,
    pub namespace: String,
    pub timeout: Option<i32>, // 0 or None for no timeout
    pub element: String,      // ID of WidgetElement
}

impl WidgetWindow {
    pub fn from_wrapper(window_raw: crate::data::config::wraper::WidgetWindowWraper) -> Self {
        Self {
            //size: Some((Some(window_raw.size.0), Some(window_raw.size.1))),
            size: window_raw.size,
            layer: match window_raw.layer {
                Some(value) => match value.to_lowercase().as_str() {
                    "background" => Layer::Background,
                    "bottom" => Layer::Bottom,
                    "top" => Layer::Top,
                    "overlay" => Layer::Overlay,
                    _ => {
                        log::warn!("Unknown layer: {value}, defaulting to Overlay");
                        Layer::Overlay
                    }
                },
                None => Layer::Overlay,
            },
            anchor: match window_raw.location {
                Some(loc) => {
                    let mut anchor = Anchor::empty();
                    for location_str in loc.iter() {
                        anchor |= match location_str.to_lowercase().as_str() {
                            "top" => Anchor::TOP,
                            "bottom" => Anchor::BOTTOM,
                            "left" => Anchor::LEFT,
                            "right" => Anchor::RIGHT,
                            _ => {
                                log::warn!("Unknown anchor: {location_str}, ignoring");
                                continue;
                            }
                        };
                    }
                    if anchor == Anchor::empty() {
                        log::warn!("No valid anchors found, defaulting to Top|Right");
                        Anchor::TOP | Anchor::RIGHT
                    } else {
                        anchor
                    }
                }
                None => Anchor::TOP | Anchor::RIGHT,
            },
            exclusive_zone: window_raw.exclusive.unwrap_or(0),
            //margin: window_raw.margin,
            keyboard_interactivity: match window_raw.focus {
                Some(value) => match value.to_lowercase().as_str() {
                    "none" => KeyboardInteractivity::None,
                    "exclusive" => KeyboardInteractivity::Exclusive,
                    "on_demand" | "ondemand" => KeyboardInteractivity::OnDemand,
                    _ => {
                        log::warn!(
                            "Unknown keyboard interactivity: {value}, defaulting to None"
                        );
                        KeyboardInteractivity::None
                    }
                },
                None => KeyboardInteractivity::None,
            },
            namespace: window_raw.name,
            timeout: window_raw.timeout,
            element: window_raw.element,
        }
    }
}

// Semi final, maybe add max width and height
#[derive(Debug, Clone)]
pub struct Container {
    pub child: String, // ID of child
    pub padding: iced::Padding,
    pub width: iced::Length,
    pub height: iced::Length,
    pub align_x: iced::alignment::Horizontal,
    pub align_y: iced::alignment::Vertical,
    pub style: iced::widget::container::Style,
}

// Semi final, maybe add max width and height
impl Container {
    pub fn from_wrapper(
        w: crate::data::config::wraper::ContainerWpraper,
        container_button_styles: &std::collections::HashMap<String, ContainerButtonStyle>,
    ) -> Self {
        Self {
            child: w.child,
            padding: crate::data::config::helper::parse_padding(w.padding),
            width: crate::data::config::helper::parse_length(w.width, "width"),
            height: crate::data::config::helper::parse_length(w.height, "height"),
            align_x: crate::data::config::helper::allinment_horizontal(w.align_x),
            align_y: crate::data::config::helper::allinment_vertical(w.align_y),
            style: Self::create_style(w.style, container_button_styles),
        }
    }

    fn create_style(
        style_id: Option<String>,
        container_button_styles: &std::collections::HashMap<String, ContainerButtonStyle>,
    ) -> iced::widget::container::Style {
        let container_style = style_id
            .clone()
            .as_ref()
            .and_then(|id| container_button_styles.get(id));

        if container_style.is_none() {
            log::debug!("style with id {style_id:?} not found, using default style");
            return iced::widget::container::Style::default();
        }

        iced::widget::container::Style {
            text_color: container_style.and_then(|s| s.text_color).or_else(|| {
                log::debug!("No text color found, defaulting to white");
                Some(iced::Color::WHITE)
            }),
            background: container_style
                .and_then(|s| s.background_color)
                .or_else(|| {
                    log::debug!("No background color found, defaulting to black");
                    Some(iced::Background::Color(iced::Color::BLACK))
                }),
            border: container_style.map(|s| s.border).unwrap_or({
                log::debug!(
                    "No border style found for style {style_id:?}, defaulting to no border"
                );
                iced::Border {
                    color: iced::Color::TRANSPARENT,
                    width: 0.0,
                    radius: iced::border::Radius::from(0.0),
                }
            }),
            shadow: container_style.map(|s| s.shadow).unwrap_or({
                log::debug!(
                    "No shadow style found for style {style_id:?}, defaulting to no shadow"
                );
                iced::Shadow {
                    color: iced::Color::TRANSPARENT,
                    offset: iced::Vector { x: 0.0, y: 0.0 },
                    blur_radius: 0.0,
                }
            }),
            // FIXME: missing `icon_color`
            ..Default::default()
        }
    }
}

// Final
#[derive(Debug, Clone)]
pub struct Row {
    pub children: Vec<String>, // IDs of child elements
    pub spacing: f32,
    pub padding: iced::Padding,
    pub width: iced::Length,
    pub height: iced::Length,
    pub allinment: iced::alignment::Vertical,
}

// Final
impl Row {
    pub fn from_wrapper(r: crate::data::config::wraper::RowWraper) -> Self {
        Self {
            children: r.children,
            spacing: r.spacing.unwrap_or(0.3), // Figure out wierd behavior with transparent borders
            padding: crate::data::config::helper::parse_padding(r.padding),
            width: crate::data::config::helper::parse_length(r.width, "width"),
            height: crate::data::config::helper::parse_length(r.height, "height"),
            allinment: crate::data::config::helper::allinment_vertical(r.allinment),
        }
    }
}

// Final
#[derive(Debug, Clone)]
pub struct Column {
    pub children: Vec<String>, // IDs of child elements
    pub spacing: f32,
    pub padding: iced::Padding,
    pub width: iced::Length,
    pub height: iced::Length,
    pub allinment: iced::alignment::Horizontal,
}

// Final
impl Column {
    pub fn from_wrapper(c: crate::data::config::wraper::ColumnWraper) -> Self {
        Self {
            children: c.children,
            spacing: c.spacing.unwrap_or(3.0),
            padding: crate::data::config::helper::parse_padding(c.padding),
            width: crate::data::config::helper::parse_length(c.width, "width"),
            height: crate::data::config::helper::parse_length(c.height, "height"),
            allinment: crate::data::config::helper::allinment_horizontal(c.allinment),
        }
    }
}

// Final
#[derive(Debug, Clone)]
pub struct Button {
    pub text: String,
    pub on_click: Option<String>,
    pub width: iced::Length,
    pub height: iced::Length,
    pub padding: iced::Padding,
    pub style_active: iced::widget::button::Style,
    pub style_hover: iced::widget::button::Style,
    pub style_pressed: iced::widget::button::Style,
}

// Final
impl Button {
    pub fn from_wrapper(
        b: crate::data::config::wraper::ButtonWpraper,
        container_button_styles: &std::collections::HashMap<String, ContainerButtonStyle>,
    ) -> Self {
        Self {
            text: b.text,
            on_click: b.on_click,
            width: crate::data::config::helper::parse_length(b.width, "width"),
            height: crate::data::config::helper::parse_length(b.height, "height"),
            padding: crate::data::config::helper::parse_padding(b.padding),
            style_active: Self::create_style(b.style_active, container_button_styles),
            style_hover: Self::create_style(b.style_hover, container_button_styles),
            style_pressed: Self::create_style(b.style_pressed, container_button_styles),
        }
    }

    fn create_style(
        style_id: Option<String>,
        container_button_styles: &std::collections::HashMap<String, ContainerButtonStyle>,
    ) -> iced::widget::button::Style {
        let button_style = style_id
            .clone()
            .as_ref()
            .and_then(|id| container_button_styles.get(id));

        iced::widget::button::Style {
            text_color: button_style.and_then(|s| s.text_color).unwrap_or_else(|| {
                log::debug!("No text color found, defaulting to white");
                iced::Color::WHITE
            }),
            background: button_style.and_then(|s| s.background_color).or_else(|| {
                log::debug!("No background color found, defaulting to black");
                Some(iced::Background::Color(iced::Color::BLACK))
            }),
            border: button_style.map(|s| s.border).unwrap_or({
                log::debug!(
                    "No border style found for style {style_id:?}, defaulting to no border"
                );
                iced::Border {
                    color: iced::Color::TRANSPARENT,
                    width: 0.0,
                    radius: iced::border::Radius::from(0.0),
                }
            }),
            shadow: button_style.map(|s| s.shadow).unwrap_or({
                log::debug!(
                    "No shadow style found for style {style_id:?}, defaulting to no shadow"
                );
                iced::Shadow {
                    color: iced::Color::TRANSPARENT,
                    offset: iced::Vector { x: 0.0, y: 0.0 },
                    blur_radius: 0.0,
                }
            }),
            // FIXME: missing `border_color`, `border_radius`, `border_width` and 1 other field
            ..Default::default()
        }
    }
}

// Final
#[derive(Debug, Clone)]
pub struct ContainerButtonStyle {
    pub text_color: Option<iced::Color>,
    pub background_color: Option<iced::Background>,
    pub border: iced::Border,
    pub shadow: iced::Shadow,
}

// Final
impl ContainerButtonStyle {
    pub fn from_wrapper(
        s: crate::data::config::wraper::ContainerButtonStyleWpraper,
        border_styles: &std::collections::HashMap<String, iced::Border>,
        shadow_styles: &std::collections::HashMap<String, iced::Shadow>,
    ) -> Self {
        Self {
            text_color: s
                .text_color
                .and_then(|color_str| iced::Color::parse(&color_str)),
            background_color: s.background_color.map(|color_str| {
                iced::Background::Color(iced::Color::parse(&color_str).unwrap_or_else(|| {
                    log::warn!("Invalid background color, defaulting to white");
                    iced::Color::WHITE
                }))
            }),
            border: s
                .border
                .and_then(|border_id| border_styles.get(&border_id).cloned())
                .unwrap_or(iced::Border {
                    color: iced::Color::BLACK,
                    width: 0.0,
                    radius: iced::border::Radius::from(0.0),
                }),
            shadow: s
                .shadow
                .and_then(|shadow_id| shadow_styles.get(&shadow_id).cloned())
                .unwrap_or(iced::Shadow {
                    color: iced::Color::BLACK,
                    offset: iced::Vector { x: 0.0, y: 0.0 },
                    blur_radius: 0.0,
                }),
        }
    }
}

//Unfinished, add font family & for text create function to handle widget variables
#[derive(Debug, Clone)]
pub struct Text {
    pub text: String,
    pub width: iced::Length,
    pub height: iced::Length,
    pub align_x: iced::alignment::Horizontal,
    pub align_y: iced::alignment::Vertical,
    pub font_size: iced::Pixels,
    pub color: iced::Color,
    pub font: iced::Font,
}

//Unfinished, add font family
impl Text {
    pub fn from_wrapper(
        t: crate::data::config::wraper::TextWraper,
        f: &std::collections::HashMap<String, iced::Font>,
    ) -> Self {
        Self {
            text: t.text,
            width: crate::data::config::helper::parse_length(t.width, "width"),
            height: crate::data::config::helper::parse_length(t.height, "height"),
            align_x: crate::data::config::helper::allinment_horizontal(t.align_x),
            align_y: crate::data::config::helper::allinment_vertical(t.align_y),
            font_size: iced::Pixels(t.font_size.unwrap_or(16.0)),
            color: t
                .font_color
                .and_then(|c| iced::Color::parse(&c))
                .unwrap_or_else(|| {
                    //TODO: separate invalid color from missing color
                    log::warn!("Invalid font color, defaulting to white");
                    iced::Color::WHITE
                }),
            font: f
                .get(&t.font_id.unwrap_or_default())
                .cloned()
                .unwrap_or_else(|| {
                    log::warn!("Font not found, defaulting to system font");
                    iced::Font::default()
                }),
        }
    }
}

// Unfinished
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Global {
    pub antialiasing: Option<bool>,
    pub output: Option<String>,
}

// Unknown, maybe transform to a widget
#[derive(Debug, Clone)]
pub struct NotificationConfig {
    pub enable: bool,
    pub location: iced::platform_specific::shell::commands::subsurface::Anchor,
    pub local_expire_timeout: i32, //in seconds
    pub max_notifications: i32,    //0 for unlimited
    pub height: u32,
    pub width: u32,
    pub vertical_margin: i32,
    pub horizontal_margin: i32,
    pub border_radius: iced::border::Radius,
    pub border_color: iced::Color,
    pub border_width: f32,
    pub primary_text_color: iced::Color,
    pub secondary_text_color: iced::Color,
    pub background_color: iced::Color,
    pub respect_notification_icon: bool,
    pub respect_notification_timeout: bool,
}

// Unknown, maybe transform to a widget
impl NotificationConfig {
    pub fn from_wrapper(notif_cfg: crate::data::config::wraper::NotificationConfig) -> Self {
        Self {
            enable: notif_cfg.enable.unwrap_or(false),
            location: crate::data::config::helper::parse_anchor(notif_cfg.location),
            local_expire_timeout: notif_cfg.local_expire_timeout.unwrap_or(7),
            max_notifications: notif_cfg.max_notifications.unwrap_or(5),
            height: notif_cfg.height.unwrap_or(85),
            width: notif_cfg.width.unwrap_or(400),
            vertical_margin: notif_cfg.vertical_margin.unwrap_or(10),
            horizontal_margin: notif_cfg.horizontal_margin.unwrap_or(10),
            border_radius: notif_cfg
                .border_radius
                .map(|r| iced::border::Radius {
                    top_left: r.0,
                    top_right: r.1,
                    bottom_right: r.2,
                    bottom_left: r.3,
                })
                .unwrap_or_else(|| iced::border::radius(10.0)),
            border_color: notif_cfg
                .border_color
                .and_then(|c| iced::Color::parse(&c))
                .unwrap_or_else(|| iced::Color::parse("#BA5816").unwrap()),
            border_width: notif_cfg.border_width.unwrap_or(2.0),
            primary_text_color: notif_cfg
                .primary_text_color
                .and_then(|c| iced::Color::parse(&c))
                .unwrap_or_else(|| iced::Color::parse("#e7d4a2").unwrap()),
            secondary_text_color: notif_cfg
                .secondary_text_color
                .and_then(|c| iced::Color::parse(&c))
                .unwrap_or_else(|| iced::Color::parse("#e7d4a2").unwrap()),
            background_color: notif_cfg
                .background_color
                .and_then(|c| iced::Color::parse(&c))
                .unwrap_or_else(|| iced::Color::parse("#282828").unwrap()),
            respect_notification_icon: notif_cfg.respect_notification_icon.unwrap_or(false),
            respect_notification_timeout: notif_cfg.respect_notification_timeout.unwrap_or(false),
        }
    }
}
// Unknown, maybe transform to a widget
impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enable: true,
            location: iced::platform_specific::shell::commands::subsurface::Anchor::TOP
                | iced::platform_specific::shell::commands::subsurface::Anchor::RIGHT,
            local_expire_timeout: 7,
            max_notifications: 5,
            height: 85, // to be min 65
            width: 400, // to be min 300
            vertical_margin: 10,
            horizontal_margin: 10,
            border_radius: iced::border::radius(10.0),
            border_color: iced::Color::parse("#BA5816").unwrap(),
            border_width: 2.0,
            primary_text_color: iced::Color::parse("#e7d4a2").unwrap(),
            secondary_text_color: iced::Color::parse("#e7d4a2").unwrap(),
            background_color: iced::Color::parse("#282828").unwrap(),
            respect_notification_icon: false,
            respect_notification_timeout: false,
        }
    }
}

// Unfinished, add actions and maybe more element types
#[derive(Default, Debug, Clone)]
pub struct Config {
    pub global: Global,
    pub notifications: NotificationConfig,
    pub widgets: std::collections::HashMap<String, WidgetWindow>,
    pub containers: std::collections::HashMap<String, Container>,
    pub rows: std::collections::HashMap<String, Row>,
    pub columns: std::collections::HashMap<String, Column>,
    pub buttons: std::collections::HashMap<String, Button>,
    pub texts: std::collections::HashMap<String, Text>,
    //pub subscriptions: Vec<String>, // TODO: implement subscriptions based on text elements
}

impl Config {
    pub fn load(path: Option<PathBuf>) -> Self {
        let path_buf = if let Some(p) = path {
            p
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(format!("{home}/.config/iwwc/config.json"))
        };
        let config: ConfigRead = if !path_buf.exists() {
            log::warn!("Config file not found at: {path_buf:#?}");
            log::warn!("Falling back to default config");
            return Self::default();
            //TODO: implement propper default file creation
        } else {
            match fs::read_to_string(path_buf) {
                Ok(content) => serde_json::from_str(&content).unwrap(),
                Err(e) => {
                    eprintln!("Error reading config file: {e}");
                    log::error!("Falling back to default config");
                    return Self::default();
                }
            }
        };

        Self::from(config)
    }

    pub fn from(cfg: ConfigRead) -> Self {
        let shadow_styles = Self::build_shadow_styles(&cfg);
        let border_styles = Self::build_border_styles(&cfg);
        let fonts = Self::build_fonts(&cfg);
        let container_button_styles =
            Self::build_container_button_styles(&cfg, &border_styles, &shadow_styles);

        Self {
            global: cfg.global.unwrap_or_default(),
            notifications: cfg
                .notifications
                .map(NotificationConfig::from_wrapper)
                .unwrap_or_default(),
            widgets: cfg
                .widgets
                .unwrap_or_else(|| {
                    log::debug!("No widgets found in config");
                    vec![]
                })
                .into_iter()
                .map(|w| (w.name.clone(), WidgetWindow::from_wrapper(w)))
                .collect(),
            containers: cfg
                .containers
                .unwrap_or_else(|| {
                    log::debug!("No containers found in config");
                    vec![]
                })
                .into_iter()
                .map(|c| {
                    (
                        c.id.clone(),
                        Container::from_wrapper(c, &container_button_styles),
                    )
                })
                .collect(),
            rows: cfg
                .rows
                .unwrap_or_else(|| {
                    log::debug!("No rows found in config");
                    vec![]
                })
                .into_iter()
                .map(|r| (r.id.clone(), Row::from_wrapper(r)))
                .collect(),
            columns: cfg
                .columns
                .unwrap_or_else(|| {
                    log::debug!("No columns found in config");
                    vec![]
                })
                .into_iter()
                .map(|c| (c.id.clone(), Column::from_wrapper(c)))
                .collect(),
            buttons: cfg
                .buttons
                .unwrap_or_else(|| {
                    log::debug!("No buttons found in config");
                    vec![]
                })
                .into_iter()
                .map(|b| {
                    (
                        b.id.clone(),
                        Button::from_wrapper(b, &container_button_styles),
                    )
                })
                .collect(),
            texts: cfg
                .texts
                .unwrap_or_else(|| {
                    log::debug!("No texts found in config");
                    vec![]
                })
                .into_iter()
                .map(|t| (t.id.clone(), Text::from_wrapper(t, &fonts)))
                .collect(),
            //variables: vec![],
            //subscriptions: vec![], // TODO: implement subscriptions based on text elements
        }
    }

    fn build_shadow_styles(cfg: &ConfigRead) -> std::collections::HashMap<String, iced::Shadow> {
        cfg.shadow_styles
            .clone()
            .unwrap_or_else(|| {
                log::debug!("No shadow styles found in config");
                vec![]
            })
            .into_iter()
            .map(|s| {
                (
                    s.id.clone(),
                    iced::Shadow {
                        color: iced::Color::parse(
                            &s.color.unwrap_or_else(|| "#000000".to_string()),
                        )
                        .unwrap_or_else(|| {
                            log::warn!("Invalid shadow color defaulting to black");
                            iced::Color::BLACK
                        }),
                        offset: iced::Vector {
                            x: s.offset.map_or(0.0, |o| o.0),
                            y: s.offset.map_or(0.0, |o| o.1),
                        },
                        blur_radius: s.blur_radius.unwrap_or(5.0),
                    },
                )
            })
            .collect()
    }

    fn build_border_styles(cfg: &ConfigRead) -> std::collections::HashMap<String, iced::Border> {
        cfg.border_styles
            .clone()
            .unwrap_or_else(|| {
                log::debug!("No border styles found in config");
                vec![]
            })
            .into_iter()
            .map(|s| {
                (
                    s.id.clone(),
                    iced::Border {
                        color: iced::Color::parse(
                            &s.color.unwrap_or_else(|| "#000000".to_string()),
                        )
                        .unwrap_or_else(|| {
                            log::warn!("Invalid border color defaulting to black");
                            iced::Color::BLACK
                        }),
                        width: s.width.unwrap_or(1.0),
                        radius: {
                            let rad = s.radius.unwrap_or((0.0, 0.0, 0.0, 0.0));
                            iced::border::Radius {
                                top_left: rad.0,
                                top_right: rad.1,
                                bottom_right: rad.2,
                                bottom_left: rad.3,
                            }
                        },
                    },
                )
            })
            .collect()
    }

    fn build_container_button_styles(
        cfg: &ConfigRead,
        border_styles: &std::collections::HashMap<String, iced::Border>,
        shadow_styles: &std::collections::HashMap<String, iced::Shadow>,
    ) -> std::collections::HashMap<String, ContainerButtonStyle> {
        cfg.container_button_styles
            .clone()
            .unwrap_or_else(|| {
                log::warn!("No container styles found in config");
                vec![]
            })
            .into_iter()
            .map(|s| {
                (
                    s.id.clone(),
                    ContainerButtonStyle::from_wrapper(s, border_styles, shadow_styles),
                )
            })
            .collect()
    }

    fn build_fonts(cfg: &ConfigRead) -> std::collections::HashMap<String, iced::Font> {
        cfg.fonts
            .clone()
            .unwrap_or_else(|| {
                log::warn!("No fonts found in config");
                vec![]
            })
            .into_iter()
            .map(|f| {
                let family_static =
                    crate::data::config::helper::get_font_name_static(f.family.clone());
                (
                    f.id.clone(),
                    iced::Font {
                        family: iced::font::Family::Name(family_static),
                        weight: crate::data::config::helper::parse_font_weight(f.weight),
                        stretch: crate::data::config::helper::parse_font_stretch(f.stretch),
                        style: crate::data::config::helper::parse_font_style(f.style),
                    },
                )
            })
            .collect()
    }
}
