#[derive(Debug, Clone, PartialEq)]
pub struct NotificationWindowInfo {
    pub notification: crate::data::notification::Notification,
    pub icon: std::path::PathBuf,
}

pub fn body(
    iwwc: &crate::gui::app::IcedWaylandWidgetCenter,
    window_info: NotificationWindowInfo,
) -> iced::widget::Container<'_, crate::gui::app::Message> {
    iced::widget::container(
        iced::widget::row![
            iced::widget::svg(window_info.icon.clone())
                .width(iced::Length::Fixed(iwwc.precalc.image_size))
                .height(iced::Length::Fixed(iwwc.precalc.image_size)),
            iced::widget::column![
                iced::widget::column![
                    iced::widget::text(window_info.notification.summary.clone())
                        .size(iwwc.precalc.font_size_summary)
                        .align_x(iced::alignment::Horizontal::Left),
                ]
                .padding(iwwc.precalc.text_summary_paddings),
                iced::widget::column![
                    iced::widget::text(window_info.notification.body.clone())
                        .color(iwwc.config.notifications.secondary_text_color)
                        .size(iwwc.precalc.font_size_body),
                ]
                .padding(iwwc.precalc.text_body_paddings),
            ]
            .padding(iwwc.precalc.text_paddings_block)
        ]
        .align_y(iced::alignment::Vertical::Center)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill),
    )
    .padding(iwwc.precalc.general_padding)
    .width(iced::Length::Fill)
    .height(iced::Length::Fill)
    .style(move |_| crate::gui::elements::style::notification_style(&iwwc.config))
}
