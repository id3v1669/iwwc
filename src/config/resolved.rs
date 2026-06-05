use crate::config::types::{
    AlignX, AlignY, Anchor, ColAlign, Layer, Output, RowAlign, Span,
};
use iced::{Color,Padding,border::Radius};
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
    pub margin: Option<(f32,f32,f32,f32)>,
    pub output: Option<Output>,
    pub keyboard: Option<bool>,
    pub transparent: Option<bool>,
    pub child: Option<Box<ResolvedElement>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ResolvedElement {
    Container(ResolvedContainer),
    Button(Box<ResolvedButton>),
    Row(ResolvedRow),
    Column(ResolvedColumn),
    Text(ResolvedText),
    Apptray(ResolvedApptraySettings),
}

#[derive(Debug, Clone)]
pub struct ResolvedContainer {
    pub w: Option<iced::Length>,
    pub h: Option<iced::Length>,
    pub padding: Option<Padding>,
    pub align_x: Option<AlignX>,
    pub align_y: Option<AlignY>,
    pub clip: Option<bool>,
    pub style: Option<ResolvedStyle>,
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
    pub style: Option<ResolvedStyle>,
    pub style_hover: Option<ResolvedStyle>,
    pub style_active: Option<ResolvedStyle>,
    pub style_disabled: Option<ResolvedStyle>,
    pub text: Option<String>,
    pub font: Option<String>,
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
    pub align: Option<RowAlign>,
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
    pub align: Option<ColAlign>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ResolvedText {
    pub w: Option<iced::Length>,
    pub h: Option<iced::Length>,
    pub align_x: Option<AlignX>,
    pub align_y: Option<AlignY>,
    pub color: Option<Color>,
    pub font: Option<String>,
    pub content: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, Default)]
pub struct ResolvedStyle {
    pub text: Option<Color>,
    pub bg: Option<Color>,
    pub border: Option<Border>,
    pub shadow: Option<ResolvedShadow>,
    pub snap: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct ResolvedShadow {
    pub color: Option<Color>,
    pub offset: Option<(f32, f32)>,
    pub blur_radius: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct ResolvedMenu {
    pub font_name: Option<String>,
    pub font_size: f32,
    pub row_height: f32,
    pub icon_size: f32,
    pub row_spacing: f32,
    pub menu_container_padding: Padding,
    pub menu_container_style: Option<ResolvedStyle>,
    pub button_padding: Padding,
    pub button_style: Option<ResolvedStyle>,
    pub button_style_hover: Option<ResolvedStyle>,
    pub button_style_active: Option<ResolvedStyle>,
    pub button_style_disabled: Option<ResolvedStyle>,
}

impl Default for ResolvedMenu {
    fn default() -> Self {
        let white = Color { r: 0xff, g: 0xff, b: 0xff, a: 0xff };
        ResolvedMenu {
            font_name: None,
            font_size: 16.0,
            row_height: 26.0,
            icon_size: 16.0,
            row_spacing: 6.0,
            menu_container_padding: Padding::from(10),
            menu_container_style: Some(ResolvedStyle {
                bg: Some(Color { r: 0x22, g: 0x22, b: 0x22, a: 0xff }),
                ..Default::default()
            }),
            button_padding: Padding::from([5, 2]),
            button_style: Some(ResolvedStyle {
                text: Some(white),
                ..Default::default()
            }),
            button_style_hover: Some(ResolvedStyle {
                text: Some(white),
                bg: Some(Color { r: 0x3a, g: 0x3a, b: 0x3a, a: 0xff }),
                ..Default::default()
            }),
            button_style_active: Some(ResolvedStyle {
                text: Some(white),
                bg: Some(Color { r: 0x50, g: 0x50, b: 0x50, a: 0xff }),
                ..Default::default()
            }),
            button_style_disabled: Some(ResolvedStyle {
                text: Some(Color::from_str("#bdae93").unwrap()),
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
    pub border: Border,
    pub font: Option<String>,
    pub anchor: Anchor,
    pub margin: (f32,f32,f32,f32),
    pub gap: f32,
    pub max: u32,
    pub timeout_ms: i32,
    pub layer: Layer,
    pub respect_icon: bool,
}

impl Default for ResolvedNotificationSettings {
    fn default() -> Self {
        ResolvedNotificationSettings {
            width: 400.0,
            height: 110.0,
            primary_text: Color::from_str("#e7d4a2").unwrap(),
            secondary_text: Color::from_str("#e3cd92").unwrap(),
            bg: Color::from_str("#3c3836").unwrap(),
            border: Border {
                color: Color::from_str("#d65d0e").unwrap(),
                width: 2.0,
                radius: Radius::from(10.0)
            },
            font: None,
            anchor: Anchor {
                top: true,
                bottom: false,
                left: false,
                right: true,
            },
            margin: (12.0, 12.0, 12.0, 12.0),
            gap: 8.0,
            max: 5,
            timeout_ms: 5000,
            layer: Layer::Overlay,
            respect_icon: true,
        }
    }
}
