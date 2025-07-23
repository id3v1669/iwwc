use crate::data::config::wraper::ConfigRead;
use iced_layershell::reexport::{Anchor, Layer, NewLayerShellSettings, OutputOption};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf};

// Final
#[derive(Debug, Clone)]
pub struct WidgetWindow {
    pub id: String, // ID of the widget window
    pub settings: iced_layershell::reexport::NewLayerShellSettings,
    pub timeout: Option<i32>, // 0 or None for no timeout
    pub element: String,      // ID of WidgetElement
}

// Semi-final, fix lib for output_option
impl WidgetWindow {
    pub fn from_wrapper(window_raw: crate::data::config::wraper::WidgetWindowWraper) -> Self {
        Self {
            id: window_raw.name.clone(),
            settings: iced_layershell::reexport::NewLayerShellSettings {
                size: Some(window_raw.size),
                layer: match window_raw.layer {
                    Some(value) => match value.to_lowercase().as_str() {
                        "background" => iced_layershell::reexport::Layer::Background,
                        "bottom" => iced_layershell::reexport::Layer::Bottom,
                        "top" => iced_layershell::reexport::Layer::Top,
                        "overlay" => iced_layershell::reexport::Layer::Overlay,
                        _ => {
                            log::warn!("Unknown layer: {value}, defaulting to Overlay");
                            iced_layershell::reexport::Layer::Overlay
                        }
                    },
                    None => iced_layershell::reexport::Layer::Overlay,
                },
                anchor: match window_raw.location {
                    Some(loc) => {
                        let mut anchor = iced_layershell::reexport::Anchor::empty();
                        for location_str in loc.iter() {
                            anchor |= match location_str.to_lowercase().as_str() {
                                "top" => iced_layershell::reexport::Anchor::Top,
                                "bottom" => iced_layershell::reexport::Anchor::Bottom,
                                "left" => iced_layershell::reexport::Anchor::Left,
                                "right" => iced_layershell::reexport::Anchor::Right,
                                _ => {
                                    log::warn!("Unknown anchor: {}, ignoring", location_str);
                                    continue;
                                }
                            };
                        }
                        if anchor == iced_layershell::reexport::Anchor::empty() {
                            log::warn!("No valid anchors found, defaulting to Top|Right");
                            iced_layershell::reexport::Anchor::Top
                                | iced_layershell::reexport::Anchor::Right
                        } else {
                            anchor
                        }
                    }
                    None => {
                        iced_layershell::reexport::Anchor::Top
                            | iced_layershell::reexport::Anchor::Right
                    }
                },
                exclusive_zone: window_raw.exclusive,
                margin: window_raw.margin,
                keyboard_interactivity: match window_raw.focus {
                    Some(value) => match value.to_lowercase().as_str() {
                        "none" => iced_layershell::reexport::KeyboardInteractivity::None,
                        "exclusive" => iced_layershell::reexport::KeyboardInteractivity::Exclusive,
                        "on_demand" | "ondemand" => {
                            iced_layershell::reexport::KeyboardInteractivity::OnDemand
                        }
                        _ => {
                            log::warn!(
                                "Unknown keyboard interactivity: {}, defaulting to None",
                                value
                            );
                            iced_layershell::reexport::KeyboardInteractivity::None
                        }
                    },
                    None => iced_layershell::reexport::KeyboardInteractivity::None,
                },
                output_option: iced_layershell::reexport::OutputOption::LastOutput, // TODO: fix lib
                events_transparent: window_raw.events_transparent.unwrap_or(false),
                namespace: Some(window_raw.name),
            },
            timeout: window_raw.timeout,
            element: window_raw.element,
        }
    }
}

// Semi final, maybe add max width and height
#[derive(Debug, Clone)]
pub struct Container {
    pub id: String,
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
        container_button_styles: &[ContainerButtonStyle],
    ) -> Self {
        Self {
            id: w.id,
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
        container_button_styles: &[ContainerButtonStyle],
    ) -> iced::widget::container::Style {
        let container_style = style_id.clone()
        .and_then(|id| container_button_styles.iter().find(|s| s.id == id));

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
                .and_then(|s| s.background_color.clone())
                .or_else(|| {
                    log::debug!("No background color found, defaulting to black");
                    Some(iced::Background::Color(iced::Color::BLACK))
                }),
            border: container_style.map(|s| s.border.clone()).unwrap_or({
                log::debug!(
                    "No border style found for style {style_id:?}, defaulting to no border"
                );
                iced::Border {
                    color: iced::Color::BLACK,
                    width: 0.0,
                    radius: iced::border::Radius::from(0.0),
                }
            }),
            shadow: container_style.map(|s| s.shadow.clone()).unwrap_or({
                log::debug!(
                    "No shadow style found for style {style_id:?}, defaulting to no shadow"
                );
                iced::Shadow {
                    color: iced::Color::BLACK,
                    offset: iced::Vector { x: 0.0, y: 0.0 },
                    blur_radius: 0.0,
                }
            }),
            snap: container_style.map(|s| s.snap).unwrap_or(false),
        }
    }
}

// Final
#[derive(Debug, Clone)]
pub struct Row {
    pub id: String,
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
            id: r.id,
            children: r.children,
            spacing: r.spacing.unwrap_or(3.0),
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
    pub id: String,
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
            id: c.id,
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
    pub id: String,
    pub text: String,
    pub action_id: String,
    pub width: iced::Length,
    pub height: iced::Length,
    pub padding: iced::Padding,
    pub style: iced::widget::button::Style,
}

