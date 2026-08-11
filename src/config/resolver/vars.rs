use crate::config::math::{
    self,
    value::{Value, VarStore},
};
use crate::config::types::{ParsedConfig, VarValue};
use crate::config::{ConfigError, ConfigErrorKind, Severity};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

pub struct FlatEnv {
    map: HashMap<String, VarValue>,
    config_declared: HashSet<String>,
    smart: RefCell<HashMap<String, VarValue>>,
    sampled: RefCell<HashSet<crate::config::smart::Namespace>>,
    accessed_smart: RefCell<HashSet<String>>,
}

impl FlatEnv {
    pub fn lookup_value(&self, name: &str) -> Option<VarValue> {
        if let Some(v) = self.lookup_smart(name) {
            return Some(v);
        }
        self.map.get(name).cloned()
    }

    fn lookup_smart(&self, name: &str) -> Option<VarValue> {
        let ns = crate::config::smart::Namespace::of(name)?;
        if self.sampled.borrow_mut().insert(ns) {
            self.smart.borrow_mut().extend(ns.sample());
        }
        let smart = self.smart.borrow();
        let v = smart.get(name)?;
        self.accessed_smart.borrow_mut().insert(name.to_string());
        if crate::config::smart::is_unset(name, v) && self.config_declared.contains(name) {
            return None;
        }
        Some(v.clone())
    }

    pub fn smart_polls(&self) -> Vec<(String, Option<std::time::Duration>)> {
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        for key in self.accessed_smart.borrow().iter() {
            if let Some(ns) = crate::config::smart::Namespace::of(key)
                && seen.insert(ns)
            {
                out.push((ns.name().to_string(), ns.poll_interval()));
            }
        }
        out
    }
}

impl VarStore for FlatEnv {
    fn lookup(&self, name: &str) -> Option<VarValue> {
        self.lookup_value(name)
    }
}

#[derive(Clone)]
pub struct AliasBinding {
    pub name: String,
    pub prefix: String,
    pub index: VarValue,
}

pub struct AliasEnv<'a> {
    base: &'a FlatEnv,
    aliases: Vec<AliasBinding>,
}

impl<'a> AliasEnv<'a> {
    pub fn new(base: &'a FlatEnv, aliases: Vec<AliasBinding>) -> Self {
        Self { base, aliases }
    }
}

impl VarStore for AliasEnv<'_> {
    fn lookup(&self, name: &str) -> Option<VarValue> {
        for binding in self.aliases.iter().rev() {
            if let Some(rest) = name.strip_prefix(&binding.name)
                && let Some(rest) = rest.strip_prefix('.')
            {
                if rest == "index" {
                    return Some(binding.index.clone());
                }
                return self.base.lookup(&format!("{}.{}", binding.prefix, rest));
            }
        }
        self.base.lookup(name)
    }
}

pub(crate) fn resolve_vars(
    config: &ParsedConfig,
    used: &mut HashSet<String>,
    errs: &mut Vec<ConfigError>,
) -> FlatEnv {
    let mut env = FlatEnv {
        map: HashMap::new(),
        config_declared: config.vars.keys().cloned().collect(),
        smart: RefCell::new(HashMap::new()),
        sampled: RefCell::new(HashSet::new()),
        accessed_smart: RefCell::new(HashSet::new()),
    };
    let mut resolving: HashSet<String> = HashSet::new();
    let names: Vec<String> = config.vars.keys().cloned().collect();
    for name in &names {
        resolve_one(name, config, &mut env, used, &mut resolving, errs);
    }
    env
}

