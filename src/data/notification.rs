#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub app_name: String,
    pub app_icon: String,
    pub replaces_id: u32,
    pub summary: String,
    pub body: String,
    pub actions: Vec<String>,
    pub expire_timeout: i32,
    pub notification_id: u32,
    pub desktop_entry: String,
}

#[derive(Debug, Clone)]
pub enum NotificationAction {
    ActionClose { notification_id: u32, reason: u32 },
    ActionInvoked { notification_id: u32 },
    Notify { notification: Notification },
    Close { notification_id: u32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreCalc {
    pub general_padding: f32,
    pub font_size_summary: f32,
    pub font_size_body: f32,
    pub image_size: f32,
    pub text_summary_paddings: iced::Padding,
    pub text_body_paddings: iced::Padding,
    pub text_paddings_block: iced::Padding,
}

impl PreCalc {
    pub fn generate() -> Self {
        let config = crate::data::shared::CONFIG.lock().unwrap();
        // precalculation of font sizes to avoid recalculating them every frame(view) update
        // TODO: ajust formulas here after figuring out propper grid layout and proportions
        Self {
            general_padding: std::cmp::min(
                (config.height as f32 * 0.15) as u16,
                (config.width as f32 * 0.03) as u16,
            ) as f32,
            font_size_summary: std::cmp::min(
                (config.height as f32 * 0.24) as u16,
                (((config.width as f32) - ((config.height as f32) * 0.65)) * 0.06) as u16,
            ) as f32,
            font_size_body: std::cmp::min(
                (config.height as f32 * 0.17) as u16,
                (((config.width as f32) - ((config.height as f32) * 0.65)) * 0.042) as u16,
            ) as f32,
            image_size: (config.height as f32) * 0.65,
            text_summary_paddings: iced::Padding {
                top: 0.0,
                bottom: 0.0,
                left: (config.height as f32 * 0.05) + (config.height as f32 * 0.01),
                right: 0.0,
            },
            text_body_paddings: iced::Padding {
                top: 0.0,
                bottom: 0.0,
                left: config.height as f32 * 0.05,
                right: 0.0,
            },
            text_paddings_block: iced::Padding {
                top: config.height as f32 * 0.1,
                bottom: config.height as f32 * 0.1,
                left: config.height as f32 * 0.15,
                right: 0.0,
            },
        }
    }
}
