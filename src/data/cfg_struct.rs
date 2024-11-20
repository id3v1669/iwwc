#[derive(Debug, Clone)]
pub struct Config {
    pub respect_notification_timeout: bool,
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
    pub default_icon_dir: std::path::PathBuf,
    // nfcenter stuff ...
}
impl Config {
    pub fn default() -> Self {
        let nvidia_sucks = crate::data::shared_data::NVIDIA_SUCKS.lock().unwrap();
        Config {
            respect_notification_timeout: true,
            local_expire_timeout: 7,
            max_notifications: 5,
            height: 85, // to be min 65
            width: 400, // to be min 300
            vertical_margin: 10,
            horizontal_margin: 10,
            border_radius: iced::border::radius(if *nvidia_sucks { 0.0 } else { 10.0 }),
            border_color: iced::Color::parse("#BA5816").unwrap(),
            border_width: 2.0,
            primary_text_color: iced::Color::parse("#e7d4a2").unwrap(),
            secondary_text_color: iced::Color::parse("#e7d4a2").unwrap(),
            background_color: iced::Color::parse("#282828").unwrap(),
            respect_notification_icon: false,
            default_icon_dir: std::path::PathBuf::from(
                std::env::var("HOME").unwrap() + "/.config/rs-nc",
            ),
        }
    }
}
