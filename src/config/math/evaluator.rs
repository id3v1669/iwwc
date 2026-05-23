use crate::config::math::ast::{BinaryOp, Expr, Function, UnaryOp};
use crate::config::math::error::{EvalError, EvalErrorKind};
use crate::config::math::value::{Decimal, Value, VarStore};
use crate::config::types::{SourceText, Span, VarValue};
use std::sync::Arc;

pub fn eval(expr: &Expr, vars: &dyn VarStore) -> Result<Value, EvalError> {
    match expr {
        Expr::Int(i) => Ok(Value::Int(*i)),
        Expr::Float(f) => Ok(Value::Float(Decimal::new(*f))),
        Expr::Var(name) => match vars.lookup(name) {
            None => Err(eval_err(
                EvalErrorKind::UnknownVariable,
                format!("variable \"{}\" is not defined", name),
            )),
            Some(v) => Ok(value_from_var(v)),
        },
        Expr::Unary(UnaryOp::Neg, inner) => {
            let v = eval(inner, vars)?;
            match v {
                Value::Int(i) => i.checked_neg().map(Value::Int).ok_or_else(overflow),
                Value::Float(d) => Ok(Value::Float(Decimal {
                    value: -d.value,
                    precision: d.precision,
                })),
                Value::Bool(_) | Value::Str(_) => Err(type_mismatch_for(&v)),
            }
        }
        Expr::Binary(op, a, b) => {
            let lhs = eval(a, vars)?;
            let rhs = eval(b, vars)?;
            eval_binop(*op, lhs, rhs)
        }
        Expr::Call(func, args, precision) => eval_call(*func, args, *precision, vars),
    }
}

fn eval_call(
    func: Function,
    args: &[Expr],
    precision: Option<u8>,
    vars: &dyn VarStore,
) -> Result<Value, EvalError> {
    match func {
        Function::Round => {
            if args.len() != 1 {
                return Err(eval_err(
                    EvalErrorKind::WrongArity,
                    format!("round expects 1 argument, got {}", args.len()),
                ));
            }
            let n = match precision {
                Some(n) => n,
                None => {
                    return Err(eval_err(
                        EvalErrorKind::MissingPrecision,
                        "round() requires a .N precision suffix, e.g. round(x).1".into(),
                    ));
                }
            };
            let v = eval(&args[0], vars)?;
            let raw = match v {
                Value::Int(i) => i as f64,
                Value::Float(d) => d.value,
                Value::Bool(_) | Value::Str(_) => return Err(type_mismatch_for(&v)),
            };
            let factor = 10f64.powi(n as i32);
            let rounded = (raw * factor).round() / factor;
            Ok(Value::Float(Decimal {
                value: rounded,
                precision: Some(n),
            }))
        }
        Function::Min | Function::Max => {
            if args.len() != 2 {
                let name = if func == Function::Min { "min" } else { "max" };
                return Err(eval_err(
                    EvalErrorKind::WrongArity,
                    format!("{} expects 2 arguments, got {}", name, args.len()),
                ));
            }
            let a = eval(&args[0], vars)?;
            let b = eval(&args[1], vars)?;
            for v in [&a, &b] {
                if matches!(v, Value::Str(_) | Value::Bool(_)) {
                    return Err(type_mismatch_for(v));
                }
            }
            if let (Value::Int(ai), Value::Int(bi)) = (&a, &b) {
                let chosen = match func {
                    Function::Min => (*ai).min(*bi),
                    Function::Max => (*ai).max(*bi),
                    _ => unreachable!(),
                };
                return Ok(Value::Int(chosen));
            }
            let (av, ap) = to_float(a);
            let (bv, bp) = to_float(b);
            let chosen = match func {
                Function::Min => {
                    if av <= bv {
                        av
                    } else {
                        bv
                    }
                }
                Function::Max => {
                    if av >= bv {
                        av
                    } else {
                        bv
                    }
                }
                _ => unreachable!(),
            };
            Ok(Value::Float(Decimal {
                value: chosen,
                precision: max_prec(ap, bp),
            }))
        }
    }
}

fn value_from_var(v: &VarValue) -> Value {
    match v {
        VarValue::Int(i) => Value::Int(*i),
        VarValue::Float(f) => Value::Float(Decimal::new(*f)),
        VarValue::Bool(b) => Value::Bool(*b),
        VarValue::Str(s) => Value::Str(s.clone()),
    }
}

