use crate::config::math::value::Value;
use crate::config::primitives;
use crate::config::types::{
    AlignX, AlignY, Anchor, ColAlign, Color, Layer, Output, RowAlign, Span,
};
use crate::config::{ConfigError, ConfigErrorKind, Severity};

fn type_err(field: &str, expected: &str, span: &Span) -> ConfigError {
    ConfigError {
        kind: ConfigErrorKind::TypeCoercion,
        span: span.clone(),
        message: format!(
            "cannot use this value where {} is expected for field \"{}\"",
            expected, field
        ),
        severity: Severity::Error,
    }
}

pub fn coerce_f32(v: Value, field: &str, span: &Span) -> Result<f32, ConfigError> {
    match v {
        Value::Int(i) => Ok(i as f32),
        Value::Float(d) => Ok(d.value as f32),
        _ => Err(type_err(field, "a number", span)),
    }
}

pub fn coerce_length(v: Value, field: &str, span: &Span) -> Result<iced::Length, ConfigError> {
    match v {
        Value::Int(i) => Ok(iced::Length::Fixed(i as f32)),
        Value::Float(d) => Ok(iced::Length::Fixed(d.value as f32)),
        _ => Err(type_err(field, "a number", span)),
    }
}

pub fn coerce_margin(v: Value, field: &str, span: &Span) -> Result<(f32, f32, f32, f32), ConfigError> {
    match v {
        Value::Int(i) => {
            let f = i as f32;
            Ok((f, f, f, f))
        }
        Value::Float(d) => {
            let f = d.value as f32;
            Ok((f, f, f, f))
        }
        _ => Err(type_err(field, "a number", span)),
    }
}

pub fn coerce_padding(v: Value, field: &str, span: &Span) -> Result<iced::Padding, ConfigError> {
    match v {
        Value::Int(i) => Ok(iced::Padding::from(i as f32)),
        Value::Float(d) => Ok(iced::Padding::from(d.value as f32)),
        _ => Err(type_err(field, "a number", span)),
    }
}

pub fn coerce_radius(v: Value, field: &str, span: &Span) -> Result<iced::border::Radius, ConfigError> {
    match v {
        Value::Int(i) => Ok(iced::border::Radius::from(i as f32)),
        Value::Float(d) => Ok(iced::border::Radius::from(d.value as f32)),
        _ => Err(type_err(field, "a number", span)),
    }
}

pub fn coerce_bool(v: Value, field: &str, span: &Span) -> Result<bool, ConfigError> {
    match v {
        Value::Bool(b) => Ok(b),
        _ => Err(type_err(field, "a boolean", span)),
    }
}

pub fn coerce_color(v: Value, field: &str, span: &Span) -> Result<Color, ConfigError> {
    match v {
        Value::Str(s) => {
            primitives::parse_color(&s).ok_or_else(|| type_err(field, "a color", span))
        }
        _ => Err(type_err(field, "a color string", span)),
    }
}

pub fn coerce_anchor(v: Value, field: &str, span: &Span) -> Result<Anchor, ConfigError> {
    match v {
        Value::Str(s) => {
            primitives::parse_anchor(&s).map_err(|_| type_err(field, "a valid anchor", span))
        }
        _ => Err(type_err(field, "an anchor string", span)),
    }
}

pub fn coerce_layer(v: Value, field: &str, span: &Span) -> Result<Layer, ConfigError> {
    match v {
        Value::Str(s) => {
            primitives::parse_layer(&s).ok_or_else(|| type_err(field, "a valid layer", span))
        }
        _ => Err(type_err(field, "a layer string", span)),
    }
}

pub fn coerce_align_x(v: Value, field: &str, span: &Span) -> Result<AlignX, ConfigError> {
    match v {
        Value::Str(s) => {
            primitives::parse_align_x(&s).ok_or_else(|| type_err(field, "a valid align_x", span))
        }
        _ => Err(type_err(field, "an align_x string", span)),
    }
}

