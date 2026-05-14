use crate::compiler::RatError;
use crate::compiler::RatSource;
use crate::compiler::{Category, Position, Span, Token};

// TODO:
// multi-line strings
// single-line comments
// multi-line comments

pub struct Lexer {
    source: RatSource,
}

impl Lexer {
    pub fn init(source: RatSource) -> Self {
        Lexer { source }
    }

    // public interface for processing tokens
    // wrapper for advance_token() with error recovery
    pub fn next(&mut self) -> Result<Option<Token>, RatError> {
        match self.advance_token() {
            Err(err @ RatError::LexicalError(_)) => {
                let _ = self.read_until_delimiter();
                Err(err)
            }
            other => other,
        }
    }

    // returns and consumes next token in source file
    fn advance_token(&mut self) -> Result<Option<Token>, RatError> {
        match self.peek()? {
            None => return Ok(None),
            Some('\"') => return self.advance_string_literal(),
            Some('\'') => return self.advance_character_literal(),
            Some(ch) if ch.is_numeric() => return self.advance_numeric_literal(),
            Some(_) => {}
        }

        if !self.advance_whitespace()? {
            return Ok(None);
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

    // fills buffer with the current escape sequence
    fn advance_escape_sequence(
        &mut self,
        buf: &mut String,
        start_pos: Position,
    ) -> Result<(), RatError> {
        let ch = match self.read()? {
            Some(ch) => ch,
            None => {
                return Err(RatError::lexical(
                    "unterminated escape sequence at EOF",
                    buf.as_str(),
                    Span::set(start_pos, self.source.position()),
                ))
            }
        };

        match ch {
            '\\' | '\'' | '\"' | 'n' | 'r' | 't' | 'b' | '0' => {
                buf.push(ch);
            }
            'u' => {
                let unicode = self.advance_unicode_escape(buf, start_pos)?;
                buf.push_str(&unicode);
            }
            _ => {
                return Err(RatError::lexical(
                    format!("invalid escape sequence: '\\{}'", ch),
                    buf.as_str(),
                    Span::set(start_pos, self.source.position()),
                ));
            }
        }
        Ok(())
    }

    // single-line string literals
    fn advance_string_literal(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let mut buf = self.expect_opening_char('"', start_pos)?;

        loop {
            let ch = match self.read()? {
                Some(ch) => ch,
                None => {
                    return Err(RatError::lexical(
                        "missing '\"' at the end of string literal",
                        &buf,
                        Span::set(start_pos, self.source.position()),
                    ))
                }
            };

            buf.push(ch);
            match ch {
                '\\' => self.advance_escape_sequence(&mut buf, start_pos)?,
                '\"' => return self.emit(Category::Literal, buf, start_pos),
                _ => {}
            }
        }
    }

    fn advance_character_literal(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let mut buf = self.expect_opening_char('\'', start_pos)?;

        let ch = match self.read()? {
            Some(ch) => ch,
            None => {
                return Err(RatError::lexical(
                    "unterminated character literal",
                    &buf,
                    Span::set(start_pos, self.source.position()),
                ))
            }
        };

        match ch {
            '\\' => {
                self.advance_escape_sequence(&mut buf, start_pos)?;
            }
            '\'' => {
                return Err(RatError::lexical(
                    "empty character literal",
                    &buf,
                    Span::set(start_pos, self.source.position()),
                ));
            }
            _ => {
                buf.push(ch);
            }
        }

        match self.read()? {
            Some('\'') => self.emit(Category::Literal, buf, start_pos),
            Some(_) => Err(RatError::lexical(
                "character literal must contain exactly one character",
                &buf,
                Span::set(start_pos, self.source.position()),
            )),
            None => Err(RatError::lexical(
                "unterminated character literal",
                &buf,
                Span::set(start_pos, self.source.position()),
            )),
        }
    }

    // internal error upon malformed function call
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

    // rust style unicode escape sequence
    fn advance_unicode_escape(
        &mut self,
        partial: &str,
        start_pos: Position,
    ) -> Result<String, RatError> {
        if !matches!(self.read()?, Some('{')) {
            return Err(RatError::lexical(
                "expected '{' after '\\u' in unicode escape",
                partial,
                Span::set(start_pos, self.source.position()),
            ));
        }

        let mut hex = String::new();
        loop {
            match self.read()? {
                Some('}') => break,
                Some(hch) if hch.is_ascii_hexdigit() => hex.push(hch),
                Some(hch) => {
                    return Err(RatError::lexical(
                        format!("invalid hexadecimal digit in unicode escape: '{}'", hch),
                        partial,
                        Span::set(start_pos, self.source.position()),
                    ));
                }
                None => {
                    return Err(RatError::lexical(
                        "unterminated unicode escape sequence",
                        partial,
                        Span::set(start_pos, self.source.position()),
                    ));
                }
            }
        }

        if hex.is_empty() || hex.len() > 6 {
            return Err(RatError::lexical(
                format!(
                    "unicode escape must be between 1 and 6 hex digits, got {}",
                    hex.len()
                ),
                partial,
                Span::set(start_pos, self.source.position()),
            ));
        }

        let codepoint = u32::from_str_radix(&hex, 16).unwrap();

        if codepoint > 0x10FFFF {
            return Err(RatError::lexical(
                format!("unicode codepoint out of range: U+{:X}", codepoint),
                partial,
                Span::set(start_pos, self.source.position()),
            ));
        }
        if (0xD800..=0xDFFF).contains(&codepoint) {
            return Err(RatError::lexical(
                format!("unicode codepoint is a surrogate: U+{:X}", codepoint),
                partial,
                Span::set(start_pos, self.source.position()),
            ));
        }

        Ok(format!("\\u{{{}}}", hex))
    }

    // fill buffer until peek() returns a non ascii digit
    fn read_numerics(&mut self, buf: &mut String) -> Result<bool, RatError> {
        let mut read_any = false;
        while let Some(ch) = self.peek()? {
            if !ch.is_ascii_digit() {
                break;
            }
            self.read()?;
            buf.push(ch);
            read_any = true;
        }
        Ok(read_any)
    }

    fn is_int_suffix(ch: char) -> bool {
        matches!(ch, 'i' | 'c' | 'l' | 's')
    }

    // regex: (\d+)(((\.)?(\d*)?(d|f)?)|(u)?[icls]?)?)
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
                    ));
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
                if let Some(ch) = self.peek()? {
                    if Self::is_int_suffix(ch) {
                        self.read()?;
                        buf.push(ch);
                    }
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

    // when at the end of a token, require next char to be a delimiter
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

    // error recovery, try to read ahead as far as possible
    fn read_until_delimiter(&mut self) -> Result<(), RatError> {
        loop {
            match self.peek()? {
                Some(ch) if Category::is_delimiter(ch) => return Ok(()),
                None => return Ok(()),
                _ => {
                    self.read()?;
                }
            }
        }
    }

    // skip non-newline whitespace
    // newline is a separate punctuator
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

    // check if the partial + peek() is a type, keyword, operator, or punctuator
    // if so, current partial can be extended
    fn should_advance(&mut self, partial: &str) -> Result<bool, RatError> {
        let Some(ch) = self.peek()? else {
            return Ok(false);
        };
        let mut extended = String::from(partial);
        extended.push(ch);
        Ok(Category::is_any(&extended).is_some())
    }

    // source.read() wrapper
    fn read(&mut self) -> Result<Option<char>, RatError> {
        Ok(self.source.read()?.map(|b| b as char))
    }

    // source.peek() wrapper
    fn peek(&mut self) -> Result<Option<char>, RatError> {
        Ok(self.source.peek()?.map(|b| b as char))
    }

    // Token::new() wrapper
    fn emit(
        &mut self,
        category: Category,
        partial: String,
        start_pos: Position,
    ) -> Result<Option<Token>, RatError> {
        let end_pos = self.source.position();
        Ok(Some(Token::new(
            category,
            partial,
            Span::set(start_pos, end_pos),
        )))
    }
}
