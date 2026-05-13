use crate::compiler::RatError;
use crate::compiler::RatSource;
use crate::compiler::{Category, Position, Span, Token};

pub struct Lexer {
    source: RatSource,
}

impl Lexer {
    pub fn init(source: RatSource) -> Self {
        Lexer { source: source }
    }

    pub fn advance_token(&mut self) -> Result<Option<Token>, RatError> {
        match self.source.peek()? {
            Some(b) => {
                let ch = b as char;
                match ch {
                    '\"' => return self.advance_string_literal(),
                    '\'' => return self.advance_character_literal(),
                    _ if ch.is_numeric() => return self.advance_numeric_literal(),
                    _ => {}
                }
            }
            None => return Ok(None),
        };

        match self.advance_whitespace()? {
            Some(_) => {}
            None => {
                return Ok(None);
            }
        };

        let start_pos = self.source.position();
        let mut partial = String::new();

        loop {
            let ch = match self.source.read()? {
                Some(b) => b as char,
                None => {
                    return Ok(None);
                }
            };

            partial.push(ch);

            if self.should_advance(&partial)? {
                continue;
            }

            match Category::is_any(partial.as_str()) {
                Some(category) => {
                    let end_pos = self.source.position();
                    let token = Token::new(category, partial, Span::set(start_pos, end_pos));
                    return Ok(Some(token));
                }
                None => {}
            };

            match self.source.peek()? {
                Some(b) => {
                    let ch = b as char;
                    if Category::is_delimiter(ch) {
                        let end_pos = self.source.position();
                        let token = Token::new(
                            Category::Identifier,
                            partial,
                            Span::set(start_pos, end_pos),
                        );
                        return Ok(Some(token));
                    }
                }
                None => {
                    let end_pos = self.source.position();
                    return Ok(Some(Token::new(
                        Category::Identifier,
                        partial,
                        Span::set(start_pos, end_pos),
                    )));
                }
            }
        }
    }

    fn advance_string_literal(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let mut is_esc_sequence = false;

        let mut partial = String::new();
        match self.source.read()? {
            Some(b) => {
                let ch = b as char;
                partial.push(ch);
            }
            None => {
                return Err(RatError::InternalError(String::from(
                    "Expected \'\"\', Got EOF",
                )))
            }
        };

        loop {
            let ch = match self.source.read()? {
                Some(b) => b as char,
                None => {
                    return Err(RatError::LexicalError(String::from(
                        "missing \'\"\' at the end of string literal",
                    )))
                }
            };

            if is_esc_sequence {
                match ch {
                    '\\' => {}
                    '\'' => {}
                    '\"' => {}
                    'n' => {}
                    'r' => {}
                    't' => {}
                    'b' => {}
                    '0' => {}
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
                let end_pos = self.source.position();
                let token = Token::new(Category::Literal, partial, Span::set(start_pos, end_pos));
                return Ok(Some(token));
            }
        }
    }

    fn advance_unicode_escape(&mut self) -> Result<String, RatError> {
        match self.source.read()? {
            Some(b) if (b as char) == '{' => {}
            _ => {
                return Err(RatError::LexicalError(String::from(
                    "expected '{' after '\\u' in unicode escape",
                )));
            }
        }

        let mut hex = String::new();
        loop {
            match self.source.read()? {
                Some(b) => {
                    let hch = b as char;
                    if hch == '}' {
                        break;
                    }
                    if !hch.is_ascii_hexdigit() {
                        return Err(RatError::LexicalError(format!(
                            "invalid hexadecimal digit in unicode escape: '{}'",
                            hch
                        )));
                    }
                    hex.push(hch);
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

    fn advance_numeric_literal(&mut self) -> Result<Option<Token>, RatError> {
        Ok(None)
    }

    fn advance_whitespace(&mut self) -> Result<Option<()>, RatError> {
        loop {
            let Some(b) = self.source.peek()? else {
                return Ok(None);
            };

            let ch = b as char;
            if !ch.is_whitespace() || ch == '\n' {
                return Ok(Some(()));
            }

            self.source.read()?;
        }
    }

    fn should_advance(&mut self, partial: &str) -> Result<bool, RatError> {
        match self.source.peek()? {
            Some(b) => {
                let ch = b as char;
                let mut extended = String::from(partial);
                extended.push(ch);
                Ok(Category::is_any(extended.as_str()).is_some())
            }
            None => Ok(false),
        }
    }
}
