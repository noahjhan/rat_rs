use crate::compiler::RatError;
use crate::compiler::RatSource;
use crate::compiler::{Category, Position, Span, Token};

pub struct Lexer {
    source: RatSource,
}

impl Lexer {
    pub fn init(source: RatSource) -> Self {
        Lexer { source }
    }

    pub fn next(&mut self) -> Result<Option<Token>, RatError> {
        match self.advance_token() {
            Ok(token) => Ok(token),
            Err(err @ RatError::LexicalError(_)) => {
                let _ = self.read_until_delimiter();
                Err(err)
            }
            Err(err) => Err(err),
        }
    }

    fn advance_token(&mut self) -> Result<Option<Token>, RatError> {
        match self.peek()? {
            Some(ch) => match ch {
                '\"' => return self.advance_string_literal(),
                '\'' => return self.advance_character_literal(),
                _ if ch.is_numeric() => return self.advance_numeric_literal(),
                _ => {}
            },
            None => return Ok(None),
        };

        match self.advance_whitespace()? {
            Some(_) => {}
            None => return Ok(None),
        };

        let start_pos = self.source.position();
        let mut partial = String::new();

        loop {
            let ch = match self.read()? {
                Some(ch) => ch,
                None => return Ok(None),
            };

            partial.push(ch);

            if self.should_advance(&partial)? {
                continue;
            }

            match Category::is_any(partial.as_str()) {
                Some(category) => return self.emit(category, partial, start_pos),
                None => {}
            };

            match self.peek()? {
                Some(ch) if Category::is_delimiter(ch) => {
                    return self.emit(Category::Identifier, partial, start_pos);
                }
                Some(_) => {}
                None => return self.emit(Category::Identifier, partial, start_pos),
            }
        }
    }

    fn advance_string_literal(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let mut is_esc_sequence = false;

        let mut partial = String::new();

        // overly defensive?
        match self.read()? {
            Some('"') => partial.push('"'),
            Some(ch) => {
                return Err(RatError::InternalError(format!(
                    "expected '\"', got {:?}",
                    ch
                )))
            }
            None => {
                return Err(RatError::InternalError(String::from(
                    "expected '\"', got EOF",
                )))
            }
        };

        loop {
            let ch = match self.read()? {
                Some(ch) => ch,
                None => {
                    return Err(RatError::LexicalError(String::from(
                        "missing '\"' at the end of string literal",
                    )))
                }
            };

            if is_esc_sequence {
                match ch {
                    '\\' | '\'' | '\"' | 'n' | 'r' | 't' | 'b' | '0' => {}
                    'u' => {
                        let unicode = self.advance_unicode_escape()?;
                        partial.push_str(&unicode);
                        is_esc_sequence = false;
                        continue;
                    }
                    _ => {
                        return Err(RatError::LexicalError(format!(
                            "invalid escape sequence: '\\{}'",
                            ch
                        )));
                    }
                }
                is_esc_sequence = false;
                partial.push(ch);
                continue;
            }

            partial.push(ch);

            if ch == '\\' {
                is_esc_sequence = true;
            } else if ch == '\"' {
                return self.emit(Category::Literal, partial, start_pos);
            }
        }
    }

    fn advance_unicode_escape(&mut self) -> Result<String, RatError> {
        match self.read()? {
            Some('{') => {}
            _ => {
                return Err(RatError::LexicalError(String::from(
                    "expected '{' after '\\u' in unicode escape",
                )));
            }
        }

        let mut hex = String::new();
        loop {
            match self.read()? {
                Some('}') => break,
                Some(hch) if hch.is_ascii_hexdigit() => hex.push(hch),
                Some(hch) => {
                    return Err(RatError::LexicalError(format!(
                        "invalid hexadecimal digit in unicode escape: '{}'",
                        hch
                    )));
                }
                None => {
                    return Err(RatError::LexicalError(String::from(
                        "unterminated unicode escape sequence",
                    )));
                }
            }
        }

        if hex.is_empty() || hex.len() > 6 {
            return Err(RatError::LexicalError(format!(
                "unicode escape must be between 1 and 6 hex digits, got {}",
                hex.len()
            )));
        }

        let codepoint = u32::from_str_radix(&hex, 16).unwrap();
        if codepoint > 0x10FFFF {
            return Err(RatError::LexicalError(format!(
                "unicode codepoint out of range: U+{:X}",
                codepoint
            )));
        }

        if (0xD800..=0xDFFF).contains(&codepoint) {
            return Err(RatError::LexicalError(format!(
                "unicode codepoint is a surrogate: U+{:X}",
                codepoint
            )));
        }

        Ok(format!("\\u{{{}}}", hex))
    }

