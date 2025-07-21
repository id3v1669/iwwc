use crate::data::config::wraper::ConfigRead;
use iced_layershell::reexport::{Anchor, Layer, NewLayerShellSettings, OutputOption};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf};

#[derive(Debug, Clone)]
pub struct WidgetWindow {
    pub id: String, // ID of the widget window
    pub settings: iced_layershell::reexport::NewLayerShellSettings,
    pub timeout: Option<i32>, // 0 or None for no timeout
    pub element: String,      // ID of WidgetElement
}

impl WidgetWindow {
    pub fn from_wrapper(w: crate::data::config::wraper::WidgetWindowWraper) -> Self {
        Self {
            id: w.name.clone(),
            settings: iced_layershell::reexport::NewLayerShellSettings {
                size: Some(w.size),
                layer: match w.layer {
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
                anchor: match w.location {
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
                                    iced_layershell::reexport::Anchor::empty()
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
                exclusive_zone: w.exclusive,
                margin: w.margin,
                keyboard_interactivity: match w.focus {
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
                output_option: iced_layershell::reexport::OutputOption::LastOutput,
                events_transparent: w.events_transparent.unwrap_or(false),
                namespace: Some(w.name),
            },
            timeout: w.timeout,
            element: w.element,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Container {
    pub id: String,
    pub child: String, // ID of child
    pub width: iced::Length,
    pub height: iced::Length,
    pub style: iced::widget::container::Style,
    //other parameters to be added later
    // alignment, padding, size, center(location)
}

impl Container {
    pub fn from_wrapper(
        w: crate::data::config::wraper::ContainerWpraper,
        container_button_styles: &[ContainerButtonStyle],
    ) -> Self {
        Self {
            id: w.id,
            child: w.child,
            width: Self::parse_length(w.width, "width"),
            height: Self::parse_length(w.height, "height"),
            style: Self::create_style(w.style, container_button_styles),
        }
    }

    fn parse_length(length: Option<String>, dimension: &str) -> iced::Length {
        length.map_or(iced::Length::Fill, |s| match s.to_lowercase().as_str() {
            "fill" => iced::Length::Fill,
            "shrink" => iced::Length::Shrink,
            _ => {
                let size: f32 = match s.parse::<f32>() {
                    Ok(size) => size,
                    Err(_) => {
                        log::warn!("Invalid {dimension} value: {s}, defaulting to Fill");
                        return iced::Length::Fill;
                    }
                };
                iced::Length::Fixed(size)
            }
        })
    }

    fn create_style(
        style_id: Option<String>,
        container_button_styles: &[ContainerButtonStyle],
    ) -> iced::widget::container::Style {
        let container_style =
            style_id.and_then(|id| container_button_styles.iter().find(|s| s.id == id));

        iced::widget::container::Style {
            text_color: container_style.and_then(|s| s.text_color),
            background: container_style.and_then(|s| s.background_color.clone()),
            border: container_style
                .map(|s| s.border.clone())
                .unwrap_or(iced::Border {
                    color: iced::Color::BLACK,
                    width: 0.0,
                    radius: iced::border::Radius::from(0.0),
                }),
            shadow: container_style
                .map(|s| s.shadow.clone())
                .unwrap_or(iced::Shadow {
                    color: iced::Color::BLACK,
                    offset: iced::Vector { x: 0.0, y: 0.0 },
                    blur_radius: 0.0,
                }),
            snap: container_style.map(|s| s.snap).unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Row {
    pub id: String,
    pub children: Vec<String>, // IDs of child elements
    pub allinment: iced::alignment::Vertical,
}

impl Row {
    pub fn from_wrapper(r: crate::data::config::wraper::RowWraper) -> Self {
        Self {
            id: r.id,
            children: r.children,
            allinment: r
                .allinment
                .map(|a| match a.to_lowercase().as_str() {
                    "top" => iced::alignment::Vertical::Top,
                    "center" => iced::alignment::Vertical::Center,
                    "bottom" => iced::alignment::Vertical::Bottom,
                    _ => {
                        log::warn!("Unknown vertical alignment: {a}, defaulting to Center");
                        iced::alignment::Vertical::Center
                    }
                })
                .unwrap_or(iced::alignment::Vertical::Center),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Column {
    pub id: String,
    pub children: Vec<String>, // IDs of child elements
    pub allinment: iced::alignment::Horizontal,
}

impl Column {
    pub fn from_wrapper(c: crate::data::config::wraper::ColumnWraper) -> Self {
        Self {
            id: c.id,
            children: c.children,
            allinment: c
                .allinment
                .map(|a| match a.to_lowercase().as_str() {
                    "left" => iced::alignment::Horizontal::Left,
                    "center" => iced::alignment::Horizontal::Center,
                    "right" => iced::alignment::Horizontal::Right,
                    _ => {
                        log::warn!("Unknown horizontal alignment: {a}, defaulting to Center");
                        iced::alignment::Horizontal::Center
                    }
                })
                .unwrap_or(iced::alignment::Horizontal::Center),
        }
    }
}

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

impl Button {
    pub fn from_wrapper(
        b: crate::data::config::wraper::ButtonWpraper,
        container_button_styles: &[ContainerButtonStyle],
    ) -> Self {
        Self {
            id: b.id,
            text: b.text,
            action_id: b.action_id,
            width: Self::parse_length(b.width, "width"),
            height: Self::parse_length(b.height, "height"),
            padding: b
                .padding
                .map(|p| iced::Padding {
                    top: p.0,
                    right: p.1,
                    bottom: p.2,
                    left: p.3,
                })
                .unwrap_or(iced::Padding::new(5.0)),
            style: Self::create_style(b.style, container_button_styles),
        }
    }

    fn parse_length(length: Option<String>, dimension: &str) -> iced::Length {
        length.map_or(iced::Length::Shrink, |s| match s.to_lowercase().as_str() {
            "fill" => iced::Length::Fill,
            "shrink" => iced::Length::Shrink,
            _ => {
                let size: f32 = match s.parse::<f32>() {
                    Ok(size) => size,
                    Err(_) => {
                        log::warn!("Invalid button {dimension}: {s}, defaulting to Shrink");
                        return iced::Length::Shrink;
                    }
                };
                iced::Length::Fixed(size)
            }
        })
    }

    fn create_style(
        style_id: Option<String>,
        container_button_styles: &[ContainerButtonStyle],
    ) -> iced::widget::button::Style {
        let button_style =
            style_id.and_then(|id| container_button_styles.iter().find(|s| s.id == id));

        iced::widget::button::Style {
            text_color: button_style
                .and_then(|s| s.text_color)
                .unwrap_or(iced::Color::BLACK),
            background: button_style.and_then(|s| s.background_color.clone()),
            border: button_style
                .map(|s| s.border.clone())
                .unwrap_or(iced::Border {
                    color: iced::Color::BLACK,
                    width: 0.0,
                    radius: iced::border::Radius::from(0.0),
                }),
            shadow: button_style
                .map(|s| s.shadow.clone())
                .unwrap_or(iced::Shadow {
                    color: iced::Color::BLACK,
                    offset: iced::Vector { x: 0.0, y: 0.0 },
                    blur_radius: 0.0,
                }),
            snap: button_style.map(|s| s.snap).unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContainerButtonStyle {
    pub id: String,
    pub text_color: Option<iced::Color>,
    pub background_color: Option<iced::Background>,
    pub border: iced::Border,
    pub shadow: iced::Shadow,
    pub snap: bool,
}

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Global {
    pub antialiasing: bool,
}

impl Default for Global {
    fn default() -> Self {
        Self { antialiasing: true }
    }
}

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

impl NotificationConfig {
    pub fn from_wrapper(notif_cfg: crate::data::config::wraper::NotificationConfig) -> Self {
        Self {
            enable: notif_cfg.enable.unwrap_or(true),
            location: Self::parse_anchor(notif_cfg.location),
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

    fn parse_anchor(locations: Option<Vec<String>>) -> iced_layershell::reexport::Anchor {
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
                anchor = iced_layershell::reexport::Anchor::Top
                    | iced_layershell::reexport::Anchor::Right;
            }
        }
        anchor
    }
}

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
            return Self::default();
            // if let Some(parent) = path_buf.parent() {
            //     if !parent.exists() {
            //         if let Err(e) = fs::create_dir_all(parent) {
            //             log::error!("Error creating config directory: {e}");
            //             log::error!("Falling back to default config");
            //             return Self::default();
            //         }
            //     }
            // }
            // match fs::File::create(&path_buf) {
            //     Ok(mut file) => {
            //         let cfg = ConfigRead::default();
            //         log::debug!("Created default aplin config at: {path_buf:#?}");
            //         let _ = file.write_all(serde_yml::to_string(&cfg).unwrap().as_bytes());
            //         return Self::default();
            //     }
            //     Err(e) => {
            //         log::error!("Error creating config file: {e}");
            //         log::error!("Falling back to default config");
            //         return Self::default();
            //     }
            // };
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
        // First, build the style maps
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
                            x: s.offset_x.unwrap_or(0.0),
                            y: s.offset_y.unwrap_or(0.0),
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