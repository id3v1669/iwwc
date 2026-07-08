use crate::config::types::Span;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, container};
use iced::{Background, Border, Color, Font, Padding, Shadow, border::Radius};
use iced_layershell::reexport::{Anchor, Layer, OutputOption};
use indexmap::IndexMap;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub widgets: IndexMap<String, ResolvedWidget>,
    pub notification: ResolvedNotificationSettings,
    pub apptray: ResolvedApptraySettings,
    pub smart_polls: Vec<(String, std::time::Duration)>,
    pub icon_theme: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedWidget {
    pub h: Option<f32>,
    pub w: Option<f32>,
    pub layer: Option<Layer>,
    pub anchor: Option<Anchor>,
    pub exclusive: Option<bool>,
    pub margin: Option<(f32, f32, f32, f32)>,
    pub output: OutputOption,
    pub keyboard: Option<bool>,
    pub transparent: Option<bool>,
    pub child: Option<Box<ResolvedElement>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ResolvedElement {
    Container(Box<ResolvedContainer>),
    Button(Box<ResolvedButton>),
    Row(ResolvedRow),
    Column(ResolvedColumn),
    Text(ResolvedText),
    Apptray(Box<ResolvedApptraySettings>),
}

#[derive(Debug, Clone)]
pub struct ResolvedContainer {
    pub w: Option<iced::Length>,
    pub h: Option<iced::Length>,
    pub padding: Option<Padding>,
    pub align_x: Option<Horizontal>,
    pub align_y: Option<Vertical>,
    pub clip: Option<bool>,
    pub style: Option<container::Style>,
    pub child: Box<ResolvedElement>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ResolvedButton {
    pub w: Option<iced::Length>,
    pub h: Option<iced::Length>,
    pub padding: Option<Padding>,
    pub action: Option<String>,
    pub clip: Option<bool>,
    pub style: Option<button::Style>,
    pub style_hover: Option<button::Style>,
    pub style_active: Option<button::Style>,
    pub style_disabled: Option<button::Style>,
    pub text: Option<String>,
    pub font: Option<Font>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ResolvedRow {
    pub children: Vec<ResolvedElement>,
    pub w: Option<iced::Length>,
    pub h: Option<iced::Length>,
    pub padding: Option<Padding>,
    pub spacing: Option<f32>,
    pub clip: Option<bool>,
    pub align: Option<Vertical>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ResolvedColumn {
    pub children: Vec<ResolvedElement>,
    pub w: Option<iced::Length>,
    pub h: Option<iced::Length>,
    pub padding: Option<Padding>,
    pub spacing: Option<f32>,
    pub clip: Option<bool>,
    pub align: Option<Horizontal>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ResolvedText {
    pub w: Option<iced::Length>,
    pub h: Option<iced::Length>,
    pub align_x: Option<iced::advanced::text::Alignment>,
    pub align_y: Option<Vertical>,
    pub color: Option<Color>,
    pub font: Option<Font>,
    pub content: Option<String>,
    pub span: Span,
}

// Struct stays acts as universal layer between final Style and cfg
#[derive(Debug, Clone, Default)]
pub struct PreResolvedStyle {
    pub text: Option<Color>,
    pub bg: Option<Color>,
    pub border: Option<Border>,
    pub shadow: Option<Shadow>,
    pub snap: Option<bool>,
}

impl PreResolvedStyle {
    pub fn to_container(&self) -> container::Style {
        container::Style {
            text_color: self.text,
            background: self.bg.map(Background::Color),
            border: self.border.unwrap_or_default(),
            shadow: self.shadow.unwrap_or_default(),
            snap: self.snap.unwrap_or_default(),
        }
    }

    pub fn to_button(&self) -> button::Style {
        button::Style {
            background: self.bg.map(Background::Color),
            text_color: self.text.unwrap_or(Color::BLACK),
            border: self.border.unwrap_or_default(),
            shadow: self.shadow.unwrap_or_default(),
            snap: self.snap.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedMenu {
    pub font: Option<Font>,
    pub font_size: f32,
    pub icon_size: f32,
    pub row_spacing: f32,
    pub menu_container_padding: Padding,
    pub menu_container_style: Option<container::Style>,
    pub button_padding: Padding,
    pub button_style: Option<button::Style>,
    pub button_style_hover: Option<button::Style>,
    pub button_style_active: Option<button::Style>,
    pub button_style_disabled: Option<button::Style>,
}

impl Default for ResolvedMenu {
    fn default() -> Self {
        ResolvedMenu {
            font: None,
            font_size: 16.0,
            icon_size: 16.0,
            row_spacing: 6.0,
            menu_container_padding: Padding::from(10.0),
            menu_container_style: Some(container::Style {
                background: Some(Background::Color(Color::from_str("3c3836").unwrap())),
                border: Border {
                    width: 0.0,
                    radius: Radius::from(15.0),
                    ..Default::default()
                },
                shadow: Shadow {
                    color: Color::from_str("665c54").unwrap(),
                    blur_radius: 4.0,
                    ..Default::default()
                },
                ..Default::default()
            }),
            button_padding: Padding::from([5.0, 2.0]),
            button_style: Some(button::Style {
                background: Some(Background::Color(Color::from_str("3c3836").unwrap())),
                text_color: Color::from_str("bdae93").unwrap(),
                border: Border {
                    width: 0.0,
                    radius: Radius::from(15.0),
                    ..Default::default()
                },
                ..Default::default()
            }),
            button_style_hover: Some(button::Style {
                background: Some(Background::Color(Color::from_str("504945").unwrap())),
                text_color: Color::from_str("d65d0e").unwrap(),
                border: Border {
                    width: 0.0,
                    radius: Radius::from(15.0),
                    ..Default::default()
                },
                ..Default::default()
            }),
            button_style_active: Some(button::Style {
                background: Some(Background::Color(Color::from_str("504945").unwrap())),
                text_color: Color::from_str("fe8019").unwrap(),
                border: Border {
                    width: 0.0,
                    radius: Radius::from(15.0),
                    ..Default::default()
                },
                ..Default::default()
            }),
            button_style_disabled: Some(button::Style {
                background: Some(Background::Color(Color::from_str("3c3836").unwrap())),
                text_color: Color::from_str("bdae93").unwrap(),
                border: Border {
                    width: 0.0,
                    radius: Radius::from(15.0),
                    ..Default::default()
                },
                ..Default::default()
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedApptraySettings {
    pub icon_size: f32,
    pub spacing: f32,
    pub padding: Option<Padding>,
    pub bg: Option<Color>,
    pub border: Option<Border>,
    pub swap_buttons: bool,
    pub vertical: bool,
    pub menu: ResolvedMenu,
}

impl Default for ResolvedApptraySettings {
    fn default() -> Self {
        ResolvedApptraySettings {
            icon_size: 22.0,
            spacing: 4.0,
            padding: None,
            bg: None,
            border: None,
            swap_buttons: false,
            vertical: false,
            menu: ResolvedMenu::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedNotificationSettings {
    pub width: f32,
    pub height: f32,
    pub primary_text: Color,
    pub secondary_text: Color,
    pub bg: Color,
    pub border: Option<Border>,
    pub font: Option<Font>,
    pub anchor: Anchor,
    pub margin: (f32, f32, f32, f32),
    pub gap: f32,
    pub max: u32,
    pub timeout_ms: i32,
    pub layer: Layer,
    pub respect_icon: bool,
    pub freeze_on_hover: bool,
}

impl Default for ResolvedNotificationSettings {
    fn default() -> Self {
        ResolvedNotificationSettings {
            width: 400.0,
            height: 110.0,
            primary_text: Color::from_str("e7d4a2").unwrap(),
            secondary_text: Color::from_str("e3cd92").unwrap(),
            bg: Color::from_str("3c3836").unwrap(),
            border: Some(Border {
                color: Color::from_str("d65d0e").unwrap(),
                width: 2.0,
                radius: Radius::from(10.0),
            }),
            font: Some(Font::DEFAULT),
            anchor: Anchor::Top | Anchor::Right,
            margin: (12.0, 12.0, 12.0, 12.0),
            gap: 8.0,
            max: 5,
            timeout_ms: 5000,
            layer: Layer::Overlay,
            respect_icon: true,
            freeze_on_hover: true,
        }
    }
}
