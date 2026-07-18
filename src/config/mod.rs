pub mod math;
pub mod parser;
pub mod primitives;
pub mod resolved;
pub mod resolver;
pub mod smart;
pub mod store;
pub mod types;

pub use types::ParsedConfig;

use crate::config::types::Span;
use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigErrorKind {
    Syntax,
    UnknownNode,
    MissingRequiredField,
    InvalidEnumValue,
    InvalidColor,
    InvalidLengthValue,
    InvalidPaddingArity,
    InvalidMarginArity,
    InvalidRadiusArity,
    InvalidOffsetArity,
    AnchorConflict,
    AnchorTooMany,
    EmptyChildrenList,
    DuplicateVariable,
    DuplicateElement,
    InvalidBool,
    InvalidFieldType,
    PortionMissingInt,
    VariableMissingValue,
    Expression,
    UnresolvedReference,
    CircularReference,
    TypeCoercion,
    UnusedElement,
    UnusedVariable,
    MissingSizeAnchor,
    Import,
}

#[derive(Debug, Clone)]
pub struct ConfigError {
    pub kind: ConfigErrorKind,
    pub span: Span,
    pub message: String,
    pub severity: Severity,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (line, col) = self.span.line_col();
        let tag = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(
            f,
            "{}:{}:{}: {}: {}",
            self.span.source.label, line, col, tag, self.message
        )
    }
}

#[derive(Debug)]
pub enum LoadError {
    PathDiscovery(String),
    Io(io::Error, PathBuf),
    Syntax(ConfigError),
    Semantic(Vec<ConfigError>),
}

#[derive(Debug)]
pub struct LoadOk {
    pub config: ParsedConfig,
    pub warnings: Vec<ConfigError>,
}

use crate::config::types::SourceText;
use std::fs;
use std::path::Path;
use std::sync::Arc;

pub(crate) fn parse_into(
    input: &str,
    source_label: &str,
    base_dir: Option<&Path>,
    visited: &mut Vec<PathBuf>,
    out: &mut ParsedConfig,
    errs: &mut Vec<ConfigError>,
) {
    let source = SourceText {
        label: Arc::from(source_label),
        text: Arc::from(input),
    };

    match input.parse::<kdl::KdlDocument>() {
        Ok(doc) => parser::parse_document_into(&doc, &source, base_dir, visited, out, errs),
        Err(e) => errs.push(ConfigError {
            kind: ConfigErrorKind::Syntax,
            span: crate::config::types::Span {
                source: source.clone(),
                span: miette::SourceSpan::new(0.into(), 0),
            },
            message: format!("KDL syntax error: {}", e),
            severity: Severity::Error,
        }),
    }
}

pub fn parse_str(input: &str, source_label: &str) -> (Option<ParsedConfig>, Vec<ConfigError>) {
    let mut cfg = ParsedConfig::default();
    let mut errs = Vec::new();
    parse_into(
        input,
        source_label,
        None,
        &mut Vec::new(),
        &mut cfg,
        &mut errs,
    );
    let has_error = errs.iter().any(|e| e.severity == Severity::Error);
    if has_error {
        (None, errs)
    } else {
        (Some(cfg), errs)
    }
}

pub fn load_from_path(path: &Path) -> Result<LoadOk, LoadError> {
    let text = fs::read_to_string(path).map_err(|e| LoadError::Io(e, path.to_path_buf()))?;

    let canon = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let mut visited = vec![canon.clone()];
    let mut cfg = ParsedConfig::default();
    let mut msgs = Vec::new();
    parse_into(
        &text,
        &path.display().to_string(),
        canon.parent(),
        &mut visited,
        &mut cfg,
        &mut msgs,
    );
    if let Some(e) = msgs.iter().find(|m| m.kind == ConfigErrorKind::Syntax) {
        return Err(LoadError::Syntax(e.clone()));
    }
    let has_error = msgs.iter().any(|m| m.severity == Severity::Error);
    if has_error {
        return Err(LoadError::Semantic(msgs));
    }
    Ok(LoadOk {
        config: cfg,
        warnings: msgs,
    })
}

pub fn discover_path() -> Result<PathBuf, String> {
    let candidates = config_candidates();
    if candidates.is_empty() {
        return Err(
            "no candidate config paths available, create proper config file in default location or pass it to -c flag".into(),
        );
    }
    for p in &candidates {
        if p.exists() {
            return Ok(p.clone());
        }
    }
    Err(format!(
        "no config.kdl found, tried: {}",
        candidates
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

pub fn load() -> Result<LoadOk, LoadError> {
    let path = discover_path().map_err(LoadError::PathDiscovery)?;
    load_from_path(&path)
}

fn config_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME")
        && !xdg.is_empty()
    {
        out.push(PathBuf::from(xdg).join("iwwc/config.kdl"));
    }
    if let Ok(home) = std::env::var("HOME")
        && !home.is_empty()
    {
        out.push(PathBuf::from(home).join(".config/iwwc/config.kdl"));
    }
    if let Ok(user) = std::env::var("USER")
        && !user.is_empty()
    {
        out.push(PathBuf::from(format!(
            "/home/{}/.config/iwwc/config.kdl",
            user
        )));
    }
    out
}
