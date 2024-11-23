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
        }
    }
    pub fn read() -> Self {
        let mut config = Config::default();

        let read_config =
            std::fs::read_to_string(std::env::var("HOME").unwrap() + "/.config/rs-nc/config");
        if read_config.is_err() {
            log::warn!("Config file not found, using default values");
            return config;
        }

        let binding = read_config.unwrap();
        let lines = binding.lines();
        for line in lines {
            let mut splited = line.split('=');
            let key = match splited.next() {
                Some(key) => key.trim(),
                None => {
                    log::warn!("Incorrect line in config file: {}", line);
                    continue;
                }
            };
            let value = match splited.next() {
                Some(value) => value.trim(),
                None => {
                    log::warn!("Incorrect line in config file: {}", line);
                    continue;
                }
            };
            match key {
                "respect_notification_timeout" => {
                    config.respect_notification_timeout = match value.to_lowercase().as_str() {
                        "true" => true,
                        "false" => false,
                        _ => {
                            log::warn!(
                                "Incorrect value for respect_notification_timeout: {}",
                                value
                            );
                            log::warn!(
                                "Using default value: {}",
                                config.respect_notification_timeout
                            );
                            continue;
                        }
                    };
                }
                "local_expire_timeout" => {
                    config.local_expire_timeout = match value.parse() {
                        Ok(value) => {
                            if value <= 0 {
                                log::warn!("Value for local_expire_timeout must be greater than 0");
                                log::warn!("Using default value: {}", config.local_expire_timeout);
                                continue;
                            }
                            value
                        }
                        Err(e) => {
                            log::warn!("Incorrect value for local_expire_timeout: {}", e);
                            log::warn!("Using default value: {}", config.local_expire_timeout);
                            continue;
                        }
                    };
                }
                "max_notifications" => {
                    config.max_notifications = match value.parse() {
                        Ok(value) => {
                            if value <= 0 {
                                log::warn!("Value for max_notifications must be greater than 0");
                                log::warn!("Using default value: {}", config.max_notifications);
                                continue;
                            }
                            value
                        }
                        Err(e) => {
                            log::warn!("Incorrect value for max_notifications: {}", e);
                            log::warn!("Using default value: {}", config.max_notifications);
                            continue;
                        }
                    };
                }
                "height" => {
                    config.height = match value.parse() {
                        Ok(value) => {
                            if value <= 0 {
                                log::warn!("Value for height must be greater than 0");
                                log::warn!("Using default value: {}", config.height);
                                continue;
                            }
                            if value < 65 {
                                log::warn!("Recommended value for height is 65 or greater");
                            }
                            value
                        }
                        Err(e) => {
                            log::warn!("Incorrect value for height: {}", e);
                            log::warn!("Using default value: {}", config.height);
                            continue;
                        }
                    };
                }
                "width" => {
                    config.width = match value.parse() {
                        Ok(value) => {
                            if value <= 0 {
                                log::warn!("Value for width must be greater than 0");
                                log::warn!("Using default value: {}", config.width);
                                continue;
                            }
                            if value < 300 {
                                log::warn!("Recommended value for width is 300 or greater");
                            }
                            value
                        }
                        Err(e) => {
                            log::warn!("Incorrect value for width: {}", e);
                            log::warn!("Using default value: {}", config.width);
                            continue;
                        }
                    };
                }
                "vertical_margin" => {
                    config.vertical_margin = match value.parse() {
                        Ok(value) => {
                            if value < 0 {
                                log::warn!(
                                    "Value for vertical_margin must be greater than or equal to 0"
                                );
                                log::warn!("Using default value: {}", config.vertical_margin);
                                continue;
                            }
                            value
                        }
                        Err(e) => {
                            log::warn!("Incorrect value for vertical_margin: {}", e);
                            log::warn!("Using default value: {}", config.vertical_margin);
                            continue;
                        }
                    };
                }
                "horizontal_margin" => {
                    config.horizontal_margin = match value.parse() {
                        Ok(value) => {
                            if value < 0 {
                                log::warn!("Value for horizontal_margin must be greater than or equal to 0");
                                log::warn!("Using default value: {}", config.horizontal_margin);
                                continue;
                            }
                            value
                        }
                        Err(e) => {
                            log::warn!("Incorrect value for horizontal_margin: {}", e);
                            log::warn!("Using default value: {}", config.horizontal_margin);
                            continue;
                        }
                    };
                }
                "border_radius" => {
                    let nvidia_sucks = crate::data::shared_data::NVIDIA_SUCKS.lock().unwrap();
                    if *nvidia_sucks {
                        log::warn!("Nvidia moment, gl backend is used, border_radius is ignored and set to 0");
                        continue;
                    }
                    config.border_radius = iced::border::radius(match value.parse() {
                        Ok(value) => {
                            println!("value: {}", value);
                            value
                        }
                        Err(e) => {
                            log::warn!("Incorrect value for border_radius: {}", e);
                            log::warn!("Using default value: {}", config.border_radius.top_left);
                            config.border_radius.top_left
                        }
                    });
                }
                "border_color" => {
                    config.border_color = if let Some(color) = iced::Color::parse(value) {
                        color
                    } else {
                        log::warn!("Incorrect value for border_color: {}", value);
                        log::warn!("Using default value"); // TODO: later parse Color to string
                        config.border_color
                    };
                }
                "border_width" => {
                    config.border_width = match value.parse() {
                        Ok(value) => {
                            if value < 0.0 {
                                log::warn!(
                                    "Value for border_width must be greater than or equal to 0.0"
                                );
                                log::warn!("Using default value: {}", config.border_width);
                                continue;
                            }
                            value
                        }
                        Err(e) => {
                            log::warn!("Incorrect value for border_width: {}", e);
                            log::warn!("Using default value: {}", config.border_width);
                            continue;
                        }
                    };
                }
                "primary_text_color" => {
                    config.primary_text_color = if let Some(color) = iced::Color::parse(value) {
                        color
                    } else {
                        log::warn!("Incorrect value for primary_text_color: {}", value);
                        log::warn!("Using default value"); // TODO: later parse Color to string
                        config.primary_text_color
                    };
                }
                "secondary_text_color" => {
                    config.secondary_text_color = if let Some(color) = iced::Color::parse(value) {
                        color
                    } else {
                        log::warn!("Incorrect value for secondary_text_color: {}", value);
                        log::warn!("Using default value"); // TODO: later parse Color to string
                        config.secondary_text_color
                    };
                }
                "background_color" => {
                    config.background_color = if let Some(color) = iced::Color::parse(value) {
                        color
                    } else {
                        log::warn!("Incorrect value for background_color: {}", value);
                        log::warn!("Using default value"); // TODO: later parse Color to string
                        config.background_color
                    };
                }
                "respect_notification_icon" => {
                    config.respect_notification_icon = match value.to_lowercase().as_str() {
                        "true" => true,
                        "false" => false,
                        _ => {
                            log::warn!("Incorrect value for respect_notification_icon: {}", value);
                            log::warn!("Using default value: {}", config.respect_notification_icon);
                            continue;
                        }
                    };
                }
                _ => {
                    if key.starts_with('#') {
                        continue;
                    }
                    log::warn!("Unknown key in config file: {}", key);
                }
            }
        }

        config
    }
}
