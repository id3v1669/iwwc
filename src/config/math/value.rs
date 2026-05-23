use crate::config::types::VarValue;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i128),
    Float(Decimal),
    Bool(bool),
    Str(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Decimal {
    pub value: f64,
    pub precision: Option<u8>,
}

impl Decimal {
    pub const fn new(value: f64) -> Self {
        Self {
            value,
            precision: None,
        }
    }
    pub const fn with_precision(value: f64, precision: u8) -> Self {
        Self {
            value,
            precision: Some(precision),
        }
    }
}

pub trait VarStore {
    fn lookup(&self, name: &str) -> Option<&VarValue>;
}

impl VarStore for indexmap::IndexMap<String, crate::config::types::VarDecl> {
    fn lookup(&self, name: &str) -> Option<&VarValue> {
        self.get(name).map(|d| &d.value)
    }
}