pub fn coerce_align_y(v: Value, field: &str, span: &Span) -> Result<AlignY, ConfigError> {
    match v {
        Value::Str(s) => {
            primitives::parse_align_y(&s).ok_or_else(|| type_err(field, "a valid align_y", span))
        }
        _ => Err(type_err(field, "an align_y string", span)),
    }
}

pub fn coerce_output(v: Value, field: &str, span: &Span) -> Result<Output, ConfigError> {
    match v {
        Value::Str(s) => Ok(primitives::parse_output(&s)),
        _ => Err(type_err(field, "an output string", span)),
    }
}

pub fn coerce_row_align(v: Value, field: &str, span: &Span) -> Result<RowAlign, ConfigError> {
    match v {
        Value::Str(s) => primitives::parse_row_align(&s)
            .ok_or_else(|| type_err(field, "a valid row align", span)),
        _ => Err(type_err(field, "a row align string", span)),
    }
}

pub fn coerce_col_align(v: Value, field: &str, span: &Span) -> Result<ColAlign, ConfigError> {
    match v {
        Value::Str(s) => primitives::parse_col_align(&s)
            .ok_or_else(|| type_err(field, "a valid column align", span)),
        _ => Err(type_err(field, "a column align string", span)),
    }
}

pub fn coerce_string(v: Value, _field: &str, _span: &Span) -> Result<String, ConfigError> {
    Ok(match v {
        Value::Int(i) => i.to_string(),
        Value::Float(d) => format!("{}", d.value),
        Value::Bool(b) => format!("#{}", b),
        Value::Str(s) => s,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::math::value::{Decimal, Value};
    use crate::config::types::{SourceText, Span};
    use std::sync::Arc;

    fn span() -> Span {
        Span {
            source: SourceText {
                label: Arc::from("<t>"),
                text: Arc::from(""),
            },
            span: miette::SourceSpan::new(0.into(), 0),
        }
    }
    fn int(n: i128) -> Value {
        Value::Int(n)
    }
    fn float(f: f64) -> Value {
        Value::Float(Decimal::new(f))
    }

    #[test]
    fn f32_from_int_and_float() {
        assert_eq!(coerce_f32(int(40), "h", &span()).unwrap(), 40.0);
        assert_eq!(coerce_f32(float(2.5), "h", &span()).unwrap(), 2.5);
    }
    #[test]
    fn f32_rejects_string() {
        assert!(coerce_f32(Value::Str("x".into()), "h", &span()).is_err());
    }
    #[test]
    fn length_from_number() {
        assert!(
            matches!(coerce_length(int(200), "w", &span()).unwrap(), iced::Length::Fixed(v) if v == 200.0)
        );
    }
    #[test]
    fn bool_strict() {
        assert!(coerce_bool(Value::Bool(true), "clip", &span()).unwrap());
        assert!(coerce_bool(int(1), "clip", &span()).is_err());
    }
    #[test]
    fn color_from_string() {
        let c = coerce_color(Value::Str("ffffff".into()), "text", &span()).unwrap();
        assert_eq!(
            c,
            crate::config::types::Color {
                r: 0xff,
                g: 0xff,
                b: 0xff,
                a: 0xff
            }
        );
    }
    #[test]
    fn color_rejects_number() {
        assert!(coerce_color(int(0), "text", &span()).is_err());
    }
    #[test]
    fn anchor_from_string() {
        let a = coerce_anchor(Value::Str("t | r".into()), "anchor", &span()).unwrap();
        assert_eq!(
            a,
            crate::config::types::Anchor {
                top: true,
                bottom: false,
                left: false,
                right: true
            }
        );
    }
    #[test]
    fn layer_from_string() {
        assert_eq!(
            coerce_layer(Value::Str("top".into()), "layer", &span()).unwrap(),
            crate::config::types::Layer::Top
        );
    }
    #[test]
    fn string_stringifies_any() {
        assert_eq!(coerce_string(int(42), "text", &span()).unwrap(), "42");
        assert_eq!(
            coerce_string(Value::Bool(true), "text", &span()).unwrap(),
            "#true"
        );
        assert_eq!(
            coerce_string(Value::Str("hi".into()), "text", &span()).unwrap(),
            "hi"
        );
    }
}
