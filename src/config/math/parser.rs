use crate::config::math::ast::{BinaryOp, Expr, Function, UnaryOp};
use crate::config::math::lexer::Token;
use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    UnknownFunction,
    Generic,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub offset: usize,
    pub message: String,
}

pub fn parse(tokens: &[(Token, Range<usize>)]) -> Result<Expr, ParseError> {
    let mut p = Parser { tokens, pos: 0 };
    let expr = p.parse_bp(0)?;
    if p.pos < tokens.len() {
        return Err(ParseError {
            kind: ParseErrorKind::Generic,
            offset: tokens[p.pos].1.start,
            message: "unexpected token after expression".into(),
        });
    }
    Ok(expr)
}

struct Parser<'a> {
    tokens: &'a [(Token, Range<usize>)],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|(t, _)| t)
    }
    fn advance(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos).map(|(t, _)| t);
        if t.is_some() {
            self.pos += 1;
        }
        t
    }
    fn current_offset(&self) -> usize {
        self.tokens
            .get(self.pos)
            .map(|(_, r)| r.start)
            .unwrap_or_else(|| self.tokens.last().map(|(_, r)| r.end).unwrap_or(0))
    }
    fn err(&self, message: impl Into<String>) -> ParseError {
        ParseError {
            kind: ParseErrorKind::Generic,
            offset: self.current_offset(),
            message: message.into(),
        }
    }

    fn parse_bp(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        let mut lhs = match self.peek() {
            Some(Token::Int(n)) => {
                let n = *n;
                self.advance();
                Expr::Int(n)
            }
            Some(Token::Float(f)) => {
                let f = *f;
                self.advance();
                Expr::Float(f)
            }
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();
                if self.peek() == Some(&Token::LParen) {
                    self.parse_call(name)?
                } else {
                    let mut name = name;
                    while self.peek() == Some(&Token::Dot)
                        && matches!(
                            self.tokens.get(self.pos + 1).map(|(t, _)| t),
                            Some(Token::Ident(_))
                        )
                    {
                        self.advance();
                        if let Some(Token::Ident(seg)) = self.advance() {
                            let seg = seg.clone();
                            name.push('.');
                            name.push_str(&seg);
                        }
                    }
                    Expr::Var(name)
                }
            }
            Some(Token::Minus) => {
                self.advance();
                let rhs = self.parse_bp(80)?;
                Expr::Unary(UnaryOp::Neg, Box::new(rhs))
            }
            Some(Token::LParen) => {
                self.advance();
                let e = self.parse_bp(0)?;
                match self.peek() {
                    Some(Token::RParen) => {
                        self.advance();
                        e
                    }
                    _ => return Err(self.err("expected `)`")),
                }
            }
            _ => return Err(self.err("unexpected end of expression")),
        };

        loop {
            let op = match self.peek() {
                Some(Token::Plus) => Some((BinaryOp::Add, 50, 51)),
                Some(Token::Minus) => Some((BinaryOp::Sub, 50, 51)),
                Some(Token::Star) => Some((BinaryOp::Mul, 60, 61)),
                Some(Token::Slash) => Some((BinaryOp::Div, 60, 61)),
                Some(Token::Percent) => Some((BinaryOp::Mod, 60, 61)),
                Some(Token::Caret) => Some((BinaryOp::Pow, 71, 70)),
                Some(Token::Lt) => Some((BinaryOp::Lt, 40, 41)),
                Some(Token::Gt) => Some((BinaryOp::Gt, 40, 41)),
                Some(Token::Le) => Some((BinaryOp::Le, 40, 41)),
                Some(Token::Ge) => Some((BinaryOp::Ge, 40, 41)),
                Some(Token::Eq) => Some((BinaryOp::Eq, 30, 31)),
                Some(Token::Ne) => Some((BinaryOp::Ne, 30, 31)),
                _ => None,
            };
            let Some((op, lbp, rbp)) = op else { break };
            if lbp < min_bp {
                break;
            }

            if matches!(
                op,
                BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge
            ) && matches!(&lhs, Expr::Binary(prev, ..)
                    if matches!(prev, BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge))
            {
                return Err(self.err("chained comparisons (a < b < c) are not allowed"));
            }
            if matches!(op, BinaryOp::Eq | BinaryOp::Ne)
                && matches!(&lhs, Expr::Binary(prev, ..)
                    if matches!(prev, BinaryOp::Eq | BinaryOp::Ne))
            {
                return Err(self.err("chained equality comparisons are not allowed"));
            }

            self.advance();
            let rhs = self.parse_bp(rbp)?;
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
        }

        Ok(lhs)
    }

    fn parse_call(&mut self, name: String) -> Result<Expr, ParseError> {
        let func = match name.as_str() {
            "round" => Function::Round,
            "min" => Function::Min,
            "max" => Function::Max,
            other => {
                return Err(ParseError {
                    kind: ParseErrorKind::UnknownFunction,
                    offset: self.current_offset(),
                    message: format!("unknown function \"{}\"", other),
                });
            }
        };
        match self.advance() {
            Some(Token::LParen) => {}
            _ => return Err(self.err("expected `(`")),
        }
        let mut args = Vec::new();
        if self.peek() != Some(&Token::RParen) {
            loop {
                args.push(self.parse_bp(0)?);
                match self.peek() {
                    Some(Token::Comma) => {
                        self.advance();
                    }
                    Some(Token::RParen) => break,
                    _ => return Err(self.err("expected `,` or `)`")),
                }
            }
        }
        match self.advance() {
            Some(Token::RParen) => {}
            _ => return Err(self.err("expected `)`")),
        }
        let precision = if self.peek() == Some(&Token::Dot) {
            self.advance();
            match self.advance() {
                Some(Token::Int(n)) if *n >= 0 && *n <= u8::MAX as i128 => {
                    if func != Function::Round {
                        return Err(ParseError {
                            kind: ParseErrorKind::Generic,
                            offset: self.current_offset(),
                            message: "precision suffix `.N` is only valid on round(...)".into(),
                        });
                    }
                    Some(*n as u8)
                }
                _ => return Err(self.err("expected integer after `.`")),
            }
        } else {
            None
        };
        Ok(Expr::Call(func, args, precision))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::math::ast::{BinaryOp, Expr, Function, UnaryOp};
    use crate::config::math::lexer::Lexer;

    fn parse_str(input: &str) -> Result<Expr, ParseError> {
        let toks = Lexer::new(input).tokenize().expect("lex");
        parse(&toks)
    }

    fn int(n: i128) -> Expr {
        Expr::Int(n)
    }
    fn binop(op: BinaryOp, a: Expr, b: Expr) -> Expr {
        Expr::Binary(op, Box::new(a), Box::new(b))
    }

    #[test]
    fn literals_and_vars() {
        assert_eq!(parse_str("42").unwrap(), Expr::Int(42));
        assert_eq!(parse_str("3.12").unwrap(), Expr::Float(3.12));
        assert_eq!(parse_str("x").unwrap(), Expr::Var("x".into()));
    }

    #[test]
    fn precedence_add_mul() {
        assert_eq!(
            parse_str("1+2*3").unwrap(),
            binop(BinaryOp::Add, int(1), binop(BinaryOp::Mul, int(2), int(3)))
        );
    }

    #[test]
    fn precedence_parens() {
        assert_eq!(
            parse_str("(1+2)*3").unwrap(),
            binop(BinaryOp::Mul, binop(BinaryOp::Add, int(1), int(2)), int(3))
        );
    }

    #[test]
    fn pow_right_assoc() {
        assert_eq!(
            parse_str("2^3^2").unwrap(),
            binop(BinaryOp::Pow, int(2), binop(BinaryOp::Pow, int(3), int(2)))
        );
    }

    #[test]
    fn unary_minus_tighter_than_pow() {
        assert_eq!(
            parse_str("-2^2").unwrap(),
            binop(
                BinaryOp::Pow,
                Expr::Unary(UnaryOp::Neg, Box::new(int(2))),
                int(2)
            )
        );
    }

    #[test]
    fn comparison_non_associative() {
        assert!(parse_str("1<2<3").is_err());
    }

    #[test]
    fn round_with_precision() {
        assert_eq!(
            parse_str("round(x).1").unwrap(),
            Expr::Call(Function::Round, vec![Expr::Var("x".into())], Some(1))
        );
    }

    #[test]
    fn round_without_precision_parses_ok() {
        assert_eq!(
            parse_str("round(x)").unwrap(),
            Expr::Call(Function::Round, vec![Expr::Var("x".into())], None)
        );
    }

    #[test]
    fn min_max_two_args() {
        assert_eq!(
            parse_str("min(1,2)").unwrap(),
            Expr::Call(Function::Min, vec![int(1), int(2)], None)
        );
        assert_eq!(
            parse_str("max(1,2)").unwrap(),
            Expr::Call(Function::Max, vec![int(1), int(2)], None)
        );
    }

    #[test]
    fn min_with_precision_suffix_rejected() {
        assert!(parse_str("min(1,2).5").is_err());
    }

    #[test]
    fn unknown_function_rejected_at_parse() {
        let err = parse_str("sqrt(4)").unwrap_err();
        assert_eq!(err.kind, ParseErrorKind::UnknownFunction);
    }

    #[test]
    fn dangling_op_errors() {
        assert!(parse_str("1+").is_err());
        assert!(parse_str("*1").is_err());
    }

    #[test]
    fn dotted_smart_var() {
        assert_eq!(
            parse_str("iwwc.ram.total").unwrap(),
            Expr::Var("iwwc.ram.total".into())
        );
        assert_eq!(parse_str("a.b").unwrap(), Expr::Var("a.b".into()));
    }

    #[test]
    fn round_precision_still_works() {
        assert!(matches!(
            parse_str("round(x).1").unwrap(),
            Expr::Call(Function::Round, _, Some(1))
        ));
    }
}
