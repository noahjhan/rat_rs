use crate::compiler::{Category, ErrorKind, Position, RatError, RatSource, Span, Token};
use std::collections::VecDeque;

pub struct Lexer {
    source: RatSource,
    errors: Vec<RatError>,
    poisoned: bool,
}

impl Lexer {
    pub fn init(source: RatSource) -> Self {
        Lexer {
            source,
            errors: Vec::new(),
            poisoned: false,
        }
    }

    pub fn get_tokens(&mut self) -> VecDeque<Result<Token, RatError>> {
        std::iter::from_fn(|| self.next().transpose()).collect()
    }

    pub fn errors(&self) -> &[RatError] {
        &self.errors
    }

    pub fn next(&mut self) -> Result<Option<Token>, RatError> {
        if self.poisoned {
            return Ok(None);
        }
        match self.advance_token() {
            Err(err) if err.kind == ErrorKind::Lexical => {
                let start = err.span.start;
                let mut value = err.value.clone();
                self.errors.push(err);

                while !matches!(self.peek()?, Some('\n') | None) {
                    value.push(self.read()?.unwrap());
                }

                let span = Span::new(start, self.source.position());
                Ok(Some(Token::new(Category::Invalid, value, span)))
            }
            Err(err) => {
                self.poisoned = true;
                Err(err)
            }
            other => other,
        }
    }

    fn advance_token(&mut self) -> Result<Option<Token>, RatError> {
        if !self.skip_non_newline_whitespace()? {
            return Ok(None);
        }
        match self.peek()? {
            None => Ok(None),
            Some('"') => self.advance_string_literal(),
            Some('\'') => self.advance_character_literal(),
            Some(ch) if ch.is_ascii_digit() => self.advance_numeric_literal(),
            _ => self.advance_symbol_or_identifier(),
        }
    }

