use crate::compiler::RatError;
use crate::compiler::RatSource;
use crate::compiler::{Category, Position, Span, Token};
use std::collections::VecDeque;

pub struct Lexer {
    source: RatSource,
    errors: Vec<RatError>,
}

impl Lexer {
    pub fn init(source: RatSource) -> Self {
        Lexer {
            source,
            errors: Vec::new(),
        }
    }

    pub fn tokens(&mut self) -> VecDeque<Result<Token, RatError>> {
        let mut deque = VecDeque::new();
        while let Some(result) = match self.next() {
            Ok(Some(token)) => Some(Ok(token)),
            Err(err) => Some(Err(err)),
            Ok(None) => None,
        } {
            deque.push_back(result);
        }
        deque
    }

    pub fn errors(&mut self) -> Vec<RatError> {
        self.errors.clone()
    }

    pub fn next(&mut self) -> Result<Option<Token>, RatError> {
        match self.advance_token() {
            Err(RatError::LexicalError(info)) => {
                // Uniform recovery: consume to end of line, producing one
                // invalid token whose value spans from the start of the bad
                // token to the next newline/EOF.
                let start_span = info.span.clone();
                let mut value = info.value.clone();
                self.errors.push(RatError::LexicalError(info));
                while !matches!(self.peek()?, Some('\n') | None) {
                    value.push(self.read()?.unwrap());
                }
                let span = Span::set(start_span.start, self.source.position());
                Ok(Some(Token::new(Category::Invalid, value, span)))
            }
            other => other,
        }
    }

    fn advance_token(&mut self) -> Result<Option<Token>, RatError> {
        if !self.advance_whitespace()? {
            return Ok(None);
        }
        match self.peek()? {
            None => return Ok(None),
            Some('"') => return self.advance_string_literal(),
            Some('\'') => return self.advance_character_literal(),
            Some(ch) if ch.is_numeric() => return self.advance_numeric_literal(),
            Some(_) => {}
        }
        let start_pos = self.source.position();
        let mut buf = String::new();
        loop {
            let Some(ch) = self.read()? else {
                return Ok(None);
            };
            buf.push(ch);
            if self.should_advance(&buf)? {
                continue;
            }
            if let Some(category) = Category::is_any(&buf) {
                return self.emit(category, buf, start_pos);
            }
            match self.peek()? {
                Some(ch) if Category::is_delimiter(ch) => {
                    return self.emit(Category::Identifier, buf, start_pos);
                }
                None => return self.emit(Category::Identifier, buf, start_pos),
                Some(_) => {}
            }
        }
    }

    fn advance_escape_sequence(
        &mut self,
        buf: &mut String,
        start_pos: Position,
    ) -> Result<(), RatError> {
        match self.read()? {
            None => {
                return Err(RatError::lexical(
                    "unterminated escape sequence at EOF",
                    buf.as_str(),
                    Span::set(start_pos, self.source.position()),
                ))
            }
            Some(ch) => match ch {
                '\\' | '\'' | '"' | 'n' | 'r' | 't' | 'b' | '0' => buf.push(ch),
                'u' => {
                    buf.push(ch);
                    let unicode = self.advance_unicode_escape(buf, start_pos)?;
                    buf.push_str(&unicode);
                }
                _ => {
                    buf.push(ch);
                    return Err(RatError::lexical(
                        format!("invalid escape sequence: '\\{}'", ch),
                        buf.as_str(),
                        Span::set(start_pos, self.source.position()),
                    ));
                }
            },
        }
        Ok(())
    }

