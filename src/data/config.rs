#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WidgetWindow {
    pub width: u32,
    pub height: u32,
    pub name: String,
    pub location: iced_layershell::reexport::Anchor,
    pub exclusive: bool,
    pub layer: iced_layershell::reexport::Layer,
}

//pub struct WidgetElement

#[derive(Debug, Clone)]
pub struct Global {
    pub antialiasing: bool,
    //pub output: String,
}

impl Default for Global {
    fn default() -> Self {
        Self {
            antialiasing: true,
            //output: "DP-1".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
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
    #[allow(dead_code)]
    pub widgets: Vec<WidgetWindow>,
}
