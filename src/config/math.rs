pub mod ast;
pub mod error;
pub mod evaluator;
pub mod interpolation;
pub mod lexer;
pub mod parser;
pub mod value;

pub use ast::Expr;
pub use error::{EvalError, EvalErrorKind};
pub use value::{Decimal, Value, VarStore};

use crate::config::types::{SourceText, Span};

pub fn evaluate(
    input: &str,
    vars: &dyn VarStore,
    source: &SourceText,
    base_offset: usize,
) -> Result<Value, EvalError> {
    let segs = interpolation::segments(input).map_err(|e| EvalError {
        kind: match e.kind {
            interpolation::InterpErrorKind::Unterminated => {
                EvalErrorKind::UnterminatedInterpolation
            }
            interpolation::InterpErrorKind::Nested => EvalErrorKind::NestedInterpolation,
        },
        span: span_at(source, base_offset + e.offset),
        message: match e.kind {
            interpolation::InterpErrorKind::Unterminated => "unterminated ${...} expression".into(),
            interpolation::InterpErrorKind::Nested => {
                "nested ${...} interpolation is not allowed".into()
            }
        },
    })?;

    if segs.len() == 1
        && let interpolation::Segment::Expr { text, offset } = segs[0]
    {
        return eval_one(text, base_offset + offset, vars, source);
    }

    let mut out = String::new();
    for seg in segs {
        match seg {
            interpolation::Segment::Literal(s) => out.push_str(s),
            interpolation::Segment::Expr { text, offset } => {
                let v = eval_one(text, base_offset + offset, vars, source)?;
                out.push_str(&format_value(&v));
            }
        }
    }
    Ok(Value::Str(out))
}

fn eval_one(
    text: &str,
    abs_offset: usize,
    vars: &dyn VarStore,
    source: &SourceText,
) -> Result<Value, EvalError> {
    let tokens = lexer::Lexer::new(text).tokenize().map_err(|e| EvalError {
        kind: EvalErrorKind::LexError,
        span: span_at(source, abs_offset + e.offset),
        message: format!("unexpected character '{}' in expression", e.ch),
    })?;
    let ast = parser::parse(&tokens).map_err(|e| {
        let kind = match e.kind {
            parser::ParseErrorKind::UnknownFunction => EvalErrorKind::UnknownFunction,
            parser::ParseErrorKind::Generic => EvalErrorKind::ParseError,
        };
        EvalError {
            kind,
            span: span_at(source, abs_offset + e.offset),
            message: e.message,
        }
    })?;
    evaluator::eval(&ast, vars).map_err(|mut e| {
        e.span = span_at(source, abs_offset);
        e
    })
}

fn format_value(v: &Value) -> String {
    match v {
        Value::Int(i) => i.to_string(),
        Value::Float(Decimal {
            value,
            precision: Some(n),
        }) => format!("{:.1$}", value, *n as usize),
        Value::Float(Decimal {
            value,
            precision: None,
        }) => format!("{}", value),
        Value::Bool(b) => format!("#{}", b),
        Value::Str(s) => s.clone(),
    }
}

fn span_at(source: &SourceText, offset: usize) -> Span {
    Span {
        source: source.clone(),
        span: miette::SourceSpan::new(offset.into(), 0),
    }
}

#[cfg(test)]
mod facade_tests {
    use super::*;
    use crate::config::types::{SourceText, Span, VarDecl, VarValue};
    use indexmap::IndexMap;
    use std::sync::Arc;

    fn span_of(s: &str) -> SourceText {
        SourceText {
            label: Arc::from("<t>"),
            text: Arc::from(s),
        }
    }

    fn store(pairs: &[(&str, VarValue)]) -> IndexMap<String, VarDecl> {
        let mut m = IndexMap::new();
        let blank_span = Span {
            source: span_of(""),
            span: miette::SourceSpan::new(0.into(), 0),
        };
        for (name, value) in pairs {
            m.insert(
                name.to_string(),
                VarDecl {
                    value: value.clone(),
                    span: blank_span.clone(),
                },
            );
        }
        m
    }

    fn run(input: &str, vars: &IndexMap<String, VarDecl>) -> Value {
        let src = span_of(input);
        evaluate(input, vars, &src, 0).unwrap()
    }

    #[test]
    fn pure_int_expr() {
        let v = run("${x*2}", &store(&[("x", VarValue::Int(3))]));
        assert_eq!(v, Value::Int(6));
    }

    #[test]
    fn interpolation_returns_string() {
        let v = run("Result: ${x*2} items", &store(&[("x", VarValue::Int(3))]));
        assert_eq!(v, Value::Str("Result: 6 items".into()));
    }

    #[test]
    fn pure_string_passthrough() {
        let v = run("${s}", &store(&[("s", VarValue::Str("xfdf".into()))]));
        assert_eq!(v, Value::Str("xfdf".into()));
    }

    #[test]
    fn format_int_in_interpolation() {
        let v = run("x=${x}", &store(&[("x", VarValue::Int(42))]));
        assert_eq!(v, Value::Str("x=42".into()));
    }

    #[test]
    fn format_float_tagged_in_interpolation() {
        let v = run("${round(x).1}!", &store(&[("x", VarValue::Float(3.12))]));
        assert_eq!(v, Value::Str("3.1!".into()));
    }

