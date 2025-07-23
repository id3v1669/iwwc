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

// Final
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
    pub element: String,
    pub right_click_action: Option<String>,
}

// Semi final, maybe add max width and height
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContainerWpraper {
    pub id: String,
    pub child: String,
    pub padding: Option<Vec<f32>>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub align_x: Option<String>,
    pub align_y: Option<String>,
    pub style: Option<String>,
}

// Final
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContainerButtonStyleWpraper {
    pub id: String,
    pub text_color: Option<String>,
    pub background_color: Option<String>,
    pub border: Option<String>,
    pub shadow: Option<String>,
    pub snap: Option<bool>, // TODO: figure out what it does
}

// Final
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BorderWraper {
    pub id: String,
    pub width: Option<f32>,
    pub radius: Option<(f32, f32, f32, f32)>,
    pub color: Option<String>,
}

// Final
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShadowWraper {
    pub id: String,
    pub color: Option<String>,
    pub offset: Option<(f32, f32)>,
    pub blur_radius: Option<f32>,
}

// Final
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RowWraper {
    pub id: String,
    pub children: Vec<String>,
    pub spacing: Option<f32>,
    pub padding: Option<Vec<f32>>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub allinment: Option<String>,
}

// Final
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ColumnWraper {
    pub id: String,
    pub children: Vec<String>,
    pub spacing: Option<f32>,
    pub padding: Option<Vec<f32>>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub allinment: Option<String>,
}

// Final
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ButtonWpraper {
    pub id: String,
    pub text: String,
    pub action_id: String, //figure out right click when added to iced lib
    pub width: Option<String>,
    pub height: Option<String>,
    pub padding: Option<Vec<f32>>,
    pub style: Option<String>,
}

// Unfinished
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TextWraper {
    pub id: String,
    pub text: String,
    pub width: Option<String>,
    pub height: Option<String>,
    pub allignment: Option<String>,
    pub font_size: Option<u32>,
    pub font_color: Option<String>,
    //pub font_family: Option<String>, // TODO
}

// Unfinished, more objects and actions to add
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
    pub text: Option<Vec<TextWraper>>,
}
