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
        std::iter::from_fn(|| self.next().transpose()).collect()
    }

    pub fn errors(&self) -> &[RatError] {
        &self.errors
    }

    pub fn next(&mut self) -> Result<Option<Token>, RatError> {
        match self.advance_token() {
            Err(RatError::LexicalError(info)) => {
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
            None => Ok(None),
            Some('"') => self.advance_string_literal(),
            Some('\'') => self.advance_character_literal(),
            Some(ch) if ch.is_numeric() => self.advance_numeric_literal(),
            _ => self.advance_symbol_or_identifier(),
        }
    }

    fn advance_symbol_or_identifier(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let mut buf = String::new();
        loop {
            let Some(ch) = self.read()? else {
                return self.emit_identifier_or_none(buf, start_pos);
            };
            buf.push(ch);

            if self.next_char_extends_symbol(&buf)? {
                continue;
            }

            if let Some(category) = Category::is_any(&buf) {
                return self.emit(category, buf, start_pos);
            }

            if matches!(self.peek()?, Some(ch) if Category::is_delimiter(ch))
                || self.peek()?.is_none()
            {
                return self.emit(Category::Identifier, buf, start_pos);
            }
        }
    }

    fn emit_identifier_or_none(
        &mut self,
        buf: String,
        start_pos: Position,
    ) -> Result<Option<Token>, RatError> {
        if buf.is_empty() {
            Ok(None)
        } else {
            self.emit(Category::Identifier, buf, start_pos)
        }
    }

    // (\d+)(((\.(\d+))?(\d*)(d|f)?)|(u)?[icls]?)?
    fn advance_numeric_literal(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let mut buf = String::new();

        if !self.read_ascii_digits(&mut buf)? {
            return Err(RatError::internal(
                "advance_numeric_literal() called but no leading digits found",
                &buf,
                Span::set(start_pos, self.source.position()),
            ));
        }

        if matches!(self.peek()?, Some('.')) {
            self.read()?;
            buf.push('.');
            if !self.read_ascii_digits(&mut buf)? {
                return Err(RatError::lexical(
                    "expected digits after '.' in numeric literal",
                    &buf,
                    Span::set(start_pos, self.source.position()),
                ));
            }
            self.read_ascii_digits(&mut buf)?;
            return self.numeric_suffix(&mut buf, start_pos, true);
        }

        self.numeric_suffix(&mut buf, start_pos, false)
    }

    fn numeric_suffix(
        &mut self,
        buf: &mut String,
        start_pos: Position,
        is_floating_type: bool,
    ) -> Result<Option<Token>, RatError> {
        let is_int_suffix = |ch: char| matches!(ch, 'i' | 'c' | 'l' | 's');

        match self.peek()? {
            Some(ch @ ('d' | 'f')) => {
                self.read()?;
                buf.push(ch);
                self.expect_delimiter_then_emit(Category::Literal, buf.clone(), start_pos)
            }
            Some('u') if !is_floating_type => {
                self.read()?;
                buf.push('u');
                if matches!(self.peek()?, Some(ch) if is_int_suffix(ch)) {
                    buf.push(self.read()?.unwrap());
                }
                self.expect_delimiter_then_emit(Category::Literal, buf.clone(), start_pos)
            }
            Some(ch) if !is_floating_type && is_int_suffix(ch) => {
                self.read()?;
                buf.push(ch);
                self.expect_delimiter_then_emit(Category::Literal, buf.clone(), start_pos)
            }
            Some(ch) if !Category::is_delimiter(ch) => Err(RatError::lexical(
                format!("unexpected character {:?} after numeric literal", ch),
                buf.as_str(),
                Span::set(start_pos, self.source.position()),
            )),
            _ => self.emit(Category::Literal, buf.clone(), start_pos),
        }
    }

    fn expect_delimiter_then_emit(
        &mut self,
        category: Category,
        buf: String,
        start_pos: Position,
    ) -> Result<Option<Token>, RatError> {
        match self.peek()? {
            Some(ch) if !Category::is_delimiter(ch) => Err(RatError::lexical(
                format!("unexpected character {:?} after numeric literal", ch),
                &buf,
                Span::set(start_pos, self.source.position()),
            )),
            _ => self.emit(category, buf, start_pos),
        }
    }

    fn read_ascii_digits(&mut self, buf: &mut String) -> Result<bool, RatError> {
        let mut any = false;
        while matches!(self.peek()?, Some(ch) if ch.is_ascii_digit()) {
            buf.push(self.read()?.unwrap());
            any = true;
        }
        Ok(any)
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
                buf.push(self.read()?.unwrap());
                return Err(RatError::lexical(
                    "empty character literal",
                    &buf,
                    Span::set(start_pos, self.source.position()),
                ));
            }
            Some('\\') => {
                buf.push(self.read()?.unwrap());
                self.advance_escape_sequence(&mut buf, start_pos)?;
            }
            Some(_) => {
                buf.push(self.read()?.unwrap());
            }
        }

        match self.peek()? {
            Some('\'') => {
                buf.push(self.read()?.unwrap());
                self.emit(Category::Literal, buf, start_pos)
            }
            None | Some('\n') | Some('\r') => Err(RatError::lexical(
                "unterminated character literal",
                &buf,
                Span::set(start_pos, self.source.position()),
            )),
            Some(_) => {
                buf.push(self.read()?.unwrap());
                Err(RatError::lexical(
                    "character literal must contain exactly one character",
                    &buf,
                    Span::set(start_pos, self.source.position()),
                ))
            }
        }
    }

    fn advance_escape_sequence(
        &mut self,
        buf: &mut String,
        start_pos: Position,
    ) -> Result<(), RatError> {
        let ch = match self.read()? {
            None => {
                return Err(RatError::lexical(
                    "unterminated escape sequence at EOF",
                    buf.as_str(),
                    Span::set(start_pos, self.source.position()),
                ))
            }
            Some(ch) => ch,
        };
        buf.push(ch);
        match ch {
            '\\' | '\'' | '"' | 'n' | 'r' | 't' | 'b' | '0' => {}
            'u' => {
                let unicode = self.advance_unicode_escape(buf, start_pos)?;
                buf.push_str(&unicode);
            }
            _ => {
                return Err(RatError::lexical(
                    format!("invalid escape sequence: '\\{}'", ch),
                    buf.as_str(),
                    Span::set(start_pos, self.source.position()),
                ))
            }
        }
        Ok(())
    }

    fn advance_unicode_escape(
        &mut self,
        buf: &str,
        start_pos: Position,
    ) -> Result<String, RatError> {
        if !matches!(self.read()?, Some('{')) {
            return Err(RatError::lexical(
                "expected '{' after '\\u' in unicode escape",
                buf,
                Span::set(start_pos, self.source.position()),
            ));
        }

        let mut hex = String::new();
        loop {
            match self.read()? {
                Some('}') => break,
                Some(ch) if ch.is_ascii_hexdigit() => hex.push(ch),
                Some(ch) => {
                    return Err(RatError::lexical(
                        format!("invalid hexadecimal digit in unicode escape: '{}'", ch),
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

    fn next_char_extends_symbol(&mut self, partial: &str) -> Result<bool, RatError> {
        let Some(ch) = self.peek()? else {
            return Ok(false);
        };
        let mut extended = partial.to_owned();
        extended.push(ch);
        Ok(Category::is_any(&extended).is_some())
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

    fn read(&mut self) -> Result<Option<char>, RatError> {
        self.source.read().map_err(Into::into)
    }

    fn peek(&mut self) -> Result<Option<char>, RatError> {
        self.source.peek().map_err(Into::into)
    }

    fn emit(
        &mut self,
        category: Category,
        value: String,
        start_pos: Position,
    ) -> Result<Option<Token>, RatError> {
        Ok(Some(Token::new(
            category,
            value,
            Span::set(start_pos, self.source.position()),
        )))
    }
}