fn eval_binop(op: BinaryOp, lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    use BinaryOp::*;
    if matches!(op, Eq | Ne) {
        if let (Value::Str(a), Value::Str(b)) = (&lhs, &rhs) {
            let same = a == b;
            return Ok(Value::Bool(if op == Eq { same } else { !same }));
        }
        if let (Value::Bool(a), Value::Bool(b)) = (&lhs, &rhs) {
            return Ok(Value::Bool(if op == Eq { a == b } else { a != b }));
        }
        if std::mem::discriminant(&lhs) != std::mem::discriminant(&rhs) {
            return Ok(Value::Bool(op == Ne));
        }
    }

    match op {
        Add | Sub | Mul | Div | Mod | Pow => {
            if matches!(lhs, Value::Str(_)) || matches!(rhs, Value::Str(_)) {
                return Err(eval_err(
                    EvalErrorKind::TypeMismatch,
                    "math ops cannot be applied to strings".into(),
                ));
            }
            if matches!(lhs, Value::Bool(_)) || matches!(rhs, Value::Bool(_)) {
                return Err(eval_err(
                    EvalErrorKind::TypeMismatch,
                    "math ops cannot be applied to booleans".into(),
                ));
            }
        }
        Lt | Gt | Le | Ge => {
            if matches!(lhs, Value::Str(_)) || matches!(rhs, Value::Str(_)) {
                return Err(eval_err(
                    EvalErrorKind::TypeMismatch,
                    "ordering comparisons cannot be applied to strings".into(),
                ));
            }
            if matches!(lhs, Value::Bool(_)) || matches!(rhs, Value::Bool(_)) {
                return Err(eval_err(
                    EvalErrorKind::TypeMismatch,
                    "ordering comparisons cannot be applied to booleans".into(),
                ));
            }
        }
        Eq | Ne => {}
    }

    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => match op {
            Add => a.checked_add(b).map(Value::Int).ok_or_else(overflow),
            Sub => a.checked_sub(b).map(Value::Int).ok_or_else(overflow),
            Mul => a.checked_mul(b).map(Value::Int).ok_or_else(overflow),
            Div => {
                if b == 0 {
                    Err(div_by_zero())
                } else {
                    a.checked_div(b).map(Value::Int).ok_or_else(overflow)
                }
            }
            Mod => {
                if b == 0 {
                    Err(div_by_zero())
                } else {
                    a.checked_rem(b).map(Value::Int).ok_or_else(overflow)
                }
            }
            Pow => {
                if b < 0 {
                    Ok(Value::Float(Decimal::new((a as f64).powf(b as f64))))
                } else {
                    let exp: u32 = b.try_into().map_err(|_| overflow())?;
                    a.checked_pow(exp).map(Value::Int).ok_or_else(overflow)
                }
            }
            Lt => Ok(Value::Bool(a < b)),
            Gt => Ok(Value::Bool(a > b)),
            Le => Ok(Value::Bool(a <= b)),
            Ge => Ok(Value::Bool(a >= b)),
            Eq => Ok(Value::Bool(a == b)),
            Ne => Ok(Value::Bool(a != b)),
        },
        (l, r) => {
            // At least one is Float (Str/Bool ruled out for these ops above).
            let (la, lp) = to_float(l);
            let (ra, rp) = to_float(r);
            let result = match op {
                Add => Value::Float(Decimal {
                    value: la + ra,
                    precision: max_prec(lp, rp),
                }),
                Sub => Value::Float(Decimal {
                    value: la - ra,
                    precision: max_prec(lp, rp),
                }),
                Mul => Value::Float(Decimal {
                    value: la * ra,
                    precision: max_prec(lp, rp),
                }),
                Div => {
                    if ra == 0.0 {
                        return Err(div_by_zero());
                    }
                    Value::Float(Decimal {
                        value: la / ra,
                        precision: max_prec(lp, rp),
                    })
                }
                Mod => {
                    if ra == 0.0 {
                        return Err(div_by_zero());
                    }
                    Value::Float(Decimal {
                        value: la % ra,
                        precision: max_prec(lp, rp),
                    })
                }
                Pow => Value::Float(Decimal {
                    value: la.powf(ra),
                    precision: max_prec(lp, rp),
                }),
                Lt => Value::Bool(la < ra),
                Gt => Value::Bool(la > ra),
                Le => Value::Bool(la <= ra),
                Ge => Value::Bool(la >= ra),
                Eq => Value::Bool(la == ra),
                Ne => Value::Bool(la != ra),
            };
            Ok(result)
        }
    }
}

fn to_float(v: Value) -> (f64, Option<u8>) {
    match v {
        Value::Int(i) => (i as f64, None),
        Value::Float(d) => (d.value, d.precision),
        _ => unreachable!("non-numeric ruled out before to_float"),
    }
}

fn max_prec(a: Option<u8>, b: Option<u8>) -> Option<u8> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.max(y)),
        (Some(x), None) | (None, Some(x)) => Some(x),
        (None, None) => None,
    }
}

