pub fn notification_style() -> iced::widget::container::Style {
    let config = crate::data::shared::CONFIG.lock().unwrap();
    iced::widget::container::Style {
        text_color: Some(config.primary_text_color),
        border: iced::Border {
            color: config.border_color,
            width: config.border_width,
            radius: config.border_radius,
        },
        shadow: iced::Shadow {
            //has to be here as empty shadow is not allowed and no paddings yet to make it visible
            color: iced::Color::TRANSPARENT,
            offset: iced::Vector { x: 0.0, y: 0.0 },
            blur_radius: 0.0,
        },
        background: Some(iced::Background::Color(config.background_color)),
        snap: false,
    }
}
