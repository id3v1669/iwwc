use crate::data::config::primary::{Container, Global};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotificationConfig {
    pub enable: Option<bool>,
    pub location: Option<Vec<String>>,
    pub local_expire_timeout: Option<i32>,
    pub max_notifications: Option<i32>,
    pub height: Option<u32>,
    pub width: Option<u32>,
    pub vertical_margin: Option<i32>,
    pub horizontal_margin: Option<i32>,
    pub border_radius: Option<(f32, f32, f32, f32)>,
    pub border_color: Option<String>,
    pub border_width: Option<f32>,
    pub primary_text_color: Option<String>,
    pub secondary_text_color: Option<String>,
    pub background_color: Option<String>,
    pub respect_notification_icon: Option<bool>,
    pub respect_notification_timeout: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WidgetWindowWraper {
    pub size: (u32, u32),
    pub layer: Option<String>,
    pub location: Option<Vec<String>>,
    pub exclusive: Option<i32>,
    pub margin: Option<(i32, i32, i32, i32)>,
    pub focus: Option<String>,
    pub events_transparent: Option<bool>, // TODO: fugure out what it does in NewLayerShell
    pub name: String,
    pub timeout: Option<i32>, // 0 or None for no timeout
    pub element: String,      // ID of WidgetElement
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContainerWpraper {
    pub id: String,
    pub child: String, // ID of child
    pub width: Option<String>,
    pub height: Option<String>,
    pub style: Option<String>, // ID of style
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ButtonWpraper {
    pub id: String,
    pub text: String,
    pub action_id: String,
    pub width: Option<String>,
    pub height: Option<String>,
    pub padding: Option<(f32, f32, f32, f32)>,
    pub style: Option<String>, // ID of style
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContainerButtonStyleWpraper {
    pub id: String,
    pub text_color: Option<String>,
    pub background_color: Option<String>,
    pub border: Option<String>, // ID of BorderWraper
    pub shadow: Option<String>, // ID of ShadowWraper
    pub snap: Option<bool>,     // TODO: figure out what it does
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BorderWraper {
    pub id: String,
    pub width: Option<f32>,
    pub radius: Option<(f32, f32, f32, f32)>,
    pub color: Option<String>, // hex color code
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShadowWraper {
    pub id: String,
    pub color: Option<String>,
    pub offset_x: Option<f32>,
    pub offset_y: Option<f32>,
    pub blur_radius: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RowWraper {
    pub id: String,
    pub children: Vec<String>, // IDs of child elements
    pub allinment: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ColumnWraper {
    pub id: String,
    pub children: Vec<String>, // IDs of child elements
    pub allinment: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigRead {
    pub global: Option<Global>,
    pub notifications: Option<NotificationConfig>,
    pub widgets: Option<Vec<WidgetWindowWraper>>,
    pub containers: Option<Vec<ContainerWpraper>>,
    pub rows: Option<Vec<RowWraper>>,
    pub columns: Option<Vec<ColumnWraper>>,
    pub buttons: Option<Vec<ButtonWpraper>>,
    pub container_styles: Option<Vec<ContainerButtonStyleWpraper>>,
    pub border_styles: Option<Vec<BorderWraper>>,
    pub shadow_styles: Option<Vec<ShadowWraper>>,
}