// Final
impl Button {
    pub fn from_wrapper(
        b: crate::data::config::wraper::ButtonWpraper,
        container_button_styles: &[ContainerButtonStyle],
    ) -> Self {
        Self {
            id: b.id,
            text: b.text,
            action_id: b.action_id,
            width: crate::data::config::helper::parse_length(b.width, "width"),
            height: crate::data::config::helper::parse_length(b.height, "height"),
            padding: crate::data::config::helper::parse_padding(b.padding),
            style: Self::create_style(b.style, container_button_styles),
        }
    }

    fn create_style(
        style_id: Option<String>,
        container_button_styles: &[ContainerButtonStyle],
    ) -> iced::widget::button::Style {
        let button_style =
            style_id.clone().and_then(|id| container_button_styles.iter().find(|s| s.id == id));

        iced::widget::button::Style {
            text_color: button_style.and_then(|s| s.text_color).unwrap_or_else(|| {
                log::debug!("No text color found, defaulting to white");
                iced::Color::WHITE
            }),
            background: button_style
                .and_then(|s| s.background_color.clone())
                .or_else(|| {
                    log::debug!("No background color found, defaulting to black");
                    Some(iced::Background::Color(iced::Color::BLACK))
                }),
            border: button_style.map(|s| s.border.clone()).unwrap_or({
                log::debug!(
                    "No border style found for style {style_id:?}, defaulting to no border"
                );
                iced::Border {
                    color: iced::Color::BLACK,
                    width: 0.0,
                    radius: iced::border::Radius::from(0.0),
                }
            }),
            shadow: button_style.map(|s| s.shadow.clone()).unwrap_or({
                log::debug!(
                    "No shadow style found for style {style_id:?}, defaulting to no shadow"
                );
                iced::Shadow {
                    color: iced::Color::BLACK,
                    offset: iced::Vector { x: 0.0, y: 0.0 },
                    blur_radius: 0.0,
                }
            }),
            snap: button_style.map(|s| s.snap).unwrap_or(false),
        }
    }
}

// Final
#[derive(Debug, Clone)]
pub struct ContainerButtonStyle {
    pub id: String,
    pub text_color: Option<iced::Color>,
    pub background_color: Option<iced::Background>,
    pub border: iced::Border,
    pub shadow: iced::Shadow,
    pub snap: bool,
}

// Final
impl ContainerButtonStyle {
    pub fn from_wrapper(
        s: crate::data::config::wraper::ContainerButtonStyleWpraper,
        border_styles: &std::collections::HashMap<String, iced::Border>,
        shadow_styles: &std::collections::HashMap<String, iced::Shadow>,
    ) -> Self {
        Self {
            id: s.id,
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
            snap: s.snap.unwrap_or(false),
        }
    }
}

// Unfinished
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Global {
    pub antialiasing: bool,
}

// Unfinished
impl Default for Global {
    fn default() -> Self {
        Self { antialiasing: true }
    }
}

// Unknown, maybe transform to a widget
#[derive(Debug, Clone)]
pub struct NotificationConfig {
    pub enable: bool,
    pub location: iced_layershell::reexport::Anchor,
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
            enable: notif_cfg.enable.unwrap_or(true),
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
            location: iced_layershell::reexport::Anchor::Top
                | iced_layershell::reexport::Anchor::Right,
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
    pub widgets: Vec<WidgetWindow>,
    pub containers: Vec<Container>,
    pub rows: Vec<Row>,
    pub columns: Vec<Column>,
    pub buttons: Vec<Button>,
}

impl Config {
    pub fn load(path: Option<PathBuf>) -> Self {
        let path_buf = if let Some(p) = path {
            p
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(format!("{}/.config/iwwc/config.yml", home))
        };
        let config: ConfigRead = if !path_buf.exists() {
            log::warn!("Config file not found at: {path_buf:#?}");
            log::warn!("Falling back to default config");
            return Self::default();
            //TODO: implement propper default file creation
        } else {
            match fs::read_to_string(path_buf) {
                Ok(content) => serde_yml::from_str(&content).unwrap(),
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
                    log::warn!("No widgets found in config");
                    vec![]
                })
                .into_iter()
                .map(WidgetWindow::from_wrapper)
                .collect(),
            containers: cfg
                .containers
                .unwrap_or_else(|| {
                    log::warn!("No containers found in config");
                    vec![]
                })
                .into_iter()
                .map(|w| Container::from_wrapper(w, &container_button_styles))
                .collect(),
            rows: cfg
                .rows
                .unwrap_or_else(|| {
                    log::warn!("No rows found in config");
                    vec![]
                })
                .into_iter()
                .map(Row::from_wrapper)
                .collect(),
            columns: cfg
                .columns
                .unwrap_or_else(|| {
                    log::warn!("No columns found in config");
                    vec![]
                })
                .into_iter()
                .map(Column::from_wrapper)
                .collect(),
            buttons: cfg
                .buttons
                .unwrap_or_else(|| {
                    log::warn!("No buttons found in config");
                    vec![]
                })
                .into_iter()
                .map(|b| Button::from_wrapper(b, &container_button_styles))
                .collect(),
        }
    }

    fn build_shadow_styles(cfg: &ConfigRead) -> std::collections::HashMap<String, iced::Shadow> {
        cfg.shadow_styles
            .clone()
            .unwrap_or_else(|| {
                log::warn!("No shadow styles found in config");
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
                log::warn!("No border styles found in config");
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
    ) -> Vec<ContainerButtonStyle> {
        cfg.container_styles
            .clone()
            .unwrap_or_else(|| {
                log::warn!("No container styles found in config");
                vec![]
            })
            .into_iter()
            .map(|s| ContainerButtonStyle::from_wrapper(s, border_styles, shadow_styles))
            .collect()
    }
}
