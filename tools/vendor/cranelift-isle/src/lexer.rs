//! Lexer for the ISLE language.

use crate::error::{Error, Errors, Span};
use std::borrow::Cow;
use std::path::Path;
use std::sync::Arc;

type Result<T> = std::result::Result<T, Errors>;

/// The lexer.
///
/// Breaks source text up into a sequence of tokens (with source positions).
#[derive(Clone, Debug)]
pub struct Lexer<'a> {
    /// Arena of filenames from the input source.
    ///
    /// Indexed via `Pos::file`.
    pub filenames: Vec<Arc<str>>,

    /// Arena of file source texts.
    ///
    /// Indexed via `Pos::file`.
    pub file_texts: Vec<Arc<str>>,

    file_starts: Vec<usize>,
    buf: Cow<'a, [u8]>,
    pos: Pos,
    lookahead: Option<(Pos, Token)>,
}

/// A source position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash, PartialOrd, Ord)]
pub struct Pos {
    /// This source position's file.
    ///
    /// Indexes into `Lexer::filenames` early in the compiler pipeline, and
    /// later into `TypeEnv::filenames` once we get into semantic analysis.
    pub file: usize,
    /// This source position's byte offset in the file.
    pub offset: usize,
    /// This source position's line number in the file.
    pub line: usize,
    /// This source position's column number in the file.
    pub col: usize,
}

impl Pos {
    /// Print this source position as `file.isle line 12`.
    pub fn pretty_print_line(&self, filenames: &[Arc<str>]) -> String {
        format!("{} line {}", filenames[self.file], self.line)
    }
}

/// A token of ISLE source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    /// Left paren.
    LParen,
    /// Right paren.
    RParen,
    /// A symbol, e.g. `Foo`.
    Symbol(String),
    /// An integer.
    Int(i128),
    /// `@`
    At,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source contents and filename.
    pub fn from_str(s: &'a str, filename: &'a str) -> Result<Lexer<'a>> {
        let mut l = Lexer {
            filenames: vec![filename.into()],
            file_texts: vec![s.into()],
            file_starts: vec![0],
            buf: Cow::Borrowed(s.as_bytes()),
            pos: Pos {
                file: 0,
                offset: 0,
                line: 1,
                col: 0,
            },
            lookahead: None,
        };
        l.reload()?;
        Ok(l)
    }

    /// Create a new lexer from the given files.
    pub fn from_files<P>(file_paths: impl IntoIterator<Item = P>) -> Result<Lexer<'a>>
    where
        P: AsRef<Path>,
    {
        let mut filenames = Vec::<Arc<str>>::new();
        let mut file_texts = Vec::<Arc<str>>::new();

        for f in file_paths {
            let f = f.as_ref();

            filenames.push(f.display().to_string().into());

            let s = std::fs::read_to_string(f)
                .map_err(|e| Errors::from_io(e, format!("failed to read file: {}", f.display())))?;
            file_texts.push(s.into());
        }

        assert!(!filenames.is_empty());

        let mut file_starts = vec![];
        let mut buf = String::new();
        for text in &file_texts {
            file_starts.push(buf.len());
            buf += &text;
            buf += "\n";
        }

        let mut l = Lexer {
            filenames,
            file_texts,
            buf: Cow::Owned(buf.into_bytes()),
            file_starts,
            pos: Pos {
                file: 0,
                offset: 0,
                line: 1,
                col: 0,
            },
            lookahead: None,
        };
        l.reload()?;
        Ok(l)
    }

    /// Get the lexer's current source position.
    pub fn pos(&self) -> Pos {
        Pos {
            file: self.pos.file,
            offset: self.pos.offset - self.file_starts[self.pos.file],
            line: self.pos.line,
            col: self.pos.file,
        }
    }

    fn advance_pos(&mut self) {
        self.pos.col += 1;
        if self.buf[self.pos.offset] == b'\n' {
            self.pos.line += 1;
            self.pos.col = 0;
        }
        self.pos.offset += 1;
        if self.pos.file + 1 < self.file_starts.len() {
            let next_start = self.file_starts[self.pos.file + 1];
            if self.pos.offset >= next_start {
                assert!(self.pos.offset == next_start);
                self.pos.file += 1;
                self.pos.line = 1;
            }
        }
    }

    fn error(&self, pos: Pos, msg: impl Into<String>) -> Errors {
        Errors {
            errors: vec![Error::ParseError {
                msg: msg.into(),
                span: Span::new_single(pos),
            }],
            filenames: self.filenames.clone(),
            file_texts: self.file_texts.clone(),
        }
    }

    fn next_token(&mut self) -> Result<Option<(Pos, Token)>> {
        fn is_sym_first_char(c: u8) -> bool {
            match c {
                b'-' | b'0'..=b'9' | b'(' | b')' | b';' | b'<' | b'>' => false,
                c if c.is_ascii_whitespace() => false,
                _ => true,
            }
        }
        fn is_sym_other_char(c: u8) -> bool {
            match c {
                b'(' | b')' | b';' | b'@' | b'<' => false,
                c if c.is_ascii_whitespace() => false,
                _ => true,
            }
        }

        // Skip any whitespace and any comments.
        while self.pos.offset < self.buf.len() {
            if self.buf[self.pos.offset].is_ascii_whitespace() {
                self.advance_pos();
                continue;
            }
            if self.buf[self.pos.offset] == b';' {
                while self.pos.offset < self.buf.len() && self.buf[self.pos.offset] != b'\n' {
                    self.advance_pos();
                }
                continue;
            }
            break;
        }

        if self.pos.offset == self.buf.len() {
            return Ok(None);
        }

        let char_pos = self.pos();
        match self.buf[self.pos.offset] {
            b'(' => {
                self.advance_pos();
                Ok(Some((char_pos, Token::LParen)))
            }
            b')' => {
                self.advance_pos();
                Ok(Some((char_pos, Token::RParen)))
            }
            b'@' => {
                self.advance_pos();
                Ok(Some((char_pos, Token::At)))
            }
            c if is_sym_first_char(c) => {
                let start = self.pos.offset;
                let start_pos = self.pos();
                while self.pos.offset < self.buf.len()
                    && is_sym_other_char(self.buf[self.pos.offset])
                {
                    self.advance_pos();
                }
                let end = self.pos.offset;
                let s = std::str::from_utf8(&self.buf[start..end])
                    .expect("Only ASCII characters, should be UTF-8");
                debug_assert!(!s.is_empty());
                Ok(Some((start_pos, Token::Symbol(s.to_string()))))
            }
            c if (c >= b'0' && c <= b'9') || c == b'-' => {
                let start_pos = self.pos();
                let neg = if c == b'-' {
                    self.advance_pos();
                    true
                } else {
                    false
                };

                let mut radix = 10;

                // Check for hex literals.
                if self.buf.get(self.pos.offset).copied() == Some(b'0')
                    && (self.buf.get(self.pos.offset + 1).copied() == Some(b'x')
                        || self.buf.get(self.pos.offset + 1).copied() == Some(b'X'))
                {
                    self.advance_pos();
                    self.advance_pos();
                    radix = 16;
                }

                // Find the range in the buffer for this integer literal. We'll
                // pass this range to `i64::from_str_radix` to do the actual
                // string-to-integer conversion.
                let mut s = vec![];
                while self.pos.offset < self.buf.len()
                    && ((radix == 10
                        && self.buf[self.pos.offset] >= b'0'
                        && self.buf[self.pos.offset] <= b'9')
                        || (radix == 16
                            && ((self.buf[self.pos.offset] >= b'0'
                                && self.buf[self.pos.offset] <= b'9')
                                || (self.buf[self.pos.offset] >= b'a'
                                    && self.buf[self.pos.offset] <= b'f')
                                || (self.buf[self.pos.offset] >= b'A'
                                    && self.buf[self.pos.offset] <= b'F')))
                        || self.buf[self.pos.offset] == b'_')
                {
                    if self.buf[self.pos.offset] != b'_' {
                        s.push(self.buf[self.pos.offset]);
                    }
                    self.advance_pos();
                }
                let s_utf8 = std::str::from_utf8(&s[..]).unwrap();

                // Support either signed range (-2^127..2^127) or
                // unsigned range (0..2^128).
                let num = i128::from_str_radix(s_utf8, radix)
                    .or_else(|_| u128::from_str_radix(s_utf8, radix).map(|val| val as i128))
                    .map_err(|e| self.error(start_pos, e.to_string()))?;

                let tok = if neg {
                    Token::Int(num.checked_neg().ok_or_else(|| {
                        self.error(start_pos, "integer literal cannot fit in i128")
                    })?)
                } else {
                    Token::Int(num)
                };
                Ok(Some((start_pos, tok)))
            }
            c => Err(self.error(self.pos, format!("Unexpected character '{}'", c))),
        }
    }

    /// Get the next token from this lexer's token stream, if any.
    pub fn next(&mut self) -> Result<Option<(Pos, Token)>> {
        let tok = self.lookahead.take();
        self.reload()?;
        Ok(tok)
    }

    fn reload(&mut self) -> Result<()> {
        if self.lookahead.is_none() && self.pos.offset < self.buf.len() {
            self.lookahead = self.next_token()?;
        }
        Ok(())
    }

    /// Peek ahead at the next token.
    pub fn peek(&self) -> Option<&(Pos, Token)> {
        self.lookahead.as_ref()
    }

    /// Are we at the end of the source input?
    pub fn eof(&self) -> bool {
        self.lookahead.is_none()
    }
}

