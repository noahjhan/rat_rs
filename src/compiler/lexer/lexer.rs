use crate::compiler::RatError;
use crate::compiler::RatSource;
use crate::compiler::{Category, Span, Token};

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
            let ch = match self.source.read()? {
                Some(b) => b as char,
                None => {
                    return Ok(None);
                }
            };

            partial.push(ch);

            // maximal munch
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
                    // check if next character is a delimter
                    //   e.g., ',' for identifiers , or '+' or ' ' for operators
                    // if so, break
                }
                None => break,
            }
        }

        Ok(None)
    }

    // fn advance_newline(&mut self) -> Result<Option<Token>, RatError> {
    //     let start_pos = self.source.position();
    //
    //     match self.source.read()? {
    //         Some(b'\n') => {}
    //
    //         Some(ch) => {
    //             return Err(RatError::InternalError(format!(
    //                 "Expected newline character in call to advance_newline, received {:?}",
    //                 ch
    //             )));
    //         }
    //
    //         None => {
    //             return Err(RatError::InternalError(String::from(
    //                 "Expected newline character in call to advance_newline, received EOF",
    //             )));
    //         }
    //     }
    //
    //     let end_pos = self.source.position();
    //
    //     Ok(Some(Token::new(
    //         Category::Punctuator,
    //         String::from("\n"),
    //         Span::set(start_pos, end_pos),
    //     )))
    // }

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

        // loop {
        //     let Some(b) = self.source.peek()? else {
        //         return Ok(AdvanceWhitespace::Eof);
        //     };
        //
        //     let ch = b as char;
        //
        //     if !ch.is_whitespace() {
        //         return Ok(AdvanceWhitespace::Continue);
        //     }
        //
        //     let start_pos = self.source.position();
        //     self.source.read()?;
        //
        //     if ch == '\n' {
        //         let end_pos = self.source.position();
        //         let span = Span::set(start_pos, end_pos);
        //         let token = Token::new(Category::Punctuator, String::from(";"), span);
        //
        //         return Ok(AdvanceWhitespace::Token(token));
        //     }
        // }
    }
}
