use std::path::Path;

use iced::widget::{button, column, container, image, row, svg, text};
use iced::{Element, Length};

use crate::config::resolved::ResolvedNotificationSettings;
use crate::notification::types::{Notification, PreCalc, action_pairs};
use crate::render::UiMessage;
use crate::render::convert;
use crate::render::style;

pub fn view_notification<'a>(
    settings: &ResolvedNotificationSettings,
    precalc: &PreCalc,
    n: &Notification,
    icon: &Path,
) -> Element<'a, UiMessage> {
    let id = n.notification_id;

    let icon_el: Element<'a, UiMessage> =
        if icon.extension().and_then(|s| s.to_str()) == Some("svg") {
            svg(svg::Handle::from_path(icon))
                .width(Length::Fixed(precalc.image_size))
                .height(Length::Fixed(precalc.image_size))
                .into()
        } else {
            image(image::Handle::from_path(icon))
                .width(Length::Fixed(precalc.image_size))
                .height(Length::Fixed(precalc.image_size))
                .into()
        };

    let mut summary = text(n.summary.clone())
        .size(precalc.font_size_summary)
        .color(convert::color(settings.primary_text));
    if let Some(f) = &settings.font {
        summary = summary.font(convert::font(f));
    }

    let mut body = text(n.body.clone())
        .size(precalc.font_size_body)
        .color(convert::color(settings.secondary_text));
    if let Some(f) = &settings.font {
        body = body.font(convert::font(f));
    }

    let text_block = column![
        column![summary].padding(precalc.text_summary_paddings),
        column![body].padding(precalc.text_body_paddings),
    ]
    .padding(precalc.text_paddings_block);

    let body_row = row![icon_el, text_block]
        .align_y(iced::alignment::Vertical::Center)
        .width(Length::Fill);

    let mut content = column![body_row];

    let buttons: Vec<Element<'a, UiMessage>> = action_pairs(&n.actions)
        .into_iter()
        .filter(|(k, _)| k != "default")
        .map(|(key, label)| {
            button(text(label).size(precalc.font_size_body))
                .on_press(UiMessage::NotifAction { id, key })
                .into()
        })
        .collect();
    if !buttons.is_empty() {
        content = content.push(row(buttons).spacing(precalc.general_padding));
    }

    let inner: Element<'a, UiMessage> = content.width(Length::Fill).height(Length::Fill).into();

    let bg = settings.bg;
    let border = settings.border.clone();
    container(inner)
        .padding(precalc.general_padding)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(style::background(bg)),
            border: border.as_ref().map(style::border).unwrap_or_default(),
            ..Default::default()
        })
        .into()
}