    #[test]
    fn math_md_contract() {
        enum VarValueLit {
            Int(i128),
            Float(f64),
            Str(&'static str),
        }
        enum Expect {
            Int(i128),
            FloatTagged { value: f64, prec: u8 },
            Str(&'static str),
            ErrMsg(&'static str),
        }
        struct Case {
            label: &'static str,
            input: &'static str,
            vars: &'static [(&'static str, VarValueLit)],
            expect: Expect,
        }

        let cases: &[Case] = &[
            Case {
                label: "test1: 3/2 -> 1",
                input: "${x/2}",
                vars: &[("x", VarValueLit::Int(3))],
                expect: Expect::Int(1),
            },
            Case {
                label: "test2: 5/4 -> 1",
                input: "${x/4}",
                vars: &[("x", VarValueLit::Int(5))],
                expect: Expect::Int(1),
            },
            Case {
                label: "test3: string passthrough",
                input: "${s}",
                vars: &[("s", VarValueLit::Str("xfdf"))],
                expect: Expect::Str("xfdf"),
            },
            Case {
                label: "test4: math on string errors",
                input: "${s/2}",
                vars: &[("s", VarValueLit::Str("xfdf"))],
                expect: Expect::ErrMsg("math ops cannot be applied to strings"),
            },
            Case {
                label: "test5: 3.12 * 2 -> 6.24 (no precision tag)",
                input: "${x*2}",
                vars: &[("x", VarValueLit::Float(3.12))],
                expect: Expect::FloatTagged {
                    value: 6.24,
                    prec: u8::MAX,
                },
            },
            Case {
                label: "test6: 3*2/3 -> 2",
                input: "${x*2/3}",
                vars: &[("x", VarValueLit::Int(3))],
                expect: Expect::Int(2),
            },
            Case {
                label: "test7: 3+3*2 -> 9",
                input: "${x+3*2}",
                vars: &[("x", VarValueLit::Int(3))],
                expect: Expect::Int(9),
            },
            Case {
                label: "test8: (3+3)*2 -> 12",
                input: "${(x+3)*2}",
                vars: &[("x", VarValueLit::Int(3))],
                expect: Expect::Int(12),
            },
            Case {
                label: "test9: 3^2 -> 9",
                input: "${x^2}",
                vars: &[("x", VarValueLit::Int(3))],
                expect: Expect::Int(9),
            },
            Case {
                label: "test10 (corrected): 3%2 -> 1",
                input: "${x%2}",
                vars: &[("x", VarValueLit::Int(3))],
                expect: Expect::Int(1),
            },
            Case {
                label: "test11: 'Result: ${x*2} items' -> 'Result: 6 items'",
                input: "Result: ${x*2} items",
                vars: &[("x", VarValueLit::Int(3))],
                expect: Expect::Str("Result: 6 items"),
            },
            Case {
                label: "test12: round(3.12).1 -> 3.1",
                input: "${round(x).1}",
                vars: &[("x", VarValueLit::Float(3.12))],
                expect: Expect::FloatTagged {
                    value: 3.1,
                    prec: 1,
                },
            },
            Case {
                label: "test13 (corrected): round(3.46).1*2 -> 7.0",
                input: "${round(x).1*2}",
                vars: &[("x", VarValueLit::Float(3.46))],
                expect: Expect::FloatTagged {
                    value: 7.0,
                    prec: 1,
                },
            },
            Case {
                label: "test14: round(3.46*2).1 -> 6.9",
                input: "${round(x*2).1}",
                vars: &[("x", VarValueLit::Float(3.46))],
                expect: Expect::FloatTagged {
                    value: 6.9,
                    prec: 1,
                },
            },
            Case {
                label: "test15: round(round(3.46).1*2).0 -> 7",
                input: "${round(round(x).1*2).0}",
                vars: &[("x", VarValueLit::Float(3.46))],
                expect: Expect::FloatTagged {
                    value: 7.0,
                    prec: 0,
                },
            },
        ];

        for c in cases {
            let mut s = IndexMap::new();
            let blank = Span {
                source: span_of(""),
                span: miette::SourceSpan::new(0.into(), 0),
            };
            for (name, v) in c.vars {
                let value = match v {
                    VarValueLit::Int(i) => VarValue::Int(*i),
                    VarValueLit::Float(f) => VarValue::Float(*f),
                    VarValueLit::Str(s) => VarValue::Str((*s).into()),
                };
                s.insert(
                    name.to_string(),
                    VarDecl {
                        value,
                        span: blank.clone(),
                    },
                );
            }
            let src = span_of(c.input);
            let res = evaluate(c.input, &s, &src, 0);
            match (&c.expect, &res) {
                (Expect::Int(n), Ok(Value::Int(got))) => {
                    assert_eq!(got, n, "case `{}`: got {:?}", c.label, got)
                }
                (Expect::FloatTagged { value, prec }, Ok(Value::Float(d))) => {
                    assert!(
                        (d.value - value).abs() < 1e-9,
                        "case `{}`: value {} != expected {}",
                        c.label,
                        d.value,
                        value
                    );
                    if *prec != u8::MAX {
                        assert_eq!(
                            d.precision,
                            Some(*prec),
                            "case `{}`: precision {:?} != Some({})",
                            c.label,
                            d.precision,
                            prec
                        );
                    }
                }
                (Expect::Str(s), Ok(Value::Str(got))) => {
                    assert_eq!(got, s, "case `{}`: got {:?}", c.label, got)
                }
                (Expect::ErrMsg(msg), Err(e)) => assert_eq!(
                    &e.message, msg,
                    "case `{}`: got message {:?}",
                    c.label, e.message
                ),
                _ => panic!(
                    "case `{}`: mismatch: expected one shape, got {:?}",
                    c.label, res
                ),
            }
        }
    }
}