fn resolve_one(
    name: &str,
    config: &ParsedConfig,
    env: &mut FlatEnv,
    used: &mut HashSet<String>,
    resolving: &mut HashSet<String>,
    errs: &mut Vec<ConfigError>,
) {
    if env.map.contains_key(name) {
        return;
    }
    let Some(decl) = config.vars.get(name) else {
        return;
    };

    if !resolving.insert(name.to_string()) {
        errs.push(ConfigError {
            kind: ConfigErrorKind::CircularReference,
            span: decl.span.clone(),
            message: format!("circular reference in variable \"{}\"", name),
            severity: Severity::Error,
        });
        env.map
            .insert(name.to_string(), VarValue::Str(String::new()));
        return;
    }

    if let VarValue::Str(raw) = &decl.value {
        for dep in referenced_vars(raw) {
            if dep != name && config.vars.contains_key(&dep) {
                used.insert(dep.clone());
                if !env.map.contains_key(&dep) {
                    resolve_one(&dep, config, env, used, resolving, errs);
                }
            }
        }
    }

    let (raw_str, span_source, span_clone) = match &decl.value {
        VarValue::Str(s) => (
            Some(s.clone()),
            Some(decl.span.source.clone()),
            decl.span.clone(),
        ),
        _ => (None, None, decl.span.clone()),
    };
    let value_copy = match &decl.value {
        VarValue::Int(i) => VarValue::Int(*i),
        VarValue::Float(f) => VarValue::Float(*f),
        VarValue::Bool(b) => VarValue::Bool(*b),
        VarValue::Str(_) => VarValue::Str(String::new()),
    };

    let value = if let (Some(s), Some(source)) = (raw_str, span_source) {
        match math::evaluate(&s, &*env, &source, 0) {
            Ok(v) => value_to_var(v),
            Err(eval_err) => {
                errs.push(ConfigError {
                    kind: ConfigErrorKind::Expression,
                    span: span_clone,
                    message: eval_err.message,
                    severity: Severity::Error,
                });
                VarValue::Str(String::new())
            }
        }
    } else {
        value_copy
    };

    resolving.remove(name);
    env.map.insert(name.to_string(), value);
}

fn value_to_var(v: Value) -> VarValue {
    match v {
        Value::Int(i) => VarValue::Int(i),
        Value::Float(d) => VarValue::Float(d.value),
        Value::Bool(b) => VarValue::Bool(b),
        Value::Str(s) => VarValue::Str(s),
    }
}

pub(crate) fn referenced_vars(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'$' && bytes[i + 1] == b'{' {
            let mut j = i + 2;
            while j < bytes.len() && bytes[j] != b'}' {
                if bytes[j].is_ascii_alphabetic() || bytes[j] == b'_' {
                    let start = j;
                    while j < bytes.len()
                        && (bytes[j].is_ascii_alphanumeric()
                            || bytes[j] == b'_'
                            || bytes[j] == b'.')
                    {
                        j += 1;
                    }
                    out.push(s[start..j].to_string());
                } else {
                    j += 1;
                }
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_str;
    use crate::config::{ConfigErrorKind, Severity};

    fn flat(kdl: &str) -> (FlatEnv, Vec<crate::config::ConfigError>) {
        let (cfg, parse_errs) = parse_str(kdl, "<test>");
        let cfg = cfg.expect("parse should succeed for these fixtures");
        assert!(
            parse_errs.iter().all(|e| e.severity != Severity::Error),
            "fixture must parse cleanly: {:?}",
            parse_errs
        );
        let mut errs = Vec::new();
        let mut used = std::collections::HashSet::new();
        let env = resolve_vars(&cfg, &mut used, &mut errs);
        (env, errs)
    }

    #[test]
    fn plain_values_passthrough() {
        let (env, errs) = flat("var a=1\nvar b=2.0\nvar c=\"hi\"\nvar d=#true");
        assert!(errs.is_empty());
        assert!(matches!(
            env.lookup("a"),
            Some(crate::config::types::VarValue::Int(1))
        ));
        assert!(
            matches!(env.lookup("c"), Some(crate::config::types::VarValue::Str(s)) if s == "hi")
        );
    }

    #[test]
    fn var_of_expression_resolves() {
        let (env, errs) = flat("var y=1\nvar x=\"${y/2}\"");
        assert!(errs.is_empty(), "errs: {:?}", errs);
        assert!(matches!(
            env.lookup("x"),
            Some(crate::config::types::VarValue::Int(0))
        ));
    }

    #[test]
    fn element_fragment_passthrough() {
        let (env, errs) = flat("var x=\"container c1 { child t1 }\"");
        assert!(errs.is_empty());
        assert!(
            matches!(env.lookup("x"), Some(crate::config::types::VarValue::Str(s)) if s == "container c1 { child t1 }")
        );
    }

    #[test]
    fn circular_vars_error() {
        let (_env, errs) = flat("var a=\"${b}\"\nvar b=\"${a}\"");
        assert!(
            errs.iter()
                .any(|e| e.kind == ConfigErrorKind::CircularReference),
            "expected CircularReference, got {:?}",
            errs
        );
    }
}