    fn advance_unicode_escape(
        &mut self,
        buf: &str,
        start_pos: Position,
    ) -> Result<String, RatError> {
        match self.read()? {
            Some('{') => {}
            _ => {
                return Err(RatError::lexical(
                    "expected '{' after '\\u' in unicode escape",
                    buf,
                    Span::set(start_pos, self.source.position()),
                ))
            }
        }
        let mut hex = String::new();
        loop {
            match self.read()? {
                Some('}') => break,
                Some(hch) if hch.is_ascii_hexdigit() => hex.push(hch),
                Some(hch) => {
                    return Err(RatError::lexical(
                        format!("invalid hexadecimal digit in unicode escape: '{}'", hch),
                        buf,
                        Span::set(start_pos, self.source.position()),
                    ))
                }
                None => {
                    return Err(RatError::lexical(
                        "unterminated unicode escape sequence",
                        buf,
                        Span::set(start_pos, self.source.position()),
                    ))
                }
            }
        }
        if hex.is_empty() || hex.len() > 6 {
            return Err(RatError::lexical(
                format!(
                    "unicode escape must be between 1 and 6 hex digits, got {}",
                    hex.len()
                ),
                buf,
                Span::set(start_pos, self.source.position()),
            ));
        }
        let codepoint = u32::from_str_radix(&hex, 16).unwrap();
        if codepoint > 0x10FFFF {
            return Err(RatError::lexical(
                format!("unicode codepoint out of range: U+{:X}", codepoint),
                buf,
                Span::set(start_pos, self.source.position()),
            ));
        }
        if (0xD800..=0xDFFF).contains(&codepoint) {
            return Err(RatError::lexical(
                format!("unicode codepoint is a surrogate: U+{:X}", codepoint),
                buf,
                Span::set(start_pos, self.source.position()),
            ));
        }
        Ok(format!("\\u{{{}}}", hex))
    }

    fn advance_string_literal(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let mut buf = self.expect_opening_char('"', start_pos)?;
        loop {
            match self.peek()? {
                None | Some('\n') | Some('\r') => {
                    return Err(RatError::lexical(
                        "unterminated string literal",
                        &buf,
                        Span::set(start_pos, self.source.position()),
                    ))
                }
                Some(_) => {
                    let ch = self.read()?.unwrap();
                    buf.push(ch);
                    match ch {
                        '"' => return self.emit(Category::Literal, buf, start_pos),
                        '\\' => self.advance_escape_sequence(&mut buf, start_pos)?,
                        _ => {}
                    }
                }
            }
        }
    }

    fn advance_character_literal(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let mut buf = self.expect_opening_char('\'', start_pos)?;
        match self.peek()? {
            None | Some('\n') | Some('\r') => {
                return Err(RatError::lexical(
                    "unterminated character literal",
                    &buf,
                    Span::set(start_pos, self.source.position()),
                ))
            }
            Some('\'') => {
                self.read()?;
                buf.push('\'');
                return Err(RatError::lexical(
                    "empty character literal",
                    &buf,
                    Span::set(start_pos, self.source.position()),
                ));
            }
            Some('\\') => {
                self.read()?;
                buf.push('\\');
                self.advance_escape_sequence(&mut buf, start_pos)?;
            }
            Some(_) => {
                let ch = self.read()?.unwrap();
                buf.push(ch);
            }
        }
        match self.peek()? {
            Some('\'') => {
                self.read()?;
                buf.push('\'');
                self.emit(Category::Literal, buf, start_pos)
            }
            None | Some('\n') | Some('\r') => Err(RatError::lexical(
                "unterminated character literal",
                &buf,
                Span::set(start_pos, self.source.position()),
            )),
            Some(_) => {
                let bad = self.read()?.unwrap();
                buf.push(bad);
                Err(RatError::lexical(
                    "character literal must contain exactly one character",
                    &buf,
                    Span::set(start_pos, self.source.position()),
                ))
            }
        }
    }

    fn expect_opening_char(
        &mut self,
        expected: char,
        start_pos: Position,
    ) -> Result<String, RatError> {
        match self.read()? {
            Some(ch) if ch == expected => Ok(ch.to_string()),
            Some(ch) => Err(RatError::internal(
                format!("expected {:?}, got {:?}", expected, ch),
                "",
                Span::set(start_pos, self.source.position()),
            )),
            None => Err(RatError::internal(
                format!("expected {:?}, got EOF", expected),
                "",
                Span::set(start_pos, self.source.position()),
            )),
        }
    }

