pub mod convert;
pub mod menu;
pub mod notification;
pub mod style;

#[derive(Debug, Clone)]
pub enum UiMessage {
    Action(String),
    NotifAction {
        id: u32,
        key: String,
    },
    TrayActivate(usize),
    TraySecondary(usize),
    TrayContextMenu {
        window: iced::window::Id,
        idx: usize,
    },
    TrayScroll {
        idx: usize,
        delta: f32,
    },
    MenuClick {
        level: usize,
        id: i32,
    },
    MenuHover {
        level: usize,
        id: i32,
    },
    MenuLeave {
        level: usize,
    },
    MenuDismiss,
}

pub struct RenderCtx<'a> {
    pub tray: &'a [crate::tray::types::TrayItem],
    pub window: iced::window::Id,
}

use crate::config::resolved::{
    ResolvedApptraySettings, ResolvedButton, ResolvedColumn, ResolvedContainer, ResolvedElement,
    ResolvedRow, ResolvedText, ResolvedWidget,
};
use crate::tray::types::TrayIcon;
use iced::Element;
use iced::widget::{Column, Row, button, container, text};

pub fn view_widget(w: &ResolvedWidget, ctx: &RenderCtx) -> Element<'static, UiMessage> {
    match &w.child {
        Some(child) => view_element(child, ctx),
        None => text("").into(),
    }
}

fn view_element(el: &ResolvedElement, ctx: &RenderCtx) -> Element<'static, UiMessage> {
    match el {
        ResolvedElement::Container(c) => build_container(c, ctx),
        ResolvedElement::Button(b) => build_button(b, ctx),
        ResolvedElement::Row(r) => build_row(r, ctx),
        ResolvedElement::Column(c) => build_column(c, ctx),
        ResolvedElement::Text(t) => build_text(t, ctx),
        ResolvedElement::Apptray(s) => build_apptray(s, ctx),
    }
}

fn build_text(t: &ResolvedText, _ctx: &RenderCtx) -> Element<'static, UiMessage> {
    let mut el = text(t.content.clone().unwrap_or_default());
    if let Some(w) = t.w {
        el = el.width(w);
    }
    if let Some(h) = t.h {
        el = el.height(h);
    }
    if let Some(c) = t.color {
        el = el.color(c);
    }
    if let Some(f) = &t.font {
        el = el.font(convert::font(f));
    }
    if let Some(ax) = t.align_x {
        el = el.align_x(convert::text_align_x(ax));
    }
    if let Some(ay) = t.align_y {
        el = el.align_y(convert::align_y(ay));
    }
    el.into()
}

fn build_container(c: &ResolvedContainer, ctx: &RenderCtx) -> Element<'static, UiMessage> {
    let mut el = container(view_element(&c.child, ctx));
    if let Some(w) = c.w {
        el = el.width(w);
    }
    if let Some(h) = c.h {
        el = el.height(h);
    }
    if let Some(p) = c.padding {
        el = el.padding(p);
    }
    if let Some(ax) = c.align_x {
        el = el.align_x(convert::align_x(ax));
    }
    if let Some(ay) = c.align_y {
        el = el.align_y(convert::align_y(ay));
    }
    if let Some(clip) = c.clip {
        el = el.clip(clip);
    }
    if let Some(s) = c.style {
        el = el.style(move |_theme| s);
    }
    el.into()
}

fn build_button(b: &ResolvedButton, _ctx: &RenderCtx) -> Element<'static, UiMessage> {
    let mut content = text(b.text.clone().unwrap_or_default());
    if let Some(f) = &b.font {
        content = content.font(convert::font(f));
    }
    let mut el = button(content);
    if let Some(w) = b.w {
        el = el.width(w);
    }
    if let Some(h) = b.h {
        el = el.height(h);
    }
    if let Some(p) = b.padding {
        el = el.padding(p);
    }
    if let Some(clip) = b.clip {
        el = el.clip(clip);
    }
    el = el.on_press_maybe(b.action.clone().map(UiMessage::Action));

    let base = b.style;
    let hover = b.style_hover;
    let active = b.style_active;
    let disabled = b.style_disabled;
    el = el.style(move |_theme, status| {
        let chosen = match status {
            button::Status::Hovered => hover.as_ref().or(base.as_ref()),
            button::Status::Pressed => active.as_ref().or(base.as_ref()),
            button::Status::Disabled => disabled.as_ref().or(base.as_ref()),
            button::Status::Active => base.as_ref(),
        };
        chosen.cloned().unwrap_or_default()
    });
    el.into()
}