    fn skip_non_newline_whitespace(&mut self) -> Result<bool, RatError> {
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

    fn advance_symbol_or_identifier(&mut self) -> Result<Option<Token>, RatError> {
        let start = self.source.position();
        let mut buf = String::new();

        loop {
            let Some(ch) = self.read()? else {
                return self.emit_identifier_or_none(buf, start);
            };
            buf.push(ch);

            if self.next_char_extends_token(&buf)? {
                continue;
            }

            if let Some(category) = Category::is_any(&buf) {
                return self.emit(category, buf, start);
            }

            if matches!(self.peek()?, Some(ch) if Category::is_delimiter(ch))
                || self.peek()?.is_none()
            {
                return self.emit(Category::Identifier, buf, start);
            }
        }
    }

    fn emit_identifier_or_none(
        &mut self,
        buf: String,
        start: Position,
    ) -> Result<Option<Token>, RatError> {
        if buf.is_empty() {
            Ok(None)
        } else {
            self.emit(Category::Identifier, buf, start)
        }
    }

    fn next_char_extends_token(&mut self, partial: &str) -> Result<bool, RatError> {
        let Some(ch) = self.peek()? else {
            return Ok(false);
        };

        let mut tmp = [0u8; 32];
        let base = partial.as_bytes();
        let ch_len = ch.len_utf8();
        if base.len() + ch_len <= tmp.len() {
            tmp[..base.len()].copy_from_slice(base);
            ch.encode_utf8(&mut tmp[base.len()..]);
            let extended = std::str::from_utf8(&tmp[..base.len() + ch_len]).unwrap();
            Ok(Category::is_any(extended).is_some())
        } else {
            let mut s = partial.to_owned();
            s.push(ch);
            Ok(Category::is_any(&s).is_some())
        }
    }

    // regex: (\d+)(((\.(\d+))?(d|f)?)|(u)?[icls]?)?
    fn advance_numeric_literal(&mut self) -> Result<Option<Token>, RatError> {
        let start = self.source.position();
        let mut buf = String::new();

        if !self.read_ascii_digits(&mut buf)? {
            return Err(RatError::internal(
                "advance_numeric_literal() no leading numeric digit",
                &buf,
                Span::new(start, self.source.position()),
            ));
        }

        if matches!(self.peek()?, Some('.')) {
            self.read()?;
            buf.push('.');
            if !self.read_ascii_digits(&mut buf)? {
                return Err(RatError::lexical(
                    "expected digits after '.' in numeric literal",
                    &buf,
                    Span::new(start, self.source.position()),
                ));
            }
            return self.emit_numeric_suffix(&mut buf, start, true);
        }

        self.emit_numeric_suffix(&mut buf, start, false)
    }

    fn emit_numeric_suffix(
        &mut self,
        buf: &mut String,
        start: Position,
        is_float: bool,
    ) -> Result<Option<Token>, RatError> {
        let is_int_suffix = |ch: char| matches!(ch, 'i' | 'c' | 'l' | 's');

        match self.peek()? {
            Some(ch @ ('d' | 'f')) => {
                self.read()?;
                buf.push(ch);
                self.expect_delimiter_then_emit(buf.clone(), start)
            }
            Some('u') if !is_float => {
                self.read()?;
                buf.push('u');
                if matches!(self.peek()?, Some(ch) if is_int_suffix(ch)) {
                    buf.push(self.read()?.unwrap());
                }
                self.expect_delimiter_then_emit(buf.clone(), start)
            }
            Some(ch) if !is_float && is_int_suffix(ch) => {
                self.read()?;
                buf.push(ch);
                self.expect_delimiter_then_emit(buf.clone(), start)
            }
            Some(ch) if !Category::is_delimiter(ch) => Err(RatError::lexical(
                format!("unexpected character {:?} after numeric literal", ch),
                buf.as_str(),
                Span::new(start, self.source.position()),
            )),
            _ => self.emit(Category::Literal, buf.clone(), start),
        }
    }

    fn expect_delimiter_then_emit(
        &mut self,
        buf: String,
        start: Position,
    ) -> Result<Option<Token>, RatError> {
        match self.peek()? {
            Some(ch) if !Category::is_delimiter(ch) => Err(RatError::lexical(
                format!("unexpected character {:?} after numeric literal", ch),
                &buf,
                Span::new(start, self.source.position()),
            )),
            _ => self.emit(Category::Literal, buf, start),
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
        let start = self.source.position();
        let mut buf = self.expect_opening_char('"', start)?;

        loop {
            match self.peek()? {
                None | Some('\n') | Some('\r') => {
                    return Err(RatError::lexical(
                        "unterminated string literal",
                        &buf,
                        Span::new(start, self.source.position()),
                    ))
                }
                Some(_) => {
                    let ch = self.read()?.unwrap();
                    buf.push(ch);
                    match ch {
                        '"' => return self.emit(Category::Literal, buf, start),
                        '\\' => self.advance_escape_sequence(&mut buf, start)?,
                        _ => {}
                    }
                }
            }
        }
    }

    fn advance_character_literal(&mut self) -> Result<Option<Token>, RatError> {
        let start = self.source.position();
        let mut buf = self.expect_opening_char('\'', start)?;

        match self.peek()? {
            None | Some('\n') | Some('\r') => {
                return Err(RatError::lexical(
                    "unterminated character literal",
                    &buf,
                    Span::new(start, self.source.position()),
                ))
            }
            Some('\'') => {
                buf.push(self.read()?.unwrap());
                return Err(RatError::lexical(
                    "empty character literal",
                    &buf,
                    Span::new(start, self.source.position()),
                ));
            }
            Some('\\') => {
                buf.push(self.read()?.unwrap());
                self.advance_escape_sequence(&mut buf, start)?;
            }
            Some(_) => {
                buf.push(self.read()?.unwrap());
            }
        }

        match self.peek()? {
            Some('\'') => {
                buf.push(self.read()?.unwrap());
                self.emit(Category::Literal, buf, start)
            }
            None | Some('\n') | Some('\r') => Err(RatError::lexical(
                "unterminated character literal",
                &buf,
                Span::new(start, self.source.position()),
            )),
            Some(_) => {
                buf.push(self.read()?.unwrap());
                Err(RatError::lexical(
                    "character literal should contain only one character",
                    &buf,
                    Span::new(start, self.source.position()),
                ))
            }
        }
    }

    fn advance_escape_sequence(
        &mut self,
        buf: &mut String,
        start: Position,
    ) -> Result<(), RatError> {
        let ch = self.read()?.ok_or_else(|| {
            RatError::lexical(
                "unterminated escape sequence at EOF",
                buf.as_str(),
                Span::new(start, self.source.position()),
            )
        })?;
        buf.push(ch);
        match ch {
            '\\' | '\'' | '"' | 'n' | 'r' | 't' | 'b' | '0' => {}
            'u' => {
                let s = self.advance_unicode_escape(buf, start)?;
                buf.push_str(&s);
            }
            _ => {
                return Err(RatError::lexical(
                    format!("invalid escape sequence: '\\{}'", ch),
                    buf.as_str(),
                    Span::new(start, self.source.position()),
                ))
            }
        }
        Ok(())
    }

    fn advance_unicode_escape(&mut self, buf: &str, start: Position) -> Result<String, RatError> {
        if !matches!(self.read()?, Some('{')) {
            return Err(RatError::lexical(
                "expected '{' after '\\u' in unicode escape",
                buf,
                Span::new(start, self.source.position()),
            ));
        }

        let mut hex = String::with_capacity(6);
        loop {
            match self.read()? {
                Some('}') => break,
                Some(ch) if ch.is_ascii_hexdigit() => hex.push(ch),
                Some(ch) => {
                    return Err(RatError::lexical(
                        format!("invalid hex digit in unicode escape: {:?}", ch),
                        buf,
                        Span::new(start, self.source.position()),
                    ))
                }
                None => {
                    return Err(RatError::lexical(
                        "unterminated unicode escape sequence",
                        buf,
                        Span::new(start, self.source.position()),
                    ))
                }
            }
        }

        if hex.is_empty() || hex.len() > 6 {
            return Err(RatError::lexical(
                format!("unicode escape must be 1–6 hex digits, got {}", hex.len()),
                buf,
                Span::new(start, self.source.position()),
            ));
        }

        let codepoint = u32::from_str_radix(&hex, 16).unwrap();

        if codepoint > 0x10_FFFF {
            return Err(RatError::lexical(
                format!("unicode codepoint out of range: U+{:06X}", codepoint),
                buf,
                Span::new(start, self.source.position()),
            ));
        }
        if (0xD800..=0xDFFF).contains(&codepoint) {
            return Err(RatError::lexical(
                format!(
                    "unicode surrogate codepoint is not allowed: U+{:04X}",
                    codepoint
                ),
                buf,
                Span::new(start, self.source.position()),
            ));
        }

        Ok(format!("\\u{{{}}}", hex))
    }

    fn expect_opening_char(&mut self, expected: char, start: Position) -> Result<String, RatError> {
        match self.read()? {
            Some(ch) if ch == expected => Ok(ch.to_string()),
            Some(ch) => Err(RatError::internal(
                format!("expected {:?}, got {:?}", expected, ch),
                "",
                Span::new(start, self.source.position()),
            )),
            None => Err(RatError::internal(
                format!("expected {:?}, got EOF", expected),
                "",
                Span::new(start, self.source.position()),
            )),
        }
    }

    #[inline]
    fn read(&mut self) -> Result<Option<char>, RatError> {
        self.source.read()
    }

    #[inline]
    fn peek(&mut self) -> Result<Option<char>, RatError> {
        self.source.peek()
    }

    fn emit(
        &mut self,
        category: Category,
        value: String,
        start: Position,
    ) -> Result<Option<Token>, RatError> {
        Ok(Some(Token::new(
            category,
            value,
            Span::new(start, self.source.position()),
        )))
    }
}
