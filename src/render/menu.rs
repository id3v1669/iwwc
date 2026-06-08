use iced::widget::{Space, column, container, image, mouse_area, row, text};
use iced::{Element, Font, Length};

use crate::config::resolved::ResolvedMenu;
use crate::render::UiMessage;
use crate::tray::menu_types::{MenuIcon, MenuItem, Toggle};

/// Submenu indicator glyph. Config??
pub const SUBMENU_ARROW: &str = "\u{25b8}";

fn measure_text(content: &str, font_size: f32, font: Option<Font>) -> (f32, f32) {
    use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
    use iced::advanced::text::{Alignment, LineHeight, Paragraph as _, Shaping, Text, Wrapping};

    if content.is_empty() {
        return (0.0, 0.0); //TODO: recheck the empty row
    }
    let text = Text {
        content,
        bounds: iced::Size::new(f32::INFINITY, f32::INFINITY),
        size: iced::Pixels(font_size),
        line_height: LineHeight::default(),
        font: font.unwrap_or_default(),
        align_x: Alignment::Default,
        align_y: iced::alignment::Vertical::Top,
        shaping: Shaping::Advanced,
        wrapping: Wrapping::None,
    };
    (
        GraphicsParagraph::with_text(text).min_width(),
        GraphicsParagraph::with_text(text).min_height(),
    )
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

pub fn menu_pixel_wh(items: &[MenuItem], m: &ResolvedMenu) -> (f32, f32) {
    let max_button_border = m
        .button_style
        .unwrap()
        .border
        .width
        .max(m.button_style_hover.unwrap().border.width)
        .max(m.button_style_active.unwrap().border.width)
        .max(m.button_style_disabled.unwrap().border.width);

    let cw = m.menu_container_style.unwrap().border.width * 2.0
        + m.menu_container_padding.left
        + m.menu_container_padding.right
        + max_button_border * 2.0
        + m.button_padding.left
        + m.button_padding.right;

    let rh = row_height(m);
    let (arrow_w, _) = measure_text(SUBMENU_ARROW, m.font_size, m.font);
    let mut max_w = 0.0_f32;
    let mut content_h = 0.0_f32;
    for item in items.iter().filter(|i| i.visible) {
        if item.separator {
            content_h += rh / 3.0;
            continue;
        }
        content_h += rh;
        let (mut w, _) = measure_text(&row_text(item), m.font_size, m.font);
        w += cw;
        if !matches!(item.icon, MenuIcon::None) {
            w += m.icon_size + m.row_spacing;
        }
        if item.has_submenu {
            w += m.row_spacing + arrow_w;
        }
        if w > max_w {
            max_w = w;
        }
    }

    let total_h = m.menu_container_padding.top
        + m.menu_container_padding.bottom
        + m.menu_container_style.unwrap().border.width * 2.0
        + content_h;

    let (sl, st, sr, sb) = shadow_extents(m);
    (max_w + sl + sr, total_h + st + sb)
}

pub fn row_height(m: &ResolvedMenu) -> f32 {
    let border = [
        m.button_style,
        m.button_style_hover,
        m.button_style_active,
        m.button_style_disabled,
    ]
    .into_iter()
    .map(|s| s.map_or(0.0, |s| s.border.width))
    .fold(0.0_f32, f32::max);
    let (_, line_h) = measure_text("M", m.font_size, m.font);
    line_h.max(m.icon_size) + m.button_padding.top + m.button_padding.bottom + border * 2.0
}

// TODO: Re-review that, maybe simplify to just radius??
fn shadow_extents(m: &ResolvedMenu) -> (f32, f32, f32, f32) {
    let sh = m.menu_container_style.map(|s| s.shadow).unwrap_or_default();
    let b = sh.blur_radius;
    let (ox, oy) = (sh.offset.x, sh.offset.y);
    (
        (b - ox).max(0.0),
        (b - oy).max(0.0),
        (b + ox).max(0.0),
        (b + oy).max(0.0),
    )
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

    let (width, _) = menu_pixel_wh(items, m);
    let row_h = row_height(m);
    let mut col = column![].width(Length::Fill);
    for item in items.iter().filter(|i| i.visible) {
        if item.separator {
            col = col.push(
                container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
                    .width(Length::Fill)
                    .height(Length::Fixed(row_h / 3.0)),
            );
            continue;
        }

        let mut line = row![
            text(row_text(item))
                .font(m.font.unwrap_or_default())
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
                    image(image::Handle::from_path(p))
                        .width(m.icon_size)
                        .height(m.icon_size)
                        .into()
                } else if let Some(found) = nix_freedesktop_icons::lookup(n)
                    .with_size(m.icon_size as u16)
                    .with_cache()
                    .find()
                {
                    image(image::Handle::from_path(found))
                        .width(m.icon_size)
                        .height(m.icon_size)
                        .into()
                } else {
                    Space::new().width(m.icon_size).height(m.icon_size).into()
                };
                line = row![h]
                    .push(line)
                    .spacing(m.row_spacing)
                    .align_y(iced::alignment::Vertical::Center);
            }
            MenuIcon::Png(handle) => {
                line = row![image(handle.clone()).width(m.icon_size).height(m.icon_size)]
                    .push(line)
                    .spacing(m.row_spacing)
                    .align_y(iced::alignment::Vertical::Center);
            }
            MenuIcon::None => {}
        }
        if item.has_submenu {
            line = line.push(
                text(SUBMENU_ARROW)
                    .font(m.font.unwrap_or_default())
                    .size(m.font_size)
                    .shaping(iced::widget::text::Shaping::Advanced),
            );
        }

        let id = item.id;
        let is_active_parent = active_id == Some(id);
        let menu = m.clone();
        let mut btn = button(line)
            .width(Length::Fill)
            .height(Length::Fixed(row_h))
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
    let (sl, st, sr, sb) = shadow_extents(m);
    let inner = container(col)
        .width(Length::Fixed(width - sl - sr))
        .height(Length::Shrink)
        .padding(m.menu_container_padding)
        .style(move |_| container_style);
    let surface: Element<'static, UiMessage> = if sl + st + sr + sb > 0.0 {
        container(inner)
            .padding(iced::Padding {
                top: st,
                right: sr,
                bottom: sb,
                left: sl,
            })
            .into()
    } else {
        inner.into()
    };
    if level >= 1 {
        mouse_area(surface)
            .on_exit(UiMessage::MenuLeave { level })
            .into()
    } else {
        surface
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
}