fn build_row(r: &ResolvedRow, ctx: &RenderCtx) -> Element<'static, UiMessage> {
    let children: Vec<Element<'static, UiMessage>> =
        r.children.iter().map(|e| view_element(e, ctx)).collect();
    let mut el = Row::with_children(children);
    if let Some(w) = r.w {
        el = el.width(w);
    }
    if let Some(h) = r.h {
        el = el.height(h);
    }
    if let Some(p) = r.padding {
        el = el.padding(p);
    }
    if let Some(sp) = r.spacing {
        el = el.spacing(sp);
    }
    if let Some(clip) = r.clip {
        el = el.clip(clip);
    }
    if let Some(a) = r.align {
        el = el.align_y(convert::row_align(a));
    }
    el.into()
}

fn build_column(c: &ResolvedColumn, ctx: &RenderCtx) -> Element<'static, UiMessage> {
    let children: Vec<Element<'static, UiMessage>> =
        c.children.iter().map(|e| view_element(e, ctx)).collect();
    let mut el = Column::with_children(children);
    if let Some(w) = c.w {
        el = el.width(w);
    }
    if let Some(h) = c.h {
        el = el.height(h);
    }
    if let Some(p) = c.padding {
        el = el.padding(p);
    }
    if let Some(sp) = c.spacing {
        el = el.spacing(sp);
    }
    if let Some(clip) = c.clip {
        el = el.clip(clip);
    }
    if let Some(a) = c.align {
        el = el.align_x(convert::col_align(a));
    }
    el.into()
}

fn scroll_y(delta: iced::mouse::ScrollDelta) -> f32 {
    match delta {
        iced::mouse::ScrollDelta::Lines { y, .. } => y,
        iced::mouse::ScrollDelta::Pixels { y, .. } => y,
    }
}

