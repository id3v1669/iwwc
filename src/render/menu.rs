use iced::widget::{Space, column, container, image, mouse_area, row, text};
use iced::{Element, Length};

use crate::config::resolved::ResolvedApptraySettings;
use crate::render::{UiMessage, convert, style};
use crate::tray::menu_types::{MenuIcon, MenuItem, Toggle};

pub fn menu_pixel_width(items: &[MenuItem], s: &ResolvedApptraySettings) -> f32 {
    const CHAR_W: f32 = 8.5;
    const ICON_W: f32 = 22.0;
    const EXTRA_W: f32 = 22.0;
    const PADDING_W: f32 = 28.0;
    let mut max = s.menu_width;
    for item in items.iter().filter(|i| i.visible && !i.separator) {
        let mut w = PADDING_W + item.label.chars().count() as f32 * CHAR_W;
        if !matches!(item.icon, MenuIcon::None) {
            w += ICON_W;
        }
        if !matches!(item.toggle, Toggle::None) {
            w += EXTRA_W;
        }
        if item.has_submenu {
            w += EXTRA_W;
        }
        if w > max {
            max = w;
        }
    }
    max.min(640.0)
}

pub fn view_menu(
    items: &[MenuItem],
    level: usize,
    s: &ResolvedApptraySettings,
) -> Element<'static, UiMessage> {
    let width = menu_pixel_width(items, s);
    let mut col = column![].width(Length::Fixed(width));
    for item in items.iter().filter(|i| i.visible) {
        if item.separator {
            col = col.push(
                container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
                    .width(Length::Fill)
                    .height(Length::Fixed(7.0)),
            );
            continue;
        }
        let glyph = match item.toggle {
            Toggle::Check(true) => "\u{2713} ",
            Toggle::Radio(true) => "\u{25cf} ",
            Toggle::Check(false) | Toggle::Radio(false) => "  ",
            Toggle::None => "",
        };
        let color = if item.enabled {
            s.menu_text
        } else {
            s.menu_disabled
        };
        let mut line = row![
            text(format!("{glyph}{}", item.label))
                .color(convert::color(color))
                .wrapping(iced::widget::text::Wrapping::None)
        ]
        .spacing(6.0)
        .align_y(iced::alignment::Vertical::Center)
        .width(Length::Fill);
        match &item.icon {
            MenuIcon::Name(n) => {
                let p = std::path::Path::new(n);
                let h: Element<'static, UiMessage> = if p.is_absolute() && p.is_file() {
                    image(image::Handle::from_path(p))
                        .width(16.0)
                        .height(16.0)
                        .into()
                } else if let Some(found) = nix_freedesktop_icons::lookup(n)
                    .with_size(16)
                    .with_cache()
                    .find()
                {
                    image(image::Handle::from_path(found))
                        .width(16.0)
                        .height(16.0)
                        .into()
                } else {
                    Space::new().width(16.0).height(16.0).into()
                };
                line = row![h]
                    .push(line)
                    .spacing(6.0)
                    .align_y(iced::alignment::Vertical::Center);
            }
            MenuIcon::Png(bytes) => {
                line = row![
                    image(image::Handle::from_bytes(bytes.clone()))
                        .width(16.0)
                        .height(16.0)
                ]
                .push(line)
                .spacing(6.0)
                .align_y(iced::alignment::Vertical::Center);
            }
            MenuIcon::None => {}
        }
        if item.has_submenu {
            line = line.push(text("\u{25b8}").color(convert::color(color)));
        }
        let id = item.id;
        let rowel = container(line)
            .width(Length::Fill)
            .height(Length::Fixed(s.row_height))
            .padding([2u16, 8u16]);
        let mut area = mouse_area(rowel).on_enter(UiMessage::MenuHover { level, id });
        if item.enabled {
            if item.has_submenu {
                area = area.on_press(UiMessage::MenuHover { level, id });
            } else {
                area = area.on_press(UiMessage::MenuClick { level, id });
            }
        }
        col = col.push(area);
    }
    let menu_bg = s.menu_bg;
    let surface = container(col)
        .width(Length::Fixed(width))
        .style(move |_| container::Style {
            background: Some(style::background(menu_bg)),
            ..Default::default()
        });
    if level >= 1 {
        mouse_area(surface)
            .on_exit(UiMessage::MenuLeave { level })
            .into()
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

    #[test]
    fn view_menu_builds_with_normal_separator_submenu() {
        let items = vec![
            make_item(1, "Open", false, false),
            make_item(2, "", true, false),
            make_item(3, "Submenu", false, true),
        ];
        let s = ResolvedApptraySettings::default();
        let _el = view_menu(&items, 0, &s);
    }
}
