use crate::config::types::Span;
use std::fmt;

#[derive(Debug, Clone)]
pub struct EvalError {
    pub kind: EvalErrorKind,
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalErrorKind {
    UnterminatedInterpolation,
    NestedInterpolation,
    LexError,
    ParseError,
    UnknownVariable,
    TypeMismatch,
    DivByZero,
    Overflow,
    MissingPrecision,
    WrongArity,
    UnknownFunction,
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (line, col) = self.span.line_col();
        write!(
            f,
            "{}:{}:{}: error: {}",
            self.span.source.label, line, col, self.message
        )
    }
}