    fn advance_character_literal(&mut self) -> Result<Option<Token>, RatError> {
        Ok(None)
    }

    fn read_numerics(&mut self, buf: &mut String) -> Result<bool, RatError> {
        let mut read_any = false;
        loop {
            match self.peek()? {
                Some(ch) if ch.is_ascii_digit() => {
                    self.read()?;
                    buf.push(ch);
                    read_any = true;
                }
                _ => return Ok(read_any),
            }
        }
    }

    // regex: (\d+)(((\.)?(\d*)?(d|f)?)|(u)?[icls]?)?)
    fn advance_numeric_literal(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let mut partial = String::new();

        if !self.read_numerics(&mut partial)? {
            return Err(RatError::InternalError(String::from(
                "advance_numeric_literal called but no leading digits found",
            )));
        }

        if matches!(self.peek()?, Some('.')) {
            self.read()?;
            partial.push('.');

            self.read_numerics(&mut partial)?;

            match self.peek()? {
                Some('d') | Some('f') => {
                    let suffix = self.read()?.unwrap();
                    partial.push(suffix);
                    return self.expect_delimiter_then_emit(Category::Literal, partial, start_pos);
                }
                Some(ch) if !Category::is_delimiter(ch) => {
                    return Err(RatError::LexicalError(format!(
                        "unexpected character {:?} after fractional numeric literal, expected 'd', 'f', or a delimiter",
                        ch
                    )));
                }
                _ => {
                    return self.emit(Category::Literal, partial, start_pos);
                }
            }
        }

        match self.peek()? {
            Some(ch @ 'd') | Some(ch @ 'f') => {
                self.read()?;
                partial.push(ch);
                return self.expect_delimiter_then_emit(Category::Literal, partial, start_pos);
            }
            Some('u') => {
                self.read()?;
                partial.push('u');
                match self.peek()? {
                    Some(ch @ 'i') | Some(ch @ 'c') | Some(ch @ 'l') | Some(ch @ 's') => {
                        self.read()?;
                        partial.push(ch);
                    }
                    _ => {}
                }
                return self.expect_delimiter_then_emit(Category::Literal, partial, start_pos);
            }
            Some(ch @ 'i') | Some(ch @ 'c') | Some(ch @ 'l') | Some(ch @ 's') => {
                self.read()?;
                partial.push(ch);
                return self.expect_delimiter_then_emit(Category::Literal, partial, start_pos);
            }
            Some(ch) if Category::is_delimiter(ch) => {
                return self.emit(Category::Literal, partial, start_pos);
            }
            None => {
                return self.emit(Category::Literal, partial, start_pos);
            }
            Some(ch) => {
                return Err(RatError::LexicalError(format!(
                    "unexpected character {:?} in numeric literal, expected a suffix ('u', 'i', 'c', 'l', 's', 'f', 'd'), '.', or a delimiter",
                    ch
                )));
            }
        }
    }

    fn expect_delimiter_then_emit(
        &mut self,
        category: Category,
        partial: String,
        start_pos: Position,
    ) -> Result<Option<Token>, RatError> {
        match self.peek()? {
            Some(ch) if !Category::is_delimiter(ch) => Err(RatError::LexicalError(format!(
                "unexpected character {:?} after numeric literal suffix, expected a delimiter",
                ch
            ))),
            _ => self.emit(category, partial, start_pos),
        }
    }

    fn read_until_delimiter(&mut self) -> Result<(), RatError> {
        loop {
            match self.peek()? {
                Some(ch) if Category::is_delimiter(ch) => return Ok(()),
                None => return Ok(()),
                _ => self.read()?,
            };
        }
    }

    fn advance_whitespace(&mut self) -> Result<Option<()>, RatError> {
        loop {
            let Some(ch) = self.peek()? else {
                return Ok(None);
            };
            if !ch.is_whitespace() || ch == '\n' {
                return Ok(Some(()));
            }
            self.source.read()?;
        }
    }

    fn should_advance(&mut self, partial: &str) -> Result<bool, RatError> {
        match self.peek()? {
            Some(ch) => {
                let mut extended = String::from(partial);
                extended.push(ch);
                Ok(Category::is_any(extended.as_str()).is_some())
            }
            None => Ok(false),
        }
    }

    fn read(&mut self) -> Result<Option<char>, RatError> {
        Ok(self.source.read()?.map(|b| b as char))
    }

    fn peek(&mut self) -> Result<Option<char>, RatError> {
        Ok(self.source.peek()?.map(|b| b as char))
    }

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
