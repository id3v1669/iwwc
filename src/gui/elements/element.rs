pub fn body(
    iwwc: &crate::gui::app::IcedWaylandWidgetCenter,
    window_info: String,
) -> iced::widget::Container<'_, crate::gui::app::Message> {
    if let Some(container) = iwwc.config.containers.iter().find(|c| c.id == window_info) {
        let child_content = build_child_element(iwwc, &container.child);

        return iced::widget::container(child_content)
            .padding(container.padding)
            .width(container.width)
            .height(container.height)
            .align_x(container.align_x)
            .align_y(container.align_y)
            .style(|_theme| container.style.clone());
    }
    // Should not be reached, better way to handle this?
    log::warn!("Container not found: {}", window_info);
    iced::widget::container(iced::widget::horizontal_space())
}

fn build_child_element<'a>(
    iwwc: &'a crate::gui::app::IcedWaylandWidgetCenter,
    element_id: &str,
) -> iced::Element<'a, crate::gui::app::Message> {
    if let Some(row) = iwwc.config.rows.iter().find(|r| r.id == element_id) {
        let mut row_widget = iced::widget::Row::new()
            .spacing(row.spacing)
            .padding(row.padding)
            .width(row.width)
            .height(row.height)
            .align_y(row.allinment);

        for child_id in &row.children {
            let child = build_child_element(iwwc, child_id);
            row_widget = row_widget.push(child);
        }

        return row_widget.into();
    }

    if let Some(column) = iwwc.config.columns.iter().find(|c| c.id == element_id) {
        let mut column_widget = iced::widget::Column::new()
            .spacing(column.spacing)
            .padding(column.padding)
            .width(column.width)
            .height(column.height)
            .align_x(column.allinment);

        for child_id in &column.children {
            let child = build_child_element(iwwc, child_id);
            column_widget = column_widget.push(child);
        }

        return column_widget.into();
    }

    if let Some(button) = iwwc.config.buttons.iter().find(|b| b.id == element_id) {
        let child_content = build_child_element(iwwc, &button.text);

        return iced::widget::button(child_content)
            .width(button.width)
            .height(button.height)
            .padding(button.padding)
            .style(|_, status| match status {
                iced::widget::button::Status::Active => button.style_active.clone(),
                iced::widget::button::Status::Hovered => button.style_hover.clone(),
                iced::widget::button::Status::Pressed => button.style_pressed.clone(),
                _ => iced::widget::button::Style::default(),
            })
            .on_press(crate::gui::app::Message::TestMessage)
            .into();
    }

    if let Some(container) = iwwc.config.containers.iter().find(|c| c.id == element_id) {
        let child_content = build_child_element(iwwc, &container.child);

        return iced::widget::container(child_content)
            .padding(container.padding)
            .width(container.width)
            .height(container.height)
            .align_x(container.align_x)
            .align_y(container.align_y)
            .style(|_theme| container.style.clone())
            .into();
    }

    if let Some(text) = iwwc.config.texts.iter().find(|t| t.id == element_id) {
        return iced::widget::text(&text.text)
            .width(text.width)
            .height(text.height)
            .align_x(text.align_x)
            .align_y(text.align_y)
            .size(text.font_size)
            .color(text.color)
            .font(text.font)
            .into();
    }

    // Should not be reached, better way to handle this?
    log::warn!("Element not found: {}", element_id);
    iced::widget::text(format!("Unknown element: {}", element_id)).into()
}
