pub fn body(
    iwwc: &crate::gui::app::IcedWaylandWidgetCenter,
    window_info: String,
) -> iced::widget::Container<'_, crate::gui::app::Message> {
    if let Some(container) = iwwc.config.containers.iter().find(|c| c.id == window_info) {
        let child_content = build_child_element(iwwc, &container.child);

        return iced::widget::container(child_content)
            .width(container.width)
            .height(container.height)
            .style(|_theme| container.style.clone());
    }

    let content = build_child_element(iwwc, &window_info);
    iced::widget::container(content)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
}

fn build_element<'a>(
    iwwc: &'a crate::gui::app::IcedWaylandWidgetCenter,
    element_id: &str,
) -> iced::widget::Container<'a, crate::gui::app::Message> {
    if let Some(container) = iwwc.config.containers.iter().find(|c| c.id == element_id) {
        let child_content = build_child_element(iwwc, &container.child);

        return iced::widget::container(child_content)
            .width(container.width)
            .height(container.height)
            .style(|_theme| container.style.clone());
    }

    let content = build_child_element(iwwc, element_id);
    iced::widget::container(content)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
}

fn build_child_element<'a>(
    iwwc: &'a crate::gui::app::IcedWaylandWidgetCenter,
    element_id: &str,
) -> iced::Element<'a, crate::gui::app::Message> {
    if let Some(row) = iwwc.config.rows.iter().find(|r| r.id == element_id) {
        let mut row_widget = iced::widget::Row::new().align_y(row.allinment);

        for child_id in &row.children {
            let child = build_child_element(iwwc, child_id);
            row_widget = row_widget.push(child);
        }

        return row_widget.into();
    }

    if let Some(column) = iwwc.config.columns.iter().find(|c| c.id == element_id) {
        let mut column_widget = iced::widget::Column::new().align_x(column.allinment);

        for child_id in &column.children {
            let child = build_child_element(iwwc, child_id);
            column_widget = column_widget.push(child);
        }

        return column_widget.into();
    }

    if let Some(button) = iwwc.config.buttons.iter().find(|b| b.id == element_id) {
        return iced::widget::button(iced::widget::text(&button.text))
            .width(button.width)
            .height(button.height)
            .padding(button.padding)
            .style(|_, _| button.style.clone())
            .on_press(crate::gui::app::Message::TestMessage)
            .into();
    }

    if let Some(container) = iwwc.config.containers.iter().find(|c| c.id == element_id) {
        let child_content = build_child_element(iwwc, &container.child);

        return iced::widget::container(child_content)
            .width(container.width)
            .height(container.height)
            .style(|_theme| container.style.clone())
            .into();
    }

    log::warn!("Element not found: {}", element_id);
    iced::widget::text(format!("Unknown element: {}", element_id)).into()
}
