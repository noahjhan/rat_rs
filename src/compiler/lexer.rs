use crate::compiler::{
    Category, ErrorKind, LexicalError, Position, RatError, RatSource, Recovery, Span, Token,
};

use std::collections::VecDeque;

/// this file holds the structure associated with lexing source files

/// TODO: multi-line string literals
/// TODO: Hex/Octal/Binary integer representations

pub struct Lexer<'a> {
    source: &'a mut RatSource,
    tokens: VecDeque<Token>,
    errors: Vec<RatError>,
}

impl<'a> Lexer<'a> {
    pub fn init(source: &'a mut RatSource) -> Self {
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
                // ignore repeat newlines
                Ok(Some(token))
                    if token.value == "\n"
                        && matches!(self.tokens.back(), Some(back) if back.value == "\n") =>
                {
                    continue;
                }

                Ok(Some(token)) => {
                    self.tokens.push_back(token.clone());
                }

                Err(mut err) if matches!(err.kind, ErrorKind::Lexical(_)) => {
                    let token = self.advance_lexical_error(&mut err)?;
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

    /// return next available token or error upon failure
    pub fn advance_token(&mut self) -> Result<Option<Token>, RatError> {
        self.advance_whitespace();
        self.advance_single_line_comment();
        self.advance_multi_line_comment()?;

        match self.peek() {
            None => Ok(None),

            Some('"') => self.advance_string_literal(),

            Some('\'') => self.advance_character_literal(),

            Some(ch) if ch.is_ascii_digit() => self.advance_numeric_literal(),

            Some(ch) if ch.is_alphabetic() || ch == '_' => self.advance_keyword_or_identifier(),

            _ => self.advance_operator_or_punctuator(),
        }
    }

    /// skip insignificant whitespace
    fn advance_whitespace(&mut self) {
        loop {
            match self.peek() {
                None => return,

                // treat newline ('\n') as significant character
                Some(ch) if !ch.is_whitespace() || ch == '\n' => return,

                _ => {
                    self.read();
                }
            }
        }
    }

    /// skip all characters past "//", up to the next newline
    fn advance_single_line_comment(&mut self) {
        match self.peek_n(2) {
            Some(str) if str == "//" => {}

            Some(_) => return,

            None => return,
        };

        loop {
            match self.peek() {
                None => return,

                Some('\n') => return,

                Some(_) => {
                    self.read();
                }
            }
        }
    }

    /// skip all characters between "/*" and "*/", including nested sequences
    fn advance_multi_line_comment(&mut self) -> Result<(), RatError> {
        match self.peek_n(2) {
            Some(str) if str == "/*" => {}

            _ => return Ok(()),
        }

        let start_pos = self.source.position();

        let mut partial = String::new();
        let mut stack: Vec<()> = Vec::new();

        partial.push(self.read().unwrap());
        partial.push(self.read().unwrap());
        stack.push(());

        while !stack.is_empty() {
            match self.peek_n(2) {
                Some(str) if str == "/*" => {
                    partial.push(self.read().unwrap());
                    partial.push(self.read().unwrap());
                    stack.push(());
                    continue;
                }

                Some(str) if str == "*/" => {
                    partial.push(self.read().unwrap());
                    partial.push(self.read().unwrap());
                    stack.pop();
                    continue;
                }

                _ => {}
            }

            match self.read() {
                Some(ch) => partial.push(ch),

                None => {
                    let end_pos = self.source.position();

                    return self.lexical_error(
                        LexicalError::UnterminatedMultiLineComment,
                        partial,
                        start_pos,
                        end_pos,
                    );
                }
            }
        }

        Ok(())
    }

    /// lex a string literal and return a token or error upon failure
    fn advance_string_literal(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let actual = self.read();

        #[cfg(debug_assertions)]
        self.verify_opening_char("advance_string_literal()", '"', actual, start_pos);

        let mut partial = String::from("\"");
        loop {
            self.match_unterminated(LexicalError::UnterminatedString, &partial, start_pos)?;
            let ch = self.read().unwrap();
            partial.push(ch);

            match ch {
                '"' => {
                    let end_pos = self.source.position();
                    let token = self.emit(Category::Literal, partial, start_pos, end_pos);
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
    /// lex a character literal and return a token or error upon failure
    fn advance_character_literal(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let actual = self.read();

        #[cfg(debug_assertions)]
        self.verify_opening_char("advance_character_literal()", '\'', actual, start_pos);

        let mut partial = String::from("\'");
        self.match_unterminated(LexicalError::UnterminatedCharLiteral, &partial, start_pos)?;

        match self.peek() {
            Some('\'') => {
                self.consume_into(&mut partial);
                let end_pos = self.source.position();
                self.lexical_error(LexicalError::EmptyCharLiteral, &partial, start_pos, end_pos)?;
            }

            Some('\\') => {
                self.consume_into(&mut partial);
                let sequence = self.read_escape_sequence(partial.clone(), start_pos)?;
                partial.push_str(&sequence)
            }

            _ => {
                self.consume_into(&mut partial);
            }
        }

        self.match_unterminated(LexicalError::UnterminatedCharLiteral, &partial, start_pos)?;

        match self.peek() {
            Some('\'') => {
                self.consume_into(&mut partial);
                let end_pos = self.source.position();
                let token = self.emit(Category::Literal, partial, start_pos, end_pos);
                Ok(Some(token))
            }

            _ => {
                let end_pos = self.source.position();
                self.consume_into(&mut partial);
                self.lexical_error(
                    LexicalError::MultipleCharsInLiteral,
                    partial,
                    start_pos,
                    end_pos,
                )
            }
        }
    }
    /// within string and character literals, advance position and return the valid escape sequence
    /// or associated error
    fn read_escape_sequence(
        &mut self,
        partial: String,
        start_pos: Position,
    ) -> Result<String, RatError> {
        match self.read() {
            Some(ch @ ('\\' | '\'' | '"' | 'n' | 'r' | 't' | 'b' | '0')) => Ok(String::from(ch)),

            Some('u') => self.read_unicode_escape(format!("{}u", partial), start_pos),

            Some(ch) => {
                let end_pos = self.source.position();
                self.lexical_error(
                    LexicalError::InvalidEscapeSequence(ch),
                    format!("{}{}", partial, ch),
                    start_pos,
                    end_pos,
                )
            }

            None => {
                let end_pos = self.source.position();
                self.lexical_error(
                    LexicalError::UnterminatedEscapeSequence,
                    partial,
                    start_pos,
                    end_pos,
                )
            }
        }
    }

    /// within an escape sequence, advance position and return the valid unicode sequence or
    /// associated error
    fn read_unicode_escape(
        &mut self,
        partial: String,
        start_pos: Position,
    ) -> Result<String, RatError> {
        let mut sequence = String::new();
        match self.read() {
            Some('{') => sequence.push('{'),

            Some(ch) => {
                let end_pos = self.source.position();
                return self.lexical_error(
                    LexicalError::InvalidUnicodeEscapeOpener(ch),
                    format!("{}{}", partial, ch),
                    start_pos,
                    end_pos,
                );
            }

            None => {
                let end_pos = self.source.position();
                return self.lexical_error(
                    LexicalError::UnterminatedUnicodeEscape,
                    partial,
                    start_pos,
                    end_pos,
                );
            }
        }

        loop {
            let end_pos = self.source.position();

            match self.read() {
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
                        end_pos,
                    );
                }

                None => {
                    return self.lexical_error(
                        LexicalError::UnterminatedUnicodeEscape,
                        format!("{}{}", partial, sequence),
                        start_pos,
                        end_pos,
                    );
                }
            }
        }

        let hex = &sequence[1..sequence.len() - 1];
        if hex.is_empty() || hex.len() > 6 {
            let end_pos = self.source.position();
            return self.lexical_error(
                LexicalError::InvalidUnicodeDigitCount(hex.len()),
                format!("{}{}", partial, sequence),
                start_pos,
                end_pos,
            );
        }

        let codepoint = u32::from_str_radix(hex, 16).unwrap();
        if codepoint > 0x10_FFFF {
            let end_pos = self.source.position();
            return self.lexical_error(
                LexicalError::InvalidUnicodeCodepoint(codepoint),
                format!("{}{}", partial, sequence),
                start_pos,
                end_pos,
            );
        }

        if (0xD800..=0xDFFF).contains(&codepoint) {
            let end_pos = self.source.position();
            return self.lexical_error(
                LexicalError::SurrogateCodepoint(codepoint),
                format!("{}{}", partial, sequence),
                start_pos,
                end_pos,
            );
        }

        Ok(format!("u{}", sequence))
    }

    /// lex a numeric literal and return a token or error upon failure
    fn advance_numeric_literal(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let mut partial = String::new();

        let prefix = self.read_digits(partial.clone(), start_pos)?;
        partial.push_str(&prefix);

        let mut is_floating_type = false;

        if matches!(self.peek(), Some('.')) {
            is_floating_type = true;
            self.consume_into(&mut partial);
            let suffix = self.read_digits(partial.clone(), start_pos)?;
            partial.push_str(&suffix);
        }

        let suffix = self.read_numeric_suffix(is_floating_type, partial.clone(), start_pos)?;
        partial.push_str(&suffix);

        let end_pos = self.source.position();
        let token = self.emit(Category::Literal, partial, start_pos, end_pos);

        #[cfg(debug_assertions)]
        self.verify_numeric_literal("advance_numeric_literal()", &token, start_pos);

        Ok(Some(token))
    }
    /// advance position and read and return as many ascii digits as possible
    fn read_digits(&mut self, partial: String, start_pos: Position) -> Result<String, RatError> {
        let mut digits = String::new();
        self.accumulate(&mut digits, |ch| ch.is_ascii_digit());

        if digits.is_empty() {
            let end_pos = self.source.position();
            return self.lexical_error(
                LexicalError::NoDigitsInNumericLiteral,
                format!("{}{}", partial, digits),
                start_pos,
                end_pos,
            );
        }

        Ok(digits)
    }
    /// numeric literals may end with a type specifier, read here
    fn read_numeric_suffix(
        &mut self,
        is_floating_type: bool,
        partial: String,
        start_pos: Position,
    ) -> Result<String, RatError> {
        let is_integral_type = |ch: char| matches!(ch, 'i' | 'c' | 'l' | 's');
        let mut suffix = String::new();

        match self.peek() {
            Some('d' | 'f') => {
                self.consume_into(&mut suffix);
            }

            Some('u') if !is_floating_type => {
                self.consume_into(&mut suffix);
                if matches!(self.peek(), Some(ch) if is_integral_type(ch)) {
                    self.consume_into(&mut suffix);
                }
            }

            Some(ch) if !is_floating_type && is_integral_type(ch) => {
                self.consume_into(&mut suffix);
            }

            Some(ch) if !Category::is_delimiter(ch) => {
                let _ = self.read();
                let end_pos = self.source.position();
                return self.lexical_error(
                    LexicalError::UnexpectedCharAfterNumeric(ch),
                    format!("{}{}{}", partial, suffix, ch),
                    start_pos,
                    end_pos,
                );
            }

            _ => {}
        };

        Ok(suffix)
    }

    /// read as many valid characters as possible, return the longest matching symbol else return
    /// the token as an identifier or error upon failure
    fn advance_keyword_or_identifier(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let mut partial = String::new();

        self.accumulate(&mut partial, |ch| ch.is_alphanumeric() || ch == '_');

        let category = Category::is_any(&partial).unwrap_or(Category::Identifier);
        let end_pos = self.source.position();

        Ok(Some(self.emit(category, partial, start_pos, end_pos)))
    }

    /// read as many valid characters as possible, return the longest matching operator or
    /// punctuator or error upon failure
    fn advance_operator_or_punctuator(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let mut partial = String::new();

        loop {
            let Some(ch) = self.peek() else { break };
            if ch.is_alphanumeric() || ch == '_' || (ch.is_whitespace() && ch != '\n') {
                break;
            }

            let mut extended = partial.clone();
            extended.push(ch);

            if Category::is_any(&extended).is_some() {
                self.consume_into(&mut partial);
            } else {
                break;
            }
        }

        if partial.is_empty() {
            let ch = self.read().unwrap();
            let end_pos = self.source.position();
            return self.lexical_error(
                LexicalError::UnexpectedChar(ch),
                String::from(ch),
                start_pos,
                end_pos,
            );
        }

        let category = Category::is_any(&partial).unwrap_or(Category::Invalid);
        let end_pos = self.source.position();

        Ok(Some(self.emit(category, partial, start_pos, end_pos)))
    }
    /// debug assertion that function calls match prefix predicate
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
    /// debug assertion that numeric literals match regex specification
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

    /// upon error, read chars until the invalid token has closed
    fn advance_lexical_error(&mut self, err: &mut RatError) -> Result<Token, RatError> {
        let recovery = Recovery::from_error(err);
        let partial = &mut err.value;

        let opener = partial.chars().next().filter(|ch| matches!(ch, '\'' | '"'));

        while let Some(ch) = self.peek() {
            if recovery.skips_escapes() {
                if let Some(two) = self.peek_n(2) {
                    let is_escaped_opener = match opener {
                        Some('\'') => two == "\\'",
                        Some('"') => two == "\\\"",
                        _ => false,
                    };
                    if is_escaped_opener {
                        self.read().map(|c| partial.push(c));
                        self.read().map(|c| partial.push(c));
                        continue;
                    }
                }
            }

            if recovery.is_stop(ch) {
                if recovery.consume_stop() && matches!(ch, '\'' | '"') {
                    self.read().map(|c| partial.push(c));
                }
                break;
            }

            self.read().map(|c| partial.push(c));
        }

        self.errors.push(err.clone());
        err.span = Span::set(err.span.start_pos, self.source.position());
        let token = Token::new(Category::Invalid, err.value.clone(), err.span);
        Ok(token)
    }

    /// helper function to read as many chars which match a predicate
    #[inline]
    fn accumulate(&mut self, partial: &mut String, pred: impl Fn(char) -> bool) -> usize {
        let mut chars_read = 0;
        while let Some(ch) = self.source.peek() {
            if !pred(ch) {
                break;
            }

            partial.push(self.source.read().unwrap());
            chars_read += 1;
        }
        chars_read
    }

    /// helper function to ensure certain sequences are terminated
    #[inline]
    fn match_unterminated(
        &mut self,
        kind: LexicalError,
        partial: &str,
        start_pos: Position,
    ) -> Result<(), RatError> {
        match self.peek() {
            Some('\n') | None => {
                let end_pos = self.source.position();
                self.lexical_error(kind, partial, start_pos, end_pos)
            }

            _ => Ok(()),
        }
    }

    /// given a peek() call returns an expected char, push into a partial string from a source read() call
    #[inline]
    fn consume_into(&mut self, partial: &mut String) -> char {
        let ch = self
            .read()
            .expect("consume_into() requires read to return Some(_), got None");
        partial.push(ch);
        ch
    }

    /// helper read alias
    #[inline]
    fn read(&mut self) -> Option<char> {
        self.source.read()
    }

    /// helper peek alias
    #[inline]
    fn peek(&mut self) -> Option<char> {
        self.source.peek()
    }

    fn peek_n(&mut self, n: usize) -> Option<&str> {
        self.source.peek_n(n)
    }

    /// helper token constructor alias
    #[inline]
    fn emit(
        &mut self,
        category: Category,
        value: String,
        start_pos: Position,
        end_pos: Position,
    ) -> Token {
        Token::new(category, value, Span::set(start_pos, end_pos))
    }

    /// helper error constructor alias
    #[inline]
    fn lexical_error<T>(
        &mut self,
        err: LexicalError,
        value: impl Into<String>,
        start_pos: Position,
        end_pos: Position,
    ) -> Result<T, RatError> {
        Err(RatError::lexical(
            err,
            value.into(),
            Span::set(start_pos, end_pos),
        ))
    }
}
