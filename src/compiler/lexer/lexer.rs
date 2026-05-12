use crate::compiler::RatError;
use crate::compiler::RatSource;
use crate::compiler::{Category, Position, Span, Token};

pub struct Lexer {
    source: RatSource,
}

enum AdvanceWhitespace {
    Token(Token),
    Continue,
    Eof,
}

impl Lexer {
    pub fn init(source: RatSource) -> Self {
        Lexer { source: source }
    }

    pub fn advance_token(&mut self) -> Result<Option<Token>, RatError> {
        match self.source.peek()? {
            Some(_) => {}
            None => {
                return Ok(None);
            }
        }

        match self.advance_whitespace()? {
            Some(_) => {}
            None => {
                return Ok(None);
            }
        };

        let start_pos = self.source.position();
        let mut partial = String::new();

        loop {
            // first read a character
            let ch = match self.source.read()? {
                Some(b) => b as char,
                None => {
                    return Ok(None);
                }
            };

            match ch {
                '\'' => return self.advance_character_literal(start_pos),
                '\"' => return self.advance_string_literal(start_pos),
                _ if ch.is_numeric() => return self.advance_numeric_literal(start_pos),
                _ => {}
            }

            partial.push(ch);

            // maximal munch: continue if appending the next character to partial
            // matches a longer reserved word, continue
            match self.source.peek()? {
                Some(b) => {
                    let ch = b as char;
                    let mut extended = partial.clone();
                    extended.push(ch);
                    match Category::is_any(extended.as_str()) {
                        Some(_) => continue,
                        None => {}
                    };
                }
                None => {}
            };

            // if the current partial mathches a reserved word, return it as a token
            match Category::is_any(partial.as_str()) {
                Some(category) => {
                    let end_pos = self.source.position();
                    let token = Token::new(category, partial, Span::set(start_pos, end_pos));
                    return Ok(Some(token));
                }
                None => {}
            };

            // check if next character is a delimter
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
                None => return Ok(None),
            }
        }
    }

    fn advance_string_literal(&mut self, start_pos: Position) -> Result<Option<Token>, RatError> {
        Ok(None)
    }
    fn advance_character_literal(
        &mut self,
        start_pos: Position,
    ) -> Result<Option<Token>, RatError> {
        Ok(None)
    }
    fn advance_numeric_literal(&mut self, start_pos: Position) -> Result<Option<Token>, RatError> {
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
}