    fn read_numerics(&mut self, buf: &mut String) -> Result<bool, RatError> {
        let mut read_any = false;
        while matches!(self.peek()?, Some(ch) if ch.is_ascii_digit()) {
            buf.push(self.read()?.unwrap());
            read_any = true;
        }
        Ok(read_any)
    }

    fn is_int_suffix(ch: char) -> bool {
        matches!(ch, 'i' | 'c' | 'l' | 's')
    }

    fn advance_numeric_literal(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let mut buf = String::new();
        if !self.read_numerics(&mut buf)? {
            return Err(RatError::internal(
                "advance_numeric_literal called but no leading digits found",
                &buf,
                Span::set(start_pos, self.source.position()),
            ));
        }
        if matches!(self.peek()?, Some('.')) {
            self.read()?;
            buf.push('.');
            self.read_numerics(&mut buf)?;
            match self.peek()? {
                Some(ch @ ('d' | 'f')) => {
                    self.read()?;
                    buf.push(ch);
                    return self.expect_delimiter_then_emit(Category::Literal, buf, start_pos);
                }
                Some(ch) if !Category::is_delimiter(ch) => {
                    return Err(RatError::lexical(
                        format!("unexpected character {:?} after numeric literal", ch),
                        &buf,
                        Span::set(start_pos, self.source.position()),
                    ))
                }
                _ => return self.emit(Category::Literal, buf, start_pos),
            }
        }
        match self.peek()? {
            Some(ch @ ('d' | 'f')) => {
                self.read()?;
                buf.push(ch);
                self.expect_delimiter_then_emit(Category::Literal, buf, start_pos)
            }
            Some('u') => {
                self.read()?;
                buf.push('u');
                if matches!(self.peek()?, Some(ch) if Self::is_int_suffix(ch)) {
                    buf.push(self.read()?.unwrap());
                }
                self.expect_delimiter_then_emit(Category::Literal, buf, start_pos)
            }
            Some(ch) if Self::is_int_suffix(ch) => {
                self.read()?;
                buf.push(ch);
                self.expect_delimiter_then_emit(Category::Literal, buf, start_pos)
            }
            Some(ch) if !Category::is_delimiter(ch) => Err(RatError::lexical(
                format!("unexpected character {:?} in numeric literal", ch),
                &buf,
                Span::set(start_pos, self.source.position()),
            )),
            _ => self.emit(Category::Literal, buf, start_pos),
        }
    }

    fn expect_delimiter_then_emit(
        &mut self,
        category: Category,
        partial: String,
        start_pos: Position,
    ) -> Result<Option<Token>, RatError> {
        match self.peek()? {
            Some(ch) if !Category::is_delimiter(ch) => Err(RatError::lexical(
                format!("unexpected character {:?} after numeric literal", ch),
                &partial,
                Span::set(start_pos, self.source.position()),
            )),
            _ => self.emit(category, partial, start_pos),
        }
    }

    fn advance_whitespace(&mut self) -> Result<bool, RatError> {
        loop {
            match self.peek()? {
                None => return Ok(false),
                Some(ch) if !ch.is_whitespace() || ch == '\n' => return Ok(true),
                _ => {
                    self.source.read()?;
                }
            }
        }
    }

    fn should_advance(&mut self, partial: &str) -> Result<bool, RatError> {
        let Some(ch) = self.peek()? else {
            return Ok(false);
        };
        let mut extended = String::from(partial);
        extended.push(ch);
        Ok(Category::is_any(&extended).is_some())
    }

    fn read(&mut self) -> Result<Option<char>, RatError> {
        Ok(self.source.read()?)
    }

    fn peek(&mut self) -> Result<Option<char>, RatError> {
        Ok(self.source.peek()?)
    }

    fn emit(
        &mut self,
        category: Category,
        partial: String,
        start_pos: Position,
    ) -> Result<Option<Token>, RatError> {
        Ok(Some(Token::new(
            category,
            partial,
            Span::set(start_pos, self.source.position()),
        )))
    }
}
