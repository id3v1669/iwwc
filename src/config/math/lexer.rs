use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Int(i128),
    Float(f64),
    Ident(String),
    LParen,
    RParen,
    Comma,
    Dot,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
}

#[derive(Debug, Clone)]
pub struct LexError {
    pub offset: usize,
    pub ch: char,
}

pub struct Lexer<'src> {
    src: &'src str,
    bytes: &'src [u8],
    pos: usize,
}

impl<'src> Lexer<'src> {
    pub fn new(src: &'src str) -> Self {
        Self {
            src,
            bytes: src.as_bytes(),
            pos: 0,
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<(Token, Range<usize>)>, LexError> {
        let mut out = Vec::new();
        while let Some(&b) = self.bytes.get(self.pos) {
            if b.is_ascii_whitespace() {
                self.pos += 1;
                continue;
            }
            let start = self.pos;
            let tok = match b {
                b'(' => {
                    self.pos += 1;
                    Token::LParen
                }
                b')' => {
                    self.pos += 1;
                    Token::RParen
                }
                b',' => {
                    self.pos += 1;
                    Token::Comma
                }
                b'+' => {
                    self.pos += 1;
                    Token::Plus
                }
                b'-' => {
                    self.pos += 1;
                    Token::Minus
                }
                b'*' => {
                    self.pos += 1;
                    Token::Star
                }
                b'/' => {
                    self.pos += 1;
                    Token::Slash
                }
                b'%' => {
                    self.pos += 1;
                    Token::Percent
                }
                b'^' => {
                    self.pos += 1;
                    Token::Caret
                }
                b'<' => {
                    self.pos += 1;
                    if self.bytes.get(self.pos) == Some(&b'=') {
                        self.pos += 1;
                        Token::Le
                    } else {
                        Token::Lt
                    }
                }
                b'>' => {
                    self.pos += 1;
                    if self.bytes.get(self.pos) == Some(&b'=') {
                        self.pos += 1;
                        Token::Ge
                    } else {
                        Token::Gt
                    }
                }
                b'=' => {
                    if self.bytes.get(self.pos + 1) == Some(&b'=') {
                        self.pos += 2;
                        Token::Eq
                    } else {
                        return Err(LexError {
                            offset: self.pos,
                            ch: '=',
                        });
                    }
                }
                b'!' => {
                    if self.bytes.get(self.pos + 1) == Some(&b'=') {
                        self.pos += 2;
                        Token::Ne
                    } else {
                        return Err(LexError {
                            offset: self.pos,
                            ch: '!',
                        });
                    }
                }
                b'.' => {
                    self.pos += 1;
                    Token::Dot
                }
                b'0'..=b'9' => self.lex_number(start),
                b'A'..=b'Z' | b'a'..=b'z' | b'_' => self.lex_ident(start),
                _ => {
                    let ch = self.src[self.pos..].chars().next().unwrap_or('\0');
                    return Err(LexError {
                        offset: self.pos,
                        ch,
                    });
                }
            };
            out.push((tok, start..self.pos));
        }
        Ok(out)
    }

    fn lex_number(&mut self, start: usize) -> Token {
        while matches!(self.bytes.get(self.pos), Some(b) if b.is_ascii_digit()) {
            self.pos += 1;
        }
        let has_fraction = self.bytes.get(self.pos) == Some(&b'.')
            && matches!(self.bytes.get(self.pos + 1), Some(b) if b.is_ascii_digit());
        if has_fraction {
            self.pos += 1;
            while matches!(self.bytes.get(self.pos), Some(b) if b.is_ascii_digit()) {
                self.pos += 1;
            }
            let s = &self.src[start..self.pos];
            Token::Float(s.parse::<f64>().unwrap())
        } else {
            let s = &self.src[start..self.pos];
            Token::Int(s.parse::<i128>().unwrap())
        }
    }

    fn lex_ident(&mut self, start: usize) -> Token {
        while matches!(self.bytes.get(self.pos),
            Some(b) if b.is_ascii_alphanumeric() || *b == b'_')
        {
            self.pos += 1;
        }
        Token::Ident(self.src[start..self.pos].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens_of(input: &str) -> Vec<Token> {
        Lexer::new(input)
            .tokenize()
            .unwrap()
            .into_iter()
            .map(|(t, _)| t)
            .collect()
    }

    #[test]
    fn lex_numbers() {
        assert_eq!(tokens_of("0"), vec![Token::Int(0)]);
        assert_eq!(tokens_of("42"), vec![Token::Int(42)]);
        assert_eq!(tokens_of("3.12"), vec![Token::Float(3.12)]);
        assert_eq!(tokens_of("0.5"), vec![Token::Float(0.5)]);
    }

    #[test]
    fn lex_ident() {
        assert_eq!(tokens_of("x"), vec![Token::Ident("x".into())]);
        assert_eq!(tokens_of("foo_bar"), vec![Token::Ident("foo_bar".into())]);
        assert_eq!(tokens_of("round"), vec![Token::Ident("round".into())]);
    }

    #[test]
    fn lex_operators() {
        assert_eq!(tokens_of("+"), vec![Token::Plus]);
        assert_eq!(tokens_of("-"), vec![Token::Minus]);
        assert_eq!(tokens_of("*"), vec![Token::Star]);
        assert_eq!(tokens_of("/"), vec![Token::Slash]);
        assert_eq!(tokens_of("%"), vec![Token::Percent]);
        assert_eq!(tokens_of("^"), vec![Token::Caret]);
        assert_eq!(tokens_of("("), vec![Token::LParen]);
        assert_eq!(tokens_of(")"), vec![Token::RParen]);
        assert_eq!(tokens_of(","), vec![Token::Comma]);
        assert_eq!(tokens_of("."), vec![Token::Dot]);
        assert_eq!(tokens_of("<"), vec![Token::Lt]);
        assert_eq!(tokens_of(">"), vec![Token::Gt]);
        assert_eq!(tokens_of("<="), vec![Token::Le]);
        assert_eq!(tokens_of(">="), vec![Token::Ge]);
        assert_eq!(tokens_of("=="), vec![Token::Eq]);
        assert_eq!(tokens_of("!="), vec![Token::Ne]);
    }

    #[test]
    fn lex_whitespace_skipped() {
        assert_eq!(
            tokens_of(" x + 1 "),
            vec![Token::Ident("x".into()), Token::Plus, Token::Int(1)]
        );
    }

    #[test]
    fn lex_compound() {
        assert_eq!(
            tokens_of("round(x).1"),
            vec![
                Token::Ident("round".into()),
                Token::LParen,
                Token::Ident("x".into()),
                Token::RParen,
                Token::Dot,
                Token::Int(1),
            ]
        );
    }

    #[test]
    fn lex_byte_ranges() {
        let toks = Lexer::new("ab + 3").tokenize().unwrap();
        assert_eq!(toks[0].1, 0..2);
        assert_eq!(toks[1].1, 3..4);
        assert_eq!(toks[2].1, 5..6);
    }

    #[test]
    fn lex_bad_char() {
        let err = Lexer::new("a @ b").tokenize().unwrap_err();
        assert_eq!(err.offset, 2);
    }
}
