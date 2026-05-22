use crate::compiler::{
    Category, ErrorKind, LexicalError, Position, RatError, RatSource, Span, Token,
};
use std::collections::VecDeque;

/// TODO: finish read_unicode_escape()
/// TODO: multi-line string literals
/// TODO: single-line & multi-line comments
/// TODO: Hex/Octal/Binary integer representations
///
/// TODO: Improved error recovery, currently too coarse
/// Examples:
/// Invalid character          -> consume one char                    
/// Invalid escape sequence    -> consume minimal chars, perhaps until closing quote or newline
/// Unterminated string        -> skip until closing quote OR newline
/// Unterminated block comment -> skip until EOF or `*/`              
/// Malformed number           -> stop at first invalid char          
/// unknown state              -> broader sync                       

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

    pub fn dispatch(&mut self) -> Result<(), RatError> {
        loop {
            match self.advance_token() {
                Ok(Some(token)) => {
                    self.tokens.push_back(token.clone());
                }

                Err(err) if matches!(err.kind, ErrorKind::Lexical(_)) => {
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
        self.verify_opening_char("advance_string_literal()", '"', actual, start_pos);

        let mut partial = String::from("\"");

        loop {
            self.match_unterminated(LexicalError::UnterminatedString, &partial, start_pos)?;

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

    fn advance_character_literal(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let actual = self.read()?;

        #[cfg(debug_assertions)]
        self.verify_opening_char("advance_character_literal()", '\'', actual, start_pos);
        let mut partial = String::from("\'");

        self.match_unterminated(LexicalError::UnterminatedCharLiteral, &partial, start_pos)?;

        match self.peek()? {
            Some('\'') => {
                self.consume_into(&mut partial)?;
                self.lexical_error(LexicalError::EmptyCharLiteral, &partial, start_pos)?;
            }
            Some('\\') => {
                self.consume_into(&mut partial)?;
                let sequence = self.read_escape_sequence(partial.clone(), start_pos)?;
                partial.push_str(&sequence)
            }
            _ => {
                self.consume_into(&mut partial)?;
            }
        }

        self.match_unterminated(LexicalError::UnterminatedCharLiteral, &partial, start_pos)?;

        match self.peek()? {
            Some('\'') => {
                self.consume_into(&mut partial)?;
                let token = self.emit(Category::Literal, partial, start_pos);
                Ok(Some(token))
            }
            _ => {
                self.consume_into(&mut partial)?;
                self.lexical_error(LexicalError::MultipleCharsInLiteral, partial, start_pos)
            }
        }
    }

    fn read_escape_sequence(
        &mut self,
        partial: String,
        start_pos: Position,
    ) -> Result<String, RatError> {
        match self.read()? {
            Some(ch @ ('\\' | '\'' | '"' | 'n' | 'r' | 't' | 'b' | '0')) => Ok(String::from(ch)),
            Some('u') => self.read_unicode_escape(format!("{}u", partial), start_pos),
            Some(ch) => self.lexical_error(
                LexicalError::InvalidEscapeSequence(ch),
                format!("{}{}", partial, ch),
                start_pos,
            ),
            None => {
                self.lexical_error(LexicalError::UnterminatedEscapeSequence, partial, start_pos)
            }
        }
    }

    fn read_unicode_escape(
        &mut self,
        partial: String,
        start_pos: Position,
    ) -> Result<String, RatError> {
        let mut sequence = String::new();

        match self.read()? {
            Some('{') => sequence.push('{'),
            Some(ch) => {
                return self.lexical_error(
                    LexicalError::InvalidUnicodeEscapeOpener(ch),
                    format!("{}{}", partial, ch),
                    start_pos,
                );
            }
            None => {
                return self.lexical_error(
                    LexicalError::UnterminatedUnicodeEscape,
                    partial,
                    start_pos,
                );
            }
        }

        loop {
            match self.read()? {
                Some('}') => {
                    sequence.push('}');
                    break;
                }

                Some(ch) if ch.is_ascii_hexdigit() => {
                    sequence.push(ch);
                }

                Some(ch) => {
                    return self.lexical_error(
                        LexicalError::InvalidUnicodeEscapeDigit(ch),
                        format!("{}{}{}", partial, sequence, ch),
                        start_pos,
                    );
                }

                None => {
                    return self.lexical_error(
                        LexicalError::UnterminatedUnicodeEscape,
                        format!("{}{}", partial, sequence),
                        start_pos,
                    );
                }
            }
        }

        let hex = &sequence[1..sequence.len() - 1];

        if hex.is_empty() || hex.len() > 6 {
            return self.lexical_error(
                LexicalError::InvalidUnicodeDigitCount(hex.len()),
                format!("{}{}", partial, sequence),
                start_pos,
            );
        }

        let codepoint = u32::from_str_radix(hex, 16).unwrap();

        if codepoint > 0x10_FFFF {
            return self.lexical_error(
                LexicalError::InvalidUnicodeCodepoint(codepoint),
                format!("{}{}", partial, sequence),
                start_pos,
            );
        }

        if (0xD800..=0xDFFF).contains(&codepoint) {
            return self.lexical_error(
                LexicalError::SurrogateCodepoint(codepoint),
                format!("{}{}", partial, sequence),
                start_pos,
            );
        }

        Ok(format!("u{}", sequence))
    }

    fn advance_numeric_literal(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let mut partial = String::new();

        let prefix = self.read_digits(partial.clone(), start_pos)?;
        partial.push_str(&prefix);

        let mut is_floating_type = false;

        if matches!(self.peek()?, Some('.')) {
            is_floating_type = true;
            self.consume_into(&mut partial)?;
            let suffix = self.read_digits(partial.clone(), start_pos)?;
            partial.push_str(&suffix);
        }

        let suffix = self.read_numeric_suffix(is_floating_type, partial.clone(), start_pos)?;
        partial.push_str(&suffix);

        let token = self.emit(Category::Literal, partial, start_pos);

        #[cfg(debug_assertions)]
        self.verify_numeric_literal("advance_numeric_literal()", &token, start_pos);

        Ok(Some(token))
    }

    fn read_digits(&mut self, partial: String, start_pos: Position) -> Result<String, RatError> {
        let mut digits = String::new();
        self.accumulate(&mut digits, |ch| ch.is_ascii_digit())?;

        if digits.is_empty() {
            return self.lexical_error(
                LexicalError::NoDigitsInNumericLiteral,
                format!("{}{}", partial, digits),
                start_pos,
            );
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
                self.consume_into(&mut suffix)?;
            }

            Some('u') if !is_floating_type => {
                self.consume_into(&mut suffix)?;

                if matches!(self.peek()?, Some(ch) if is_integral_type(ch)) {
                    self.consume_into(&mut suffix)?;
                }
            }

            Some(ch) if !is_floating_type && is_integral_type(ch) => {
                self.consume_into(&mut suffix)?;
            }

            Some(ch) if !Category::is_delimiter(ch) => {
                return self.lexical_error(
                    LexicalError::UnexpectedCharAfterNumeric(ch),
                    format!("{}{}{}", partial, suffix, ch),
                    start_pos,
                );
            }

            _ => {}
        };

        Ok(suffix)
    }

    fn advance_keyword_or_identifier(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let mut partial = String::new();

        self.accumulate(&mut partial, |ch| ch.is_alphanumeric() || ch == '_')?;

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
                self.consume_into(&mut partial)?;
            } else {
                break;
            }
        }

        if partial.is_empty() {
            let ch = self.read()?.unwrap();
            return self.lexical_error(
                LexicalError::UnexpectedChar(ch),
                String::from(ch),
                start_pos,
            );
        }

        let category = Category::is_any(&partial).unwrap_or(Category::Invalid);

        Ok(Some(self.emit(category, partial, start_pos)))
    }

    fn verify_opening_char(
        &mut self,
        function_name: &str,
        expected: char,
        actual: Option<char>,
        _start_pos: Position,
    ) {
        match actual {
            Some(ch) if ch == expected => {}
            Some(ch) => panic!(
                "in {:?} expected {:?}, got {:?}",
                function_name, expected, ch
            ),
            None => panic!("in {:?} expected {:?}, got EOF", function_name, expected),
        }
    }

    fn verify_numeric_literal(&self, function_name: &str, token: &Token, _start_pos: Position) {
        use std::sync::OnceLock;

        static RE: OnceLock<regex::Regex> = OnceLock::new();
        let re = RE.get_or_init(|| regex::Regex::new(r"\d+((\.\d+)?[df]?|u?[icls]?)").unwrap());

        assert!(
            re.is_match(&token.value),
            "in {:?} {:?} did not match regex specification",
            function_name,
            token.value,
        );
    }

    fn advance_lexical_error(&mut self, mut err: RatError) -> Result<Token, RatError> {
        let start_pos = err.span.start_pos;
        let mut value = err.value.clone();

        while !matches!(self.peek()?, Some('\n') | None) {
            value.push(self.read()?.unwrap());
        }

        err.value = value.clone();
        self.errors.push(err);

        let span = Span::set(start_pos, self.source.position());
        let token = Token::new(Category::Invalid, value, span);

        Ok(token)
    }

    #[inline]
    fn accumulate(
        &mut self,
        partial: &mut String,
        pred: impl Fn(char) -> bool,
    ) -> Result<usize, RatError> {
        let mut chars_read = 0;

        while let Some(ch) = self.source.peek()? {
            if !pred(ch) {
                break;
            }
            partial.push(self.source.read()?.unwrap());
            chars_read += 1;
        }

        Ok(chars_read)
    }

    #[inline]
    fn match_unterminated(
        &mut self,
        kind: LexicalError,
        partial: &str,
        start_pos: Position,
    ) -> Result<(), RatError> {
        match self.peek()? {
            Some('\n') | None => self.lexical_error(kind, partial, start_pos),
            _ => Ok(()),
        }
    }

    #[inline]
    fn consume_into(&mut self, partial: &mut String) -> Result<char, RatError> {
        // caller must guarantee peek succeeds
        let ch = self.read()?.unwrap();
        partial.push(ch);
        Ok(ch)
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
    fn emit(&mut self, category: Category, value: String, start_pos: Position) -> Token {
        Token::new(
            category,
            value,
            Span::set(start_pos, self.source.position()),
        )
    }

    #[inline]
    fn lexical_error<T>(
        &mut self,
        err: LexicalError,
        value: impl Into<String>,
        start_pos: Position,
    ) -> Result<T, RatError> {
        Err(RatError::lexical(
            err,
            value.into(),
            Span::set(start_pos, self.source.position()),
        ))
    }
}