impl Token {
    /// Is this an `Int` token?
    pub fn is_int(&self) -> bool {
        match self {
            Token::Int(_) => true,
            _ => false,
        }
    }

    /// Is this a `Sym` token?
    pub fn is_sym(&self) -> bool {
        match self {
            Token::Symbol(_) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn lex(s: &str, file: &str) -> Vec<Token> {
        let mut toks = vec![];
        let mut lexer = Lexer::from_str(s, file).unwrap();
        while let Some((_, tok)) = lexer.next().unwrap() {
            toks.push(tok);
        }
        toks
    }

    #[test]
    fn lexer_basic() {
        assert_eq!(
            lex(
                ";; comment\n; another\r\n   \t(one two three 23 -568  )\n",
                "lexer_basic"
            ),
            vec![
                Token::LParen,
                Token::Symbol("one".to_string()),
                Token::Symbol("two".to_string()),
                Token::Symbol("three".to_string()),
                Token::Int(23),
                Token::Int(-568),
                Token::RParen
            ]
        );
    }

    #[test]
    fn ends_with_sym() {
        assert_eq!(
            lex("asdf", "ends_with_sym"),
            vec![Token::Symbol("asdf".to_string()),]
        );
    }

    #[test]
    fn ends_with_num() {
        assert_eq!(lex("23", "ends_with_num"), vec![Token::Int(23)],);
    }

    #[test]
    fn weird_syms() {
        assert_eq!(
            lex("(+ [] => !! _test!;comment\n)", "weird_syms"),
            vec![
                Token::LParen,
                Token::Symbol("+".to_string()),
                Token::Symbol("[]".to_string()),
                Token::Symbol("=>".to_string()),
                Token::Symbol("!!".to_string()),
                Token::Symbol("_test!".to_string()),
                Token::RParen,
            ]
        );
    }
}
