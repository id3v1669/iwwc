use std::path::Path;

use iced::widget::{button, column, container, image, row, svg, text};
use iced::{Element, Font, Length};

use crate::config::resolved::ResolvedNotificationSettings;
use crate::notification::types::{Notification, PreCalc, action_pairs};
use crate::render::UiMessage;

fn wrapped_text_height(content: &str, font_size: f32, font: Option<Font>, max_width: f32) -> f32 {
    use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
    use iced::advanced::text::{Alignment, LineHeight, Paragraph as _, Shaping, Text, Wrapping};

    if content.is_empty() {
        return 0.0;
    }
    let text = Text {
        content,
        bounds: iced::Size::new(max_width, f32::INFINITY),
        size: iced::Pixels(font_size),
        line_height: LineHeight::default(),
        font: font.unwrap_or_default(),
        align_x: Alignment::Default,
        align_y: iced::alignment::Vertical::Top,
        shaping: Shaping::Advanced,
        wrapping: Wrapping::default(),
    };
    GraphicsParagraph::with_text(text).min_height()
}

pub fn measure_height(
    settings: &ResolvedNotificationSettings,
    precalc: &PreCalc,
    n: &Notification,
) -> f32 {
    let pad = precalc.general_padding;
    let block = precalc.text_paddings_block;
    let text_w = settings.width - 2.0 * pad - precalc.image_size - block.left - block.right;
    let summary_h = wrapped_text_height(
        &n.summary,
        precalc.font_size_summary,
        settings.font,
        text_w - precalc.text_summary_paddings.left,
    );
    let body_h = wrapped_text_height(
        &n.body,
        precalc.font_size_body,
        settings.font,
        text_w - precalc.text_body_paddings.left,
    );
    let text_block_h = block.top + block.bottom + summary_h + body_h;
    let body_row_h = precalc.image_size.max(text_block_h);
    let has_buttons = action_pairs(&n.actions).iter().any(|(k, _)| k != "default");
    let buttons_h = if has_buttons {
        wrapped_text_height("M", precalc.font_size_body, settings.font, f32::INFINITY)
            + iced::widget::button::DEFAULT_PADDING.top
            + iced::widget::button::DEFAULT_PADDING.bottom
    } else {
        0.0
    };
    (body_row_h + buttons_h + 2.0 * pad).ceil()
}

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
        .color(settings.primary_text)
        .shaping(text::Shaping::Advanced);
    if let Some(f) = &settings.font {
        summary = summary.font(*f);
    }

    let mut body = text(n.body.clone())
        .size(precalc.font_size_body)
        .color(settings.secondary_text)
        .shaping(text::Shaping::Advanced);
    if let Some(f) = &settings.font {
        body = body.font(*f);
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
        .enumerate()
        .map(|(i, (key, label))| {
            let (base, hover, active, disabled) = if i == 0 {
                (
                    settings.ok_style,
                    settings.ok_style_hover,
                    settings.ok_style_active,
                    settings.ok_style_disabled,
                )
            } else {
                (
                    settings.no_style,
                    settings.no_style_hover,
                    settings.no_style_active,
                    settings.no_style_disabled,
                )
            };
            button(text(label).size(precalc.font_size_body))
                .on_press(UiMessage::NotifAction { id, key })
                .style(move |_theme, status| {
                    let chosen = match status {
                        button::Status::Hovered => hover.or(base),
                        button::Status::Pressed => active.or(base),
                        button::Status::Disabled => disabled.or(base),
                        button::Status::Active => base,
                    };
                    chosen.unwrap_or_default()
                })
                .into()
        })
        .collect();
    if !buttons.is_empty() {
        content = content.push(
            container(row(buttons).spacing(precalc.general_padding))
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right),
        );
    }

    let inner: Element<'a, UiMessage> = content.width(Length::Fill).height(Length::Fill).into();

    let bg = settings.bg;
    let border = settings.border;
    let base = container(inner)
        .padding(precalc.general_padding)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg)),
            border: border.unwrap_or_default(),
            ..Default::default()
        });

    let dot_size = settings.width * 0.02;
    let dot_color = settings.urgency_color[n.urgency.min(2) as usize];
    let dot = container(column![])
        .width(dot_size)
        .height(dot_size)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(dot_color)),
            border: iced::border::rounded(dot_size / 2.0),
            ..Default::default()
        });

    iced::widget::stack![
        base,
        container(dot)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right)
            .padding(precalc.general_padding)
    ]
    .into()
}
