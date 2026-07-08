use iced::alignment::{Horizontal, Vertical};
use iced::{Color, Padding, border::Radius};
use iced_layershell::reexport::{Anchor, Layer, OutputOption};
use indexmap::IndexMap;
use miette::SourceSpan;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SourceText {
    pub label: Arc<str>,
    pub text: Arc<str>,
}

#[derive(Debug, Clone)]
pub struct Span {
    pub source: SourceText,
    pub span: SourceSpan,
}

impl Span {
    pub fn line_col(&self) -> (usize, usize) {
        let mut line = 1usize;
        let mut col = 1usize;
        let text = self.source.text.as_ref();
        let bytes = text.as_bytes();
        let offset = self.span.offset().min(bytes.len());
        for ch in text[..offset].chars() {
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }
}

#[derive(Debug, Clone)]
pub enum FieldValue<T> {
    Literal(T),
    Expr(String),
}

#[derive(Debug, Clone)]
pub enum VarValue {
    Int(i128),
    Float(f64),
    Bool(bool),
    Str(String),
}

#[derive(Debug, Clone, Default)]
pub struct ParsedConfig {
    pub vars: IndexMap<String, VarDecl>,
    pub widgets: IndexMap<String, Widget>,
    pub containers: IndexMap<String, Container>,
    pub buttons: IndexMap<String, Button>,
    pub rows: IndexMap<String, Row>,
    pub columns: IndexMap<String, Column>,
    pub texts: IndexMap<String, TextEl>,
    pub styles: IndexMap<String, Style>,
    pub borders: IndexMap<String, Border>,
    pub shadows: IndexMap<String, Shadow>,
    pub fonts: IndexMap<String, iced::Font>,
    pub notification: Option<NotificationSettings>,
    pub apptray: Option<ApptraySettings>,
    pub pulls: IndexMap<String, PullDecl>,
    pub icon_theme: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PullDecl {
    pub command: String,
    pub interval: std::time::Duration,
    pub default: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub value: VarValue,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Widget {
    pub h: Option<FieldValue<f32>>,
    pub w: Option<FieldValue<f32>>,
    pub layer: Option<FieldValue<Layer>>,
    pub anchor: Option<FieldValue<Anchor>>,
    pub exclusive: Option<FieldValue<bool>>,
    pub margin: Option<FieldValue<(f32, f32, f32, f32)>>,
    pub output: Option<FieldValue<OutputOption>>,
    pub keyboard: Option<FieldValue<bool>>,
    pub transparent: Option<FieldValue<bool>>,
    pub child: Option<FieldValue<String>>,
    pub span: Span,
}
#[derive(Debug, Clone)]
pub struct Container {
    pub w: Option<FieldValue<iced::Length>>,
    pub h: Option<FieldValue<iced::Length>>,
    pub padding: Option<FieldValue<Padding>>,
    pub align_x: Option<FieldValue<Horizontal>>,
    pub align_y: Option<FieldValue<Vertical>>,
    pub clip: Option<FieldValue<bool>>,
    pub style: Option<FieldValue<String>>,
    pub child: Option<FieldValue<String>>,
    pub span: Span,
}
#[derive(Debug, Clone)]
pub struct Button {
    pub w: Option<FieldValue<iced::Length>>,
    pub h: Option<FieldValue<iced::Length>>,
    pub padding: Option<FieldValue<Padding>>,
    pub action: Option<FieldValue<String>>,
    pub clip: Option<FieldValue<bool>>,
    pub style: Option<FieldValue<String>>,
    pub style_hover: Option<FieldValue<String>>,
    pub style_active: Option<FieldValue<String>>,
    pub style_disabled: Option<FieldValue<String>>,
    pub text: Option<FieldValue<String>>,
    pub font: Option<FieldValue<String>>,
    pub span: Span,
}
#[derive(Debug, Clone)]
pub struct Row {
    pub children: Option<FieldValue<Vec<String>>>,
    pub w: Option<FieldValue<iced::Length>>,
    pub h: Option<FieldValue<iced::Length>>,
    pub padding: Option<FieldValue<Padding>>,
    pub spacing: Option<FieldValue<f32>>,
    pub clip: Option<FieldValue<bool>>,
    pub align: Option<FieldValue<Vertical>>,
    pub span: Span,
}
#[derive(Debug, Clone)]
pub struct Column {
    pub children: Option<FieldValue<Vec<String>>>,
    pub w: Option<FieldValue<iced::Length>>,
    pub h: Option<FieldValue<iced::Length>>,
    pub padding: Option<FieldValue<Padding>>,
    pub spacing: Option<FieldValue<f32>>,
    pub clip: Option<FieldValue<bool>>,
    pub align: Option<FieldValue<Horizontal>>,
    pub span: Span,
}
#[derive(Debug, Clone)]
pub struct TextEl {
    pub w: Option<FieldValue<iced::Length>>,
    pub h: Option<FieldValue<iced::Length>>,
    pub align_x: Option<FieldValue<iced::advanced::text::Alignment>>,
    pub align_y: Option<FieldValue<Vertical>>,
    pub color: Option<FieldValue<Color>>,
    pub font: Option<FieldValue<String>>,
    pub content: Option<FieldValue<String>>,
    pub span: Span,
}
#[derive(Debug, Clone)]
pub struct Style {
    pub text: Option<FieldValue<Color>>,
    pub bg: Option<FieldValue<Color>>,
    pub border: Option<FieldValue<String>>,
    pub shadow: Option<FieldValue<String>>,
    pub snap: Option<FieldValue<bool>>,
    pub span: Span,
}
#[derive(Debug, Clone)]
pub struct Border {
    pub color: Option<FieldValue<Color>>,
    pub w: Option<FieldValue<f32>>,
    pub radius: Option<FieldValue<Radius>>,
    pub span: Span,
}
#[derive(Debug, Clone)]
pub struct Shadow {
    pub color: Option<FieldValue<Color>>,
    pub offset: Option<FieldValue<(f32, f32)>>,
    pub blur_radius: Option<FieldValue<f32>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ApptraySettings {
    pub icon_size: Option<FieldValue<f32>>,
    pub spacing: Option<FieldValue<f32>>,
    pub padding: Option<FieldValue<Padding>>,
    pub bg: Option<FieldValue<Color>>,
    pub border: Option<FieldValue<String>>,
    pub swap_buttons: Option<FieldValue<bool>>,
    pub vertical: Option<FieldValue<bool>>,
    pub menu_bg: Option<FieldValue<Color>>,
    pub menu_text: Option<FieldValue<Color>>,
    pub menu_disabled: Option<FieldValue<Color>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct NotificationSettings {
    pub width: Option<FieldValue<f32>>,
    pub height: Option<FieldValue<f32>>,
    pub primary_text: Option<FieldValue<Color>>,
    pub secondary_text: Option<FieldValue<Color>>,
    pub bg: Option<FieldValue<Color>>,
    pub border: Option<FieldValue<String>>,
    pub font: Option<FieldValue<String>>,
    pub anchor: Option<FieldValue<Anchor>>,
    pub margin: Option<FieldValue<(f32, f32, f32, f32)>>,
    pub gap: Option<FieldValue<f32>>,
    pub max: Option<FieldValue<f32>>,
    pub timeout: Option<FieldValue<f32>>,
    pub layer: Option<FieldValue<Layer>>,
    pub respect_notification_icon: Option<FieldValue<bool>>,
    pub freeze_on_hover: Option<FieldValue<bool>>,
    pub span: Span,
}