fn type_mismatch_for(v: &Value) -> EvalError {
    let msg = match v {
        Value::Str(_) => "math ops cannot be applied to strings",
        Value::Bool(_) => "math ops cannot be applied to booleans",
        _ => "type mismatch",
    };
    eval_err(EvalErrorKind::TypeMismatch, msg.into())
}

fn overflow() -> EvalError {
    eval_err(
        EvalErrorKind::Overflow,
        "integer overflow in expression".into(),
    )
}

fn div_by_zero() -> EvalError {
    eval_err(EvalErrorKind::DivByZero, "division by zero".into())
}

fn eval_err(kind: EvalErrorKind, message: String) -> EvalError {
    EvalError {
        kind,
        span: Span {
            source: SourceText {
                label: Arc::from("<expr>"),
                text: Arc::from(""),
            },
            span: miette::SourceSpan::new(0.into(), 0),
        },
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::math::ast::{BinaryOp, Expr, UnaryOp};
    use crate::config::math::error::EvalErrorKind;
    use crate::config::math::value::{Decimal, Value};
    use crate::config::types::{SourceText, Span, VarDecl, VarValue};
    use indexmap::IndexMap;
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

    fn store(pairs: &[(&str, VarValue)]) -> IndexMap<String, VarDecl> {
        let mut m = IndexMap::new();
        for (name, value) in pairs {
            m.insert(
                name.to_string(),
                VarDecl {
                    value: value.clone(),
                    span: span(),
                },
            );
        }
        m
    }

    fn int(n: i128) -> Expr {
        Expr::Int(n)
    }
    fn float(f: f64) -> Expr {
        Expr::Float(f)
    }
    fn binop(op: BinaryOp, a: Expr, b: Expr) -> Expr {
        Expr::Binary(op, Box::new(a), Box::new(b))
    }

    #[test]
    fn literal_int() {
        let v = eval(&int(42), &store(&[])).unwrap();
        assert_eq!(v, Value::Int(42));
    }

    #[test]
    fn literal_float() {
        let v = eval(&float(3.14), &store(&[])).unwrap();
        assert_eq!(
            v,
            Value::Float(Decimal {
                value: 3.14,
                precision: None
            })
        );
    }

    #[test]
    fn var_lookup_int() {
        let v = eval(&Expr::Var("x".into()), &store(&[("x", VarValue::Int(7))])).unwrap();
        assert_eq!(v, Value::Int(7));
    }

    #[test]
    fn var_lookup_string_passthrough() {
        let v = eval(
            &Expr::Var("s".into()),
            &store(&[("s", VarValue::Str("xfdf".into()))]),
        )
        .unwrap();
        assert_eq!(v, Value::Str("xfdf".into()));
    }

    #[test]
    fn var_not_found() {
        let err = eval(&Expr::Var("missing".into()), &store(&[])).unwrap_err();
        assert_eq!(err.kind, EvalErrorKind::UnknownVariable);
    }

    #[test]
    fn int_add() {
        let v = eval(&binop(BinaryOp::Add, int(2), int(3)), &store(&[])).unwrap();
        assert_eq!(v, Value::Int(5));
    }

    #[test]
    fn int_div_truncates() {
        let v = eval(&binop(BinaryOp::Div, int(5), int(4)), &store(&[])).unwrap();
        assert_eq!(v, Value::Int(1));
    }

    #[test]
    fn int_div_by_zero() {
        let err = eval(&binop(BinaryOp::Div, int(1), int(0)), &store(&[])).unwrap_err();
        assert_eq!(err.kind, EvalErrorKind::DivByZero);
    }

    #[test]
    fn float_mul() {
        let v = eval(&binop(BinaryOp::Mul, float(3.14), int(2)), &store(&[])).unwrap();
        if let Value::Float(d) = v {
            assert!((d.value - 6.28).abs() < 1e-9);
            assert_eq!(d.precision, None);
        } else {
            panic!("expected Float");
        }
    }

    #[test]
    fn int_pow() {
        let v = eval(&binop(BinaryOp::Pow, int(3), int(2)), &store(&[])).unwrap();
        assert_eq!(v, Value::Int(9));
    }

    #[test]
    fn int_mod() {
        let v = eval(&binop(BinaryOp::Mod, int(3), int(2)), &store(&[])).unwrap();
        assert_eq!(v, Value::Int(1));
    }

    #[test]
    fn unary_neg_int() {
        let v = eval(&Expr::Unary(UnaryOp::Neg, Box::new(int(5))), &store(&[])).unwrap();
        assert_eq!(v, Value::Int(-5));
    }

    #[test]
    fn math_op_on_string_errors() {
        let err = eval(
            &binop(BinaryOp::Div, Expr::Var("s".into()), int(2)),
            &store(&[("s", VarValue::Str("xfdf".into()))]),
        )
        .unwrap_err();
        assert_eq!(err.kind, EvalErrorKind::TypeMismatch);
        assert_eq!(err.message, "math ops cannot be applied to strings");
    }

    #[test]
    fn math_op_on_bool_errors() {
        let err = eval(
            &binop(BinaryOp::Add, Expr::Var("b".into()), int(1)),
            &store(&[("b", VarValue::Bool(true))]),
        )
        .unwrap_err();
        assert_eq!(err.kind, EvalErrorKind::TypeMismatch);
        assert_eq!(err.message, "math ops cannot be applied to booleans");
    }

    use crate::config::math::ast::Function;

    fn call(f: Function, args: Vec<Expr>, prec: Option<u8>) -> Expr {
        Expr::Call(f, args, prec)
    }

    #[test]
    fn cmp_lt_int() {
        let v = eval(&binop(BinaryOp::Lt, int(2), int(3)), &store(&[])).unwrap();
        assert_eq!(v, Value::Bool(true));
    }
    #[test]
    fn cmp_ge_mixed() {
        let v = eval(&binop(BinaryOp::Ge, float(2.5), int(2)), &store(&[])).unwrap();
        assert_eq!(v, Value::Bool(true));
    }
    #[test]
    fn cmp_eq_strings() {
        let v = eval(
            &binop(BinaryOp::Eq, Expr::Var("a".into()), Expr::Var("b".into())),
            &store(&[
                ("a", VarValue::Str("hi".into())),
                ("b", VarValue::Str("hi".into())),
            ]),
        )
        .unwrap();
        assert_eq!(v, Value::Bool(true));
    }
    #[test]
    fn cmp_eq_cross_type_false() {
        let v = eval(
            &binop(BinaryOp::Eq, int(1), Expr::Var("s".into())),
            &store(&[("s", VarValue::Str("1".into()))]),
        )
        .unwrap();
        assert_eq!(v, Value::Bool(false));
    }

    #[test]
    fn round_with_precision() {
        let v = eval(
            &call(Function::Round, vec![float(3.14)], Some(1)),
            &store(&[]),
        )
        .unwrap();
        if let Value::Float(d) = v {
            assert!((d.value - 3.1).abs() < 1e-9);
            assert_eq!(d.precision, Some(1));
        } else {
            panic!("expected Float");
        }
    }

    #[test]
    fn round_missing_precision() {
        let err = eval(&call(Function::Round, vec![float(3.14)], None), &store(&[])).unwrap_err();
        assert_eq!(err.kind, EvalErrorKind::MissingPrecision);
    }

    #[test]
    fn round_wrong_arity() {
        let err = eval(
            &call(Function::Round, vec![float(1.0), float(2.0)], Some(1)),
            &store(&[]),
        )
        .unwrap_err();
        assert_eq!(err.kind, EvalErrorKind::WrongArity);
    }

    #[test]
    fn min_two_args() {
        // min(Int, Int) stays Int.
        let v = eval(
            &call(Function::Min, vec![int(3), int(5)], None),
            &store(&[]),
        )
        .unwrap();
        assert_eq!(v, Value::Int(3));
    }

    #[test]
    fn min_int_float_promotes() {
        // min(Int, Float) promotes to Float.
        let v = eval(
            &call(Function::Min, vec![int(3), float(5.0)], None),
            &store(&[]),
        )
        .unwrap();
        if let Value::Float(d) = v {
            assert!((d.value - 3.0).abs() < 1e-9);
            assert_eq!(d.precision, None);
        } else {
            panic!("expected Float, got {:?}", v);
        }
    }

    #[test]
    fn max_mixed_promotion() {
        let v = eval(
            &call(Function::Max, vec![float(1.5), int(2)], None),
            &store(&[]),
        )
        .unwrap();
        if let Value::Float(d) = v {
            assert!((d.value - 2.0).abs() < 1e-9);
            assert_eq!(d.precision, None);
        } else {
            panic!("expected Float");
        }
    }

    #[test]
    fn min_wrong_arity() {
        let err = eval(&call(Function::Min, vec![int(1)], None), &store(&[])).unwrap_err();
        assert_eq!(err.kind, EvalErrorKind::WrongArity);
    }

    #[test]
    fn precision_propagates_through_arithmetic() {
        let inner = call(Function::Round, vec![float(3.46)], Some(1));
        let v = eval(&binop(BinaryOp::Mul, inner, int(2)), &store(&[])).unwrap();
        if let Value::Float(d) = v {
            assert!((d.value - 7.0).abs() < 1e-9);
            assert_eq!(d.precision, Some(1));
        } else {
            panic!("expected Float");
        }
    }
}