fn build_apptray(s: &ResolvedApptraySettings, ctx: &RenderCtx) -> Element<'static, UiMessage> {
    let size = s.icon_size;
    let swap = s.swap_buttons;
    let mut items: Vec<Element<'static, UiMessage>> = Vec::new();
    for (idx, item) in ctx.tray.iter().enumerate() {
        let icon: Element<'static, UiMessage> = match &item.icon {
            TrayIcon::Path(p) if p.extension().and_then(|e| e.to_str()) == Some("svg") => {
                iced::widget::svg(iced::widget::svg::Handle::from_path(p))
                    .width(size)
                    .height(size)
                    .into()
            }
            TrayIcon::Path(p) => iced::widget::image(iced::widget::image::Handle::from_path(p))
                .width(size)
                .height(size)
                .into(),
            TrayIcon::Pixmap { w, h, rgba } => {
                iced::widget::image(iced::widget::image::Handle::from_rgba(*w, *h, rgba.clone()))
                    .width(size)
                    .height(size)
                    .into()
            }
            TrayIcon::None => iced::widget::Space::new().width(size).height(size).into(),
        };
        let menu_msg = UiMessage::TrayContextMenu {
            window: ctx.window,
            idx,
        };
        let (left, right) = if swap {
            (menu_msg, UiMessage::TrayActivate(idx))
        } else {
            (UiMessage::TrayActivate(idx), menu_msg)
        };
        let area = iced::widget::mouse_area(icon)
            .on_press(left)
            .on_right_press(right)
            .on_middle_press(UiMessage::TraySecondary(idx))
            .on_scroll(move |delta| UiMessage::TrayScroll {
                idx,
                delta: scroll_y(delta),
            });
        items.push(area.into());
    }
    let pad = s.padding;
    let inner: Element<'static, UiMessage> = if s.vertical {
        let mut col = Column::with_children(items).spacing(s.spacing);
        if let Some(p) = pad {
            col = col.padding(p);
        }
        col.into()
    } else {
        let mut row = Row::with_children(items).spacing(s.spacing);
        if let Some(p) = pad {
            row = row.padding(p);
        }
        row.into()
    };
    let bg = s.bg;
    let border = s.border;
    if bg.is_some() || border.is_some() {
        iced::widget::container(inner)
            .style(move |_| iced::widget::container::Style {
                background: bg.map(style::background),
                border: border.unwrap_or_default(),
                ..Default::default()
            })
            .into()
    } else {
        inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_str;
    use crate::config::resolved::ResolvedConfig;
    use crate::config::resolver::resolve;

    fn render_kdl(kdl: &str) -> ResolvedConfig {
        let (cfg, perrs) = parse_str(kdl, "<test>");
        let cfg = cfg.expect("fixture parses");
        assert!(
            perrs
                .iter()
                .all(|e| e.severity != crate::config::Severity::Error),
            "parse errs: {:?}",
            perrs
        );
        let (rc, rerrs) = resolve(&cfg);
        assert!(
            rerrs
                .iter()
                .all(|e| e.severity != crate::config::Severity::Error),
            "resolve errs: {:?}",
            rerrs
        );
        rc.expect("resolves")
    }

    #[test]
    fn renders_text_widget() {
        let rc = render_kdl("widget bar child=t1\ntext t1 color=ffffff font=\"Sans\" align_x=c");
        let w = rc.widgets.get("bar").unwrap();
        let _el = view_widget(
            w,
            &RenderCtx {
                tray: &[],
                window: iced::window::Id::unique(),
            },
        );
    }

    #[test]
    fn renders_container_with_nested_text() {
        let rc = render_kdl(
            "widget bar child=box1\ncontainer box1 w=200 h=40 padding=5 align_x=c align_y=c clip=#true child=t1\ntext t1 color=ffffff",
        );
        let w = rc.widgets.get("bar").unwrap();
        let _el = view_widget(
            w,
            &RenderCtx {
                tray: &[],
                window: iced::window::Id::unique(),
            },
        );
    }

    #[test]
    fn renders_container_with_style() {
        let rc = render_kdl(
            "widget bar child=box1\ncontainer box1 style=s1 child=t1\ntext t1\nstyle s1 bg=000000 text=ffffff border=b1 shadow=sh1\nborder b1 color=ffffff w=2 radius=5\nshadow sh1 color=000000 blur_radius=1 {\n  offset 1 2\n}",
        );
        let w = rc.widgets.get("bar").unwrap();
        let _el = view_widget(
            w,
            &RenderCtx {
                tray: &[],
                window: iced::window::Id::unique(),
            },
        );
    }

    #[test]
    fn renders_empty_child() {
        let rc = render_kdl("widget bar layer=top");
        let w = rc.widgets.get("bar").unwrap();
        let _el = view_widget(
            w,
            &RenderCtx {
                tray: &[],
                window: iced::window::Id::unique(),
            },
        );
    }

    #[test]
    fn renders_button_with_action_and_styles() {
        let rc = render_kdl(
            "widget bar child=btn\nbutton btn text=\"hi\" action=\"echo x\" w=40 padding=5 clip=#true style=s1 style:hover=s2 style:active=s1 style:disabled=s2\nstyle s1 bg=ffffff text=000000\nstyle s2 bg=000000 text=ffffff",
        );
        let w = rc.widgets.get("bar").unwrap();
        let _el = view_widget(
            w,
            &RenderCtx {
                tray: &[],
                window: iced::window::Id::unique(),
            },
        );
    }

    #[test]
    fn renders_button_minimal() {
        let rc = render_kdl("widget bar child=btn\nbutton btn");
        let w = rc.widgets.get("bar").unwrap();
        let _el = view_widget(
            w,
            &RenderCtx {
                tray: &[],
                window: iced::window::Id::unique(),
            },
        );
    }

    #[test]
    fn renders_row_with_children() {
        let rc = render_kdl(
            "widget bar child=r1\nrow r1 spacing=5 align=c w=fill {\n  children a b\n}\nbutton a\nbutton b",
        );
        let w = rc.widgets.get("bar").unwrap();
        let _el = view_widget(
            w,
            &RenderCtx {
                tray: &[],
                window: iced::window::Id::unique(),
            },
        );
    }

    #[test]
    fn renders_column_with_children() {
        let rc = render_kdl(
            "widget bar child=c1\ncolumn c1 spacing=2 align=l {\n  children a b\n}\ntext a\ntext b",
        );
        let w = rc.widgets.get("bar").unwrap();
        let _el = view_widget(
            w,
            &RenderCtx {
                tray: &[],
                window: iced::window::Id::unique(),
            },
        );
    }

    #[test]
    fn renders_apptray_with_items() {
        use crate::tray::types::{TrayIcon, TrayItem};
        let rc = render_kdl("widget bar child=apptray\napptray icon_size=20");
        let w = rc.widgets.get("bar").unwrap();
        let items = vec![TrayItem {
            bus_name: ":1.1".into(),
            object_path: "/StatusNotifierItem".into(),
            id: "x".into(),
            title: "x".into(),
            status: "Active".into(),
            icon: TrayIcon::None,
            menu_path: None,
        }];
        let _el = view_widget(
            w,
            &RenderCtx {
                tray: &items,
                window: iced::window::Id::unique(),
            },
        );
    }
}
