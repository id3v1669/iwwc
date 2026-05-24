use crate::config::types::{
    AlignX, AlignY, Anchor, ColAlign, Color, Edges, Layer, Output, RowAlign, Span,
};
use indexmap::IndexMap;

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
    pub margin: Option<Edges>,
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
    pub padding: Option<Edges>,
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
    pub padding: Option<Edges>,
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
    pub padding: Option<Edges>,
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
    pub padding: Option<Edges>,
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

#[derive(Debug, Clone)]
pub struct ResolvedStyle {
    pub text: Option<Color>,
    pub bg: Option<Color>,
    pub border: Option<ResolvedBorder>,
    pub shadow: Option<ResolvedShadow>,
    pub snap: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct ResolvedBorder {
    pub color: Option<Color>,
    pub w: Option<f32>,
    pub radius: Option<Edges>,
}

#[derive(Debug, Clone)]
pub struct ResolvedShadow {
    pub color: Option<Color>,
    pub offset: Option<(f32, f32)>,
    pub blur_radius: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct ResolvedApptraySettings {
    pub icon_size: f32,
    pub spacing: f32,
    pub padding: Option<Edges>,
    pub bg: Option<Color>,
    pub border: Option<ResolvedBorder>,
    pub swap_buttons: bool,
    pub menu_bg: Color,
    pub menu_text: Color,
    pub menu_disabled: Color,
    pub menu_width: f32,
    pub row_height: f32,
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
            menu_bg: Color {
                r: 0x22,
                g: 0x22,
                b: 0x22,
                a: 0xff,
            },
            menu_text: Color {
                r: 0xff,
                g: 0xff,
                b: 0xff,
                a: 0xff,
            },
            menu_disabled: Color {
                r: 0x88,
                g: 0x88,
                b: 0x88,
                a: 0xff,
            },
            menu_width: 220.0,
            row_height: 26.0,
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
    pub border: Option<ResolvedBorder>,
    pub font: Option<String>,
    pub anchor: Anchor,
    pub margin: Edges,
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
            primary_text: Color {
                r: 0xff,
                g: 0xff,
                b: 0xff,
                a: 0xff,
            },
            secondary_text: Color {
                r: 0xcc,
                g: 0xcc,
                b: 0xcc,
                a: 0xff,
            },
            bg: Color {
                r: 0x22,
                g: 0x22,
                b: 0x22,
                a: 0xff,
            },
            border: None,
            font: None,
            anchor: Anchor {
                top: true,
                bottom: false,
                left: false,
                right: true,
            },
            margin: Edges::all(12.0),
            gap: 8.0,
            max: 5,
            timeout_ms: 5000,
            layer: Layer::Overlay,
            respect_icon: true,
        }
    }
}
