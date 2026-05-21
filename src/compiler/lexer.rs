use crate::compiler::{Category, ErrorKind, Position, RatError, RatSource, Span, Token};
use regex::Regex;
use std::collections::VecDeque;

pub struct Lexer {
    source: RatSource,
    tokens: VecDeque<Token>,
    errors: Vec<RatError>,
}

impl Lexer {
    pub fn init(source: RatSource) -> Self {
        Lexer {
            source,
            tokens: VecDeque::new(),
            errors: Vec::new(),
        }
    }

    pub fn get_tokens(&self) -> &VecDeque<Token> {
        &self.tokens
    }

    pub fn get_errors(&self) -> &Vec<RatError> {
        &self.errors
    }

    pub fn advance_tokens(&mut self) -> Result<(), RatError> {
        loop {
            match self.advance_token() {
                Ok(Some(token)) => {
                    self.tokens.push_back(token.clone());
                }

                Err(err) if err.kind == ErrorKind::Lexical => {
                    let token = self.advance_lexical_error(err)?;
                    self.tokens.push_back(token);
                }

                // treat Error::Kind{Io, Source} as fatal
                Err(err) => {
                    self.tokens.clear();
                    self.errors.clear();
                    return Err(err);
                }
                Ok(None) => return Ok(()),
            }
        }
    }

    pub fn advance_token(&mut self) -> Result<Option<Token>, RatError> {
        match self.advance_whitespace()? {
            Some(_) => {}
            None => return Ok(None),
        }

        match self.peek()? {
            None => Ok(None),
            Some('"') => self.advance_string_literal(),
            Some('\'') => self.advance_character_literal(),
            Some(ch) if ch.is_ascii_digit() => self.advance_numeric_literal(),
            Some(ch) if ch.is_alphabetic() || ch == '_' => self.advance_keyword_or_identifier(),
            _ => self.advance_operator_or_punctuator(),
        }
    }

    fn advance_whitespace(&mut self) -> Result<Option<()>, RatError> {
        loop {
            match self.peek()? {
                None => return Ok(None),
                // treat newline ('\n') as significant character
                Some(ch) if !ch.is_whitespace() || ch == '\n' => return Ok(Some(())),
                _ => {
                    self.source.read()?;
                }
            }
        }
    }

