#[derive(Debug, Clone)]
pub struct Config {
  pub respect_notification_timeout: bool,
  pub local_expire_timeout: i32, //in seconds
  pub max_notification: i32, //0 for unlimited
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
  // nfcenter stuff ... 
}
impl Config {
  pub fn default() -> Self {
    Config {
      respect_notification_timeout: true,
      local_expire_timeout: 5,
      max_notification: 0,
      height: 100,
      width: 400,
      vertical_margin: 10,
      horizontal_margin: 10,
      border_radius: iced::border::radius(10),
      border_color: iced::Color::parse("#ff0000").unwrap(),
      border_width: 2.0,
      primary_text_color: iced::Color::parse("#ffffff").unwrap(),
      secondary_text_color: iced::Color::parse("#ff00ff").unwrap(),
      background_color: iced::Color::parse("#000000").unwrap(),
    }
  }
}

pub struct NfCenter {
  pub test: String,
}