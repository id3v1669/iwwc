use iced::widget::{Space, column, container, image, mouse_area, row, text};
use iced::{Element, Length};

use crate::config::resolved::ResolvedMenu;
use crate::render::UiMessage;
#[cfg(test)]
use crate::render::convert;
use crate::tray::menu_types::{MenuIcon, MenuItem, Toggle};

/// Menu font family. Constant for now, find in old code converter of cfg to static
pub const MENU_FONT: iced::Font = iced::Font::DEFAULT;
/// Submenu indicator glyph. Config??
pub const SUBMENU_ARROW: &str = "\u{25b8}";

fn measure_text(content: &str, font_size: f32) -> f32 {
    use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
    use iced::advanced::text::{Alignment, LineHeight, Paragraph as _, Shaping, Text, Wrapping};

    if content.is_empty() {
        return 0.0;
    }
    let text = Text {
        content,
        bounds: iced::Size::new(f32::INFINITY, f32::INFINITY),
        size: iced::Pixels(font_size),
        line_height: LineHeight::default(),
        font: MENU_FONT,
        align_x: Alignment::Default,
        align_y: iced::alignment::Vertical::Top,
        shaping: Shaping::Advanced,
        wrapping: Wrapping::None,
    };
    GraphicsParagraph::with_text(text).min_width()
}

/// The exact text content a row renders: the toggle glyph prefix plus the label.
fn row_text(item: &MenuItem) -> String {
    let glyph = match item.toggle {
        Toggle::Check(true) => "\u{2713} ",
        Toggle::Radio(true) => "\u{25cf} ",
        Toggle::Check(false) | Toggle::Radio(false) => "  ",
        Toggle::None => "",
    };
    format!("{glyph}{}", item.label)
}

fn container_border_width(m: &ResolvedMenu) -> f32 {
    m.menu_container_style
        .as_ref()
        .map(|s| s.border.width)
        .unwrap_or(0.0)
}

pub fn menu_pixel_width(items: &[MenuItem], m: &ResolvedMenu) -> f32 {
    let border_w = container_border_width(m);
    let arrow_w = measure_text(SUBMENU_ARROW, m.font_size);
    let mut max = 0.0_f32;
    for item in items.iter().filter(|i| i.visible && !i.separator) {
        let mut w = 2.0 * border_w + 2.0 * m.button_padding.left + m.button_padding.right + measure_text(&row_text(item), m.font_size);
        if !matches!(item.icon, MenuIcon::None) {
            w += m.icon_size + m.row_spacing;
        }
        if item.has_submenu {
            w += m.row_spacing + arrow_w;
        }
        if w > max {
            max = w;
        }
    }
    max
}

pub fn menu_button_style(
    m: &ResolvedMenu,
    is_active_parent: bool,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    use iced::widget::button::Status;
    let slot = if is_active_parent {
        &m.button_style_active
    } else {
        match status {
            Status::Hovered => &m.button_style_hover,
            Status::Pressed => &m.button_style_active,
            Status::Disabled => &m.button_style_disabled,
            Status::Active => &m.button_style,
        }
    };
    (*slot).unwrap_or_default()
}

pub fn view_menu(
    items: &[MenuItem],
    level: usize,
    active_id: Option<i32>,
    m: &ResolvedMenu,
) -> Element<'static, UiMessage> {
    use iced::widget::button;

    let width = menu_pixel_width(items, m);
    let mut col = column![].width(Length::Fill);
    for item in items.iter().filter(|i| i.visible) {
        if item.separator {
            col = col.push(
                container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
                    .width(Length::Fill)
                    .height(Length::Fixed(7.0)),
            );
            continue;
        }

        let mut line = row![
            text(row_text(item))
                .font(MENU_FONT)
                .size(m.font_size)
                .shaping(iced::widget::text::Shaping::Advanced)
                .wrapping(iced::widget::text::Wrapping::None)
        ]
        .spacing(m.row_spacing)
        .align_y(iced::alignment::Vertical::Center)
        .width(Length::Fill);

        match &item.icon {
            MenuIcon::Name(n) => {
                let p = std::path::Path::new(n);
                let h: Element<'static, UiMessage> = if p.is_absolute() && p.is_file() {
                    image(image::Handle::from_path(p)).width(m.icon_size).height(m.icon_size).into()
                } else if let Some(found) =
                    nix_freedesktop_icons::lookup(n).with_size(m.icon_size as u16).with_cache().find()
                {
                    image(image::Handle::from_path(found)).width(m.icon_size).height(m.icon_size).into()
                } else {
                    Space::new().width(m.icon_size).height(m.icon_size).into()
                };
                line = row![h].push(line).spacing(m.row_spacing).align_y(iced::alignment::Vertical::Center);
            }
            MenuIcon::Png(bytes) => {
                line = row![
                    image(image::Handle::from_bytes(bytes.clone())).width(m.icon_size).height(m.icon_size)
                ]
                .push(line)
                .spacing(m.row_spacing)
                .align_y(iced::alignment::Vertical::Center);
            }
            MenuIcon::None => {}
        }
        if item.has_submenu {
            line = line.push(
                text(SUBMENU_ARROW)
                    .font(MENU_FONT)
                    .size(m.font_size)
                    .shaping(iced::widget::text::Shaping::Advanced),
            );
        }

        let id = item.id;
        let is_active_parent = active_id == Some(id);
        let menu = m.clone();
        let mut btn = button(line)
            .width(Length::Fill)
            .height(Length::Fixed(m.row_height))
            .padding(m.button_padding)
            .style(move |_theme, status| menu_button_style(&menu, is_active_parent, status));
        if item.enabled {
            if item.has_submenu {
                btn = btn.on_press(UiMessage::MenuHover { level, id });
            } else {
                btn = btn.on_press(UiMessage::MenuClick { level, id });
            }
        }
        let area = mouse_area(btn).on_enter(UiMessage::MenuHover { level, id });
        col = col.push(area);
    }

    let container_style = m.menu_container_style.unwrap_or_default();
    let surface = container(col)
        .width(Length::Fixed(
            width + m.menu_container_padding.left + m.menu_container_padding.right,
        ))
        .height(Length::Shrink)
        .padding(m.menu_container_padding)
        .style(move |_| container_style);
    if level >= 1 {
        mouse_area(surface).on_exit(UiMessage::MenuLeave { level }).into()
    } else {
        surface.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::resolved::ResolvedApptraySettings;
    use crate::tray::menu_types::{MenuIcon, MenuItem, Toggle};

    fn make_item(id: i32, label: &str, separator: bool, has_submenu: bool) -> MenuItem {
        MenuItem {
            id,
            label: label.to_string(),
            enabled: true,
            visible: true,
            separator,
            toggle: Toggle::None,
            icon: MenuIcon::None,
            has_submenu,
            children: vec![],
        }
    }

    //#[test]
    //fn button_style_maps_states() TODO: #test1

    #[test]
    fn view_menu_builds_with_states() {
        let mut disabled = make_item(4, "Disabled", false, false);
        disabled.enabled = false;
        let items = vec![
            make_item(1, "Open", false, false),
            make_item(2, "", true, false),
            make_item(3, "Submenu", false, true),
            disabled,
        ];
        let s = ResolvedApptraySettings::default();
        let _el = view_menu(&items, 1, Some(3), &s.menu);
    }

    #[test]
    fn longer_label_not_narrower() {
        let m = ResolvedMenu::default();
        let short = vec![make_item(1, "Open", false, false)];
        let long = vec![make_item(1, "Open Recent Files And Folders", false, false)];
        assert!(menu_pixel_width(&long, &m) >= menu_pixel_width(&short, &m));
    }

    #[test]
    fn submenu_row_wider_than_plain() {
        let m = ResolvedMenu::default();
        let plain = vec![make_item(1, "Settings", false, false)];
        let sub = vec![make_item(1, "Settings", false, true)];
        assert!(menu_pixel_width(&sub, &m) > menu_pixel_width(&plain, &m));
    }

    #[test]
    fn width_is_finite_and_positive() {
        let m = ResolvedMenu::default();
        let items = vec![make_item(1, "Тест \u{1f600}", false, false)];
        let w = menu_pixel_width(&items, &m);
        assert!(w.is_finite() && w > 0.0);
    }
}