    fn advance_string_literal(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();

        let actual = self.read()?;
        #[cfg(debug_assertions)]
        self.verify_opening_char('"', actual, start_pos)?;

        let mut partial = "\"".to_string();

        loop {
            match self.peek()? {
                Some('\n') | None => {
                    return Err(self.emit_error("unterminated string literal", partial, start_pos));
                }

                Some(_) => {
                    let ch = self.read()?.unwrap();
                    partial.push(ch);
                    match ch {
                        '"' => {
                            let token = self.emit(Category::Literal, partial, start_pos);
                            return Ok(Some(token));
                        }
                        '\\' => {
                            let sequence = self.read_escape_sequence(partial.clone(), start_pos)?;
                            partial.push_str(&sequence)
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn advance_character_literal(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();

        let actual = self.read()?;
        #[cfg(debug_assertions)]
        self.verify_opening_char('\'', actual, start_pos)?;

        let mut partial = "\'".to_string();

        match self.peek()? {
            Some('\n') | None => {
                return Err(self.emit_error("unterminated character literal", partial, start_pos));
            }

            Some('\'') => {
                partial.push(self.read()?.unwrap());
                return Err(self.emit_error("empty character literal", partial, start_pos));
            }

            Some('\\') => {
                let sequence = self.read_escape_sequence(partial.clone(), start_pos)?;
                partial.push_str(&sequence)
            }

            Some(_) => {
                partial.push(self.read()?.unwrap());
            }
        }

        match self.peek()? {
            Some('\n') | None => {
                return Err(self.emit_error("unterminated character literal", partial, start_pos));
            }

            Some('\'') => {
                partial.push(self.read()?.unwrap());
                let token = self.emit(Category::Literal, partial, start_pos);
                Ok(Some(token))
            }

            Some(_) => {
                partial.push(self.read()?.unwrap());
                return Err(self.emit_error(
                    "character literal should only contain one character",
                    partial,
                    start_pos,
                ));
            }
        }
    }

    fn read_escape_sequence(
        &mut self,
        partial: String,
        start_pos: Position,
    ) -> Result<String, RatError> {
        let actual = self.read()?;
        #[cfg(debug_assertions)]
        self.verify_opening_char('\\', actual, start_pos)?;
        match self.read()? {
            Some(ch @ ('\\' | '\'' | '"' | 'n' | 'r' | 't' | 'b' | '0')) => Ok(format!("\\{}", ch)),
            Some('u') => Ok(format!(
                "\\u{}",
                self.read_unicode_escape(partial, start_pos)?
            )),
            Some(ch) => Err(self.emit_error(
                format!("invalid escape sequence '\\{}'", ch),
                format!("{}\\{}", partial, ch),
                start_pos,
            )),
            None => Err(self.emit_error(
                "unterminated escape sequence at EOF",
                format!("{}\\", partial),
                start_pos,
            )),
        }
    }

    fn read_unicode_escape(
        &mut self,
        partial: String,
        start_pos: Position,
    ) -> Result<String, RatError> {
        let span = Span::new(start_pos, self.source.position());
        Err(RatError::internal(
            "advance_unicode_escape() unimplemented",
            partial,
            span,
        ))
    }

    fn advance_numeric_literal(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let mut partial = String::new();
        let prefix = self.read_digits(partial.clone(), start_pos)?;
        partial.push_str(&prefix);

        let mut is_floating_type = false;

        if matches!(self.peek()?, Some('.')) {
            is_floating_type = true;
            partial.push(self.read()?.unwrap());
            let suffix = self.read_digits(partial.clone(), start_pos)?;
            partial.push_str(&suffix);
        }

        let suffix = self.read_numeric_suffix(is_floating_type, partial.clone(), start_pos)?;
        partial.push_str(&suffix);

        let token = self.emit(Category::Literal, partial, start_pos);

        #[cfg(debug_assertions)]
        self.verify_numeric_literal(&token)?;

        Ok(Some(token))
    }

    fn read_digits(&mut self, partial: String, start_pos: Position) -> Result<String, RatError> {
        let mut digits = String::new();

        while let Some(ch) = self.peek()? {
            if !ch.is_ascii_digit() {
                break;
            }
            digits.push(self.read()?.unwrap());
        }

        #[cfg(debug_assertions)]
        if digits.is_empty() {
            return Err(self.emit_error(
                "advance_digits() returned no ascii digit",
                format!("{:?}{:?}", partial, digits),
                start_pos,
            ));
        }

        Ok(digits)
    }

    fn read_numeric_suffix(
        &mut self,
        is_floating_type: bool,
        partial: String,
        start_pos: Position,
    ) -> Result<String, RatError> {
        let is_integral_type = |ch: char| matches!(ch, 'i' | 'c' | 'l' | 's');
        let mut suffix = String::new();

        match self.peek()? {
            Some('d' | 'f') => {
                suffix.push(self.read()?.unwrap());
            }
            Some('u') if !is_floating_type => {
                suffix.push(self.read()?.unwrap());
                if matches!(self.peek()?, Some(ch) if is_integral_type(ch)) {
                    suffix.push(self.read()?.unwrap());
                }
            }
            Some(ch) if !is_floating_type && is_integral_type(ch) => {
                suffix.push(self.read()?.unwrap());
            }

            Some(ch) if !Category::is_delimiter(ch) => {
                return Err(self.emit_error(
                    format!("unexpected character {:?} after numeric literal", ch),
                    format!("{:?}{:?}", partial, suffix),
                    start_pos,
                ))
            }
            _ => {}
        };

        Ok(suffix)
    }

    fn advance_keyword_or_identifier(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let mut partial = String::new();

        while let Some(ch) = self.peek()? {
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            partial.push(self.read()?.unwrap());
        }

        let category = Category::is_any(&partial).unwrap_or(Category::Identifier);
        Ok(Some(self.emit(category, partial, start_pos)))
    }

    fn advance_operator_or_punctuator(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let mut partial = String::new();

        loop {
            let Some(ch) = self.peek()? else { break };
            if ch.is_alphanumeric() || ch == '_' || (ch.is_whitespace() && ch != '\n') {
                break;
            }

            let mut extended = partial.clone();
            extended.push(ch);

            if Category::is_any(&extended).is_some() {
                partial.push(self.read()?.unwrap());
            } else {
                break;
            }
        }

        if partial.is_empty() {
            let ch = self.read()?.unwrap();
            return Err(self.emit_error(
                format!("unexpected character {:?}", ch),
                ch.to_string(),
                start_pos,
            ));
        }

        let category = Category::is_any(&partial).unwrap_or(Category::Invalid);
        Ok(Some(self.emit(category, partial, start_pos)))
    }

    fn verify_opening_char(
        &mut self,
        expected: char,
        actual: Option<char>,
        start: Position,
    ) -> Result<(), RatError> {
        match actual {
            Some(ch) if ch == expected => Ok(()),

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

    fn verify_numeric_literal(&mut self, token: &Token) -> Result<(), RatError> {
        let re = Regex::new(r"\d+((\.\d+)?[df]?|u?[icls]?)").unwrap();
        if re.is_match(&token.value) {
            return Ok(());
        }

        return Err(RatError::internal(
            "advance_numeric_literal() did not match numeric literal regex specification",
            token.value.clone(),
            token.span,
        ));
    }

    fn advance_lexical_error(&mut self, err: RatError) -> Result<Token, RatError> {
        let start_pos = err.span.start;
        let mut value = err.value.clone();
        self.errors.push(err);

        while !matches!(self.peek()?, Some('\n') | None) {
            value.push(self.read()?.unwrap());
        }

        let span = Span::new(start_pos, self.source.position());
        let token = Token::new(Category::Invalid, value, span);
        Ok(token)
    }

    #[inline]
    fn read(&mut self) -> Result<Option<char>, RatError> {
        self.source.read()
    }

    #[inline]
    fn peek(&mut self) -> Result<Option<char>, RatError> {
        self.source.peek()
    }

    #[inline]
    fn emit(&mut self, category: Category, value: String, start: Position) -> Token {
        Token::new(category, value, Span::new(start, self.source.position()))
    }

    fn emit_error(
        &mut self,
        message: impl Into<String>,
        value: impl Into<String>,
        start: Position,
    ) -> RatError {
        RatError::lexical(
            message,
            value.into(),
            Span::new(start, self.source.position()),
        )
    }
}
