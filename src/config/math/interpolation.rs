#[derive(Debug, Clone, PartialEq)]
pub enum Segment<'a> {
    Literal(&'a str),
    Expr { text: &'a str, offset: usize },
}

#[derive(Debug, Clone)]
pub struct InterpError {
    pub kind: InterpErrorKind,
    pub offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpErrorKind {
    Unterminated,
    Nested,
}

pub fn segments(input: &str) -> Result<Vec<Segment<'_>>, InterpError> {
    let bytes = input.as_bytes();
    let mut out = Vec::new();
    let mut lit_start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'$' && bytes[i + 1] == b'{' {
            if lit_start < i {
                out.push(Segment::Literal(&input[lit_start..i]));
            }
            let expr_start = i + 2;
            i = expr_start;
            let mut depth: usize = 0;
            let saw_close;
            loop {
                if i >= bytes.len() {
                    return Err(InterpError {
                        kind: InterpErrorKind::Unterminated,
                        offset: expr_start - 2,
                    });
                }
                if i + 1 < bytes.len() && bytes[i] == b'$' && bytes[i + 1] == b'{' {
                    return Err(InterpError {
                        kind: InterpErrorKind::Nested,
                        offset: i,
                    });
                }
                match bytes[i] {
                    b'{' => {
                        depth += 1;
                        i += 1;
                    }
                    b'}' if depth == 0 => {
                        saw_close = i;
                        i += 1;
                        break;
                    }
                    b'}' => {
                        depth -= 1;
                        i += 1;
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            out.push(Segment::Expr {
                text: &input[expr_start..saw_close],
                offset: expr_start,
            });
            lit_start = i;
        } else {
            i += 1;
        }
    }
    if lit_start < bytes.len() {
        out.push(Segment::Literal(&input[lit_start..]));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect(input: &'static str) -> Vec<Segment<'static>> {
        segments(input).unwrap()
    }

    #[test]
    fn pure_expression() {
        let segs = collect("${x}");
        assert_eq!(segs.len(), 1);
        if let Segment::Expr { text, offset } = segs[0] {
            assert_eq!(text, "x");
            assert_eq!(offset, 2);
        } else {
            panic!("expected Expr");
        }
    }

    #[test]
    fn pure_literal() {
        let segs = collect("hello");
        assert_eq!(segs.len(), 1);
        assert!(matches!(segs[0], Segment::Literal("hello")));
    }

    #[test]
    fn mixed() {
        let segs = collect("hi ${x} bye");
        assert_eq!(segs.len(), 3);
        assert!(matches!(segs[0], Segment::Literal("hi ")));
        if let Segment::Expr { text, offset } = segs[1] {
            assert_eq!(text, "x");
            assert_eq!(offset, 5);
        } else {
            panic!("expected Expr");
        }
        assert!(matches!(segs[2], Segment::Literal(" bye")));
    }

    #[test]
    fn brace_depth_in_call() {
        let segs = collect("${min(a, b)}");
        assert_eq!(segs.len(), 1);
        if let Segment::Expr { text, .. } = segs[0] {
            assert_eq!(text, "min(a, b)");
        } else {
            panic!("expected Expr");
        }
    }

    #[test]
    fn multiple_blocks() {
        let segs = collect("${a} and ${b}");
        assert!(segs.len() == 3 || segs.len() == 4);
        if let Segment::Expr { text, .. } = segs[0] {
            assert_eq!(text, "a");
        } else {
            panic!();
        }
        assert!(matches!(segs[1], Segment::Literal(" and ")));
        if let Segment::Expr { text, .. } = segs[2] {
            assert_eq!(text, "b");
        } else {
            panic!();
        }
    }

    #[test]
    fn empty_input() {
        let segs = collect("");
        assert!(segs.is_empty() || (segs.len() == 1 && matches!(segs[0], Segment::Literal(""))));
    }

    #[test]
    fn unterminated() {
        let err = segments("${x").unwrap_err();
        assert_eq!(err.kind, InterpErrorKind::Unterminated);
    }

    #[test]
    fn nested_rejected() {
        let err = segments("${${x}}").unwrap_err();
        assert_eq!(err.kind, InterpErrorKind::Nested);
    }
}
