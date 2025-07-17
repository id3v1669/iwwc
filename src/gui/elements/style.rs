pub fn notification_style(config: &crate::data::config::Config) -> iced::widget::container::Style {
    iced::widget::container::Style {
        text_color: Some(config.notifications.primary_text_color),
        border: iced::Border {
            color: config.notifications.border_color,
            width: config.notifications.border_width,
            radius: config.notifications.border_radius,
        },
        shadow: iced::Shadow {
            //has to be here as empty shadow is not allowed and no paddings yet to make it visible
            color: iced::Color::TRANSPARENT,
            offset: iced::Vector { x: 0.0, y: 0.0 },
            blur_radius: 0.0,
        },
        background: Some(iced::Background::Color(
            config.notifications.background_color,
        )),
        snap: false,
    }
}
