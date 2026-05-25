use crate::compiler::{
    Category, ErrorKind, LexicalError, Position, RatError, RatSource, ReadContext, Recovery, Span,
    Token,
};

use std::collections::VecDeque;
use std::ops::ControlFlow;

/// this file holds the structure associated with lexing source files
///
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
        self.advance_whitespace();
        self.advance_single_line_comment();
        self.advance_multi_line_comment()?;

        let ch = match self.peek() {
            None => return Ok(None),
            Some(c) => c,
        };

        type Action<'s> = fn(&mut Lexer<'s>) -> Result<Option<Token>, RatError>;
        let actions: &[(fn(char) -> bool, Action)] = &[
            (|ch| ch == '"', Self::advance_string_literal),
            (|ch| ch == '\'', Self::advance_character_literal),
            (|ch| ch.is_ascii_digit(), Self::advance_numeric_literal),
            (
                |ch| ch.is_alphabetic() || ch == '_',
                Self::advance_keyword_or_identifier,
            ),
            (
                |ch| ch.is_ascii_punctuation() || ch == '\n',
                Self::advance_operator_or_punctuator,
            ),
        ];

        if let Some(token) = actions
            .iter()
            .find(|(pred, _)| pred(ch))
            .map(|(_, action)| action(self))
            .transpose()?
            .flatten()
        {
            return Ok(Some(token));
        }

        self.unexpected_char()
    }

    fn advance_whitespace(&mut self) {
        self.fold_while((), |_, _, ch| {
            if !ch.is_whitespace() || ch == '\n' {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        });
    }

    fn advance_single_line_comment(&mut self) {
        match self.peek_n(2) {
            Some(s) if s == "//" => {}
            _ => return,
        }
        self.fold_while((), |_, _, ch| {
            if ch == '\n' {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        });
    }

    fn advance_multi_line_comment(&mut self) -> Result<(), RatError> {
        match self.peek_n(2) {
            Some(s) if s == "/*" => {}
            _ => return Ok(()),
        }

        let mut context = ReadContext::new(self.source.position());
        let mut stack: Vec<()> = Vec::new();

        context.push(self.read().unwrap());
        context.push(self.read().unwrap());
        stack.push(());

        while !stack.is_empty() {
            match self.peek_n(2) {
                Some(s) if s == "/*" => {
                    context.push(self.read().unwrap());
                    context.push(self.read().unwrap());
                    stack.push(());
                    continue;
                }
                Some(s) if s == "*/" => {
                    context.push(self.read().unwrap());
                    context.push(self.read().unwrap());
                    stack.pop();
                    continue;
                }
                _ => {}
            }
            self.read_or_error(&mut context, LexicalError::UnterminatedMultiLineComment)?;
        }

        Ok(())
    }

    fn advance_string_literal(&mut self) -> Result<Option<Token>, RatError> {
        let (actual, span) = self.spanned_read(|s| s.read());
        #[cfg(debug_assertions)]
        self.verify_opening_char("advance_string_literal()", '"', actual, span.start_pos);

        let mut context = ReadContext::with_prefix(span.start_pos, "\"");

        loop {
            self.check_unterminated(LexicalError::UnterminatedString, &context)?;
            let ch = self.read().unwrap();
            context.push(ch);
            match ch {
                '"' => {
                    let ((), span) = self.spanned_read_from(context.start_pos, |_| ());
                    return Ok(Some(Token::new(Category::Literal, context.partial, span)));
                }
                '\\' => {
                    let sequence = self.read_escape_sequence(&context)?;
                    context.push_str(&sequence);
                }
                _ => {}
            }
        }
    }

    fn advance_character_literal(&mut self) -> Result<Option<Token>, RatError> {
        let (actual, span) = self.spanned_read(|s| s.read());
        #[cfg(debug_assertions)]
        self.verify_opening_char("advance_character_literal()", '\'', actual, span.start_pos);

        let mut context = ReadContext::with_prefix(span.start_pos, "'");

        self.check_unterminated(LexicalError::UnterminatedCharLiteral, &context)?;
        match self.peek() {
            Some('\'') => {
                self.consume_into(&mut context.partial);
                let end_pos = self.source.position();
                return context.error(LexicalError::EmptyCharLiteral, end_pos);
            }
            Some('\\') => {
                self.consume_into(&mut context.partial);
                let sequence = self.read_escape_sequence(&context)?;
                context.push_str(&sequence);
            }
            _ => {
                self.consume_into(&mut context.partial);
            }
        }

        self.check_unterminated(LexicalError::UnterminatedCharLiteral, &context)?;
        match self.peek() {
            Some('\'') => {
                self.consume_into(&mut context.partial);
                let ((), span) = self.spanned_read_from(context.start_pos, |_| ());
                Ok(Some(Token::new(Category::Literal, context.partial, span)))
            }
            _ => {
                let end_pos = self.source.position();
                self.consume_into(&mut context.partial);
                context.error(LexicalError::MultipleCharsInLiteral, end_pos)
            }
        }
    }

    fn read_escape_sequence(&mut self, context: &ReadContext) -> Result<String, RatError> {
        match self.read() {
            Some(ch @ ('\\' | '\'' | '"' | 'n' | 'r' | 't' | 'b' | '0')) => Ok(String::from(ch)),
            Some('u') => {
                let inner_context =
                    ReadContext::with_prefix(context.start_pos, format!("{}u", context.partial));
                self.read_unicode_escape(inner_context)
            }
            Some(ch) => {
                let end_pos = self.source.position();
                context.error_with_suffix(
                    LexicalError::InvalidEscapeSequence(ch),
                    String::from(ch),
                    end_pos,
                )
            }
            None => {
                let end_pos = self.source.position();
                context.error(LexicalError::UnterminatedEscapeSequence, end_pos)
            }
        }
    }

    fn read_unicode_escape(&mut self, mut context: ReadContext) -> Result<String, RatError> {
        let u_idx = context.partial.len() - 1;
        match self.read() {
            Some('{') => context.push('{'),
            Some(ch) => {
                let end_pos = self.source.position();
                return context.error_with_suffix(
                    LexicalError::InvalidUnicodeEscapeOpener(ch),
                    String::from(ch),
                    end_pos,
                );
            }
            None => {
                let end_pos = self.source.position();
                return context.error(LexicalError::UnterminatedUnicodeEscape, end_pos);
            }
        }

        let digits_start = context.partial.len();
        loop {
            let pre_read_end_pos = self.source.position();
            match self.read() {
                Some('}') => {
                    context.push('}');
                    break;
                }
                Some(ch) if ch.is_ascii_hexdigit() => {
                    context.push(ch);
                }
                Some(ch) => {
                    return context.error_with_suffix(
                        LexicalError::InvalidUnicodeEscapeDigit(ch),
                        String::from(ch),
                        pre_read_end_pos,
                    );
                }
                None => {
                    let end_pos = self.source.position();
                    return context.error(LexicalError::UnterminatedUnicodeEscape, end_pos);
                }
            }
        }

        let hex_start = digits_start;
        let hex_end = context.partial.len() - 1;
        let hex = &context.partial[hex_start..hex_end];
        if hex.is_empty() || hex.len() > 6 {
            let end_pos = self.source.position();
            return context.error(LexicalError::InvalidUnicodeDigitCount(hex.len()), end_pos);
        }

        let codepoint = u32::from_str_radix(hex, 16).unwrap();
        let end_pos = self.source.position();
        if codepoint > 0x10_FFFF {
            return context.error(LexicalError::InvalidUnicodeCodepoint(codepoint), end_pos);
        }

        if (0xD800..=0xDFFF).contains(&codepoint) {
            return context.error(LexicalError::SurrogateCodepoint(codepoint), end_pos);
        }

        Ok(context.partial[u_idx..].to_string())
    }

    fn advance_numeric_literal(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let mut context = ReadContext::new(start_pos);

        let prefix = self.read_digits(&context)?;
        context.push_str(&prefix);

        let mut is_floating_type = false;
        if matches!(self.peek(), Some('.')) {
            is_floating_type = true;
            self.consume_into(&mut context.partial);
            let extended = self.read_digits(&context)?;
            context.push_str(&extended);
        }

        let suffix = self.read_numeric_suffix(is_floating_type, &context)?;
        context.push_str(&suffix);

        let ((), span) = self.spanned_read_from(context.start_pos, |_| ());
        let token = Token::new(Category::Literal, context.partial, span);

        #[cfg(debug_assertions)]
        self.verify_numeric_literal("advance_numeric_literal()", &token);

        Ok(Some(token))
    }

    fn read_digits(&mut self, context: &ReadContext) -> Result<String, RatError> {
        let mut digits = String::new();
        self.accumulate(&mut digits, |ch| ch.is_ascii_digit());
        if digits.is_empty() {
            let end_pos = self.source.position();
            return context.error(LexicalError::NoDigitsInNumericLiteral, end_pos);
        }
        Ok(digits)
    }

    fn read_numeric_suffix(
        &mut self,
        is_floating_type: bool,
        context: &ReadContext,
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
                return context.error_with_suffix(
                    LexicalError::UnexpectedCharAfterNumeric(ch),
                    String::from(ch),
                    end_pos,
                );
            }
            _ => {}
        }

        Ok(suffix)
    }

    fn advance_keyword_or_identifier(&mut self) -> Result<Option<Token>, RatError> {
        let (partial, span) = self.spanned_read(|s| {
            let mut buf = String::new();
            s.accumulate(&mut buf, |ch| ch.is_alphanumeric() || ch == '_');
            buf
        });

        #[cfg(debug_assertions)]
        assert!(partial
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_'));

        let category = Category::is_any(&partial).unwrap_or(Category::Identifier);
        Ok(Some(Token::new(category, partial, span)))
    }

    fn advance_operator_or_punctuator(&mut self) -> Result<Option<Token>, RatError> {
        let (result, span) = self.spanned_read(|s| {
            let mut partial = String::new();
            loop {
                let Some(ch) = s.peek() else { break };
                if ch.is_alphanumeric() || ch == '_' || (ch.is_whitespace() && ch != '\n') {
                    break;
                }
                let mut extended = partial.clone();
                extended.push(ch);
                if Category::is_any(&extended).is_some() {
                    s.consume_into(&mut partial);
                } else {
                    break;
                }
            }
            partial
        });

        if result.is_empty() {
            return Ok(None);
        }

        let category = Category::is_any(&result).unwrap_or(Category::Invalid);
        Ok(Some(Token::new(category, result, span)))
    }

    fn unexpected_char(&mut self) -> Result<Option<Token>, RatError> {
        let start_pos = self.source.position();
        let ch = self.read().unwrap();

        let context = ReadContext::with_prefix(start_pos, String::from(ch));
        let end_pos = self.source.position();

        context.error(LexicalError::UnexpectedChar(ch), end_pos)
    }

    fn verify_opening_char(
        &self,
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

    fn verify_numeric_literal(&self, function_name: &str, token: &Token) {
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

    #[inline]
    fn fold_while<S, F>(&mut self, init: S, mut f: F) -> S
    where
        F: FnMut(&mut Self, S, char) -> ControlFlow<S, S>,
    {
        let mut state = init;
        while let Some(ch) = self.peek() {
            match f(self, state, ch) {
                ControlFlow::Continue(s) => {
                    self.read();
                    state = s;
                }
                ControlFlow::Break(s) => return s,
            }
        }
        state
    }

    #[inline]
    fn accumulate(&mut self, partial: &mut String, pred: impl Fn(char) -> bool) -> usize {
        let mut count = 0;
        while let Some(ch) = self.source.peek() {
            if !pred(ch) {
                break;
            }
            partial.push(self.source.read().unwrap());
            count += 1;
        }
        count
    }

    #[inline]
    fn check_unterminated(
        &mut self,
        kind: LexicalError,
        context: &ReadContext,
    ) -> Result<(), RatError> {
        match self.peek() {
            Some('\n') | None => {
                let end_pos = self.source.position();
                context.error(kind, end_pos)
            }
            _ => Ok(()),
        }
    }

    #[inline]
    fn read_or_error(
        &mut self,
        context: &mut ReadContext,
        eof_error: LexicalError,
    ) -> Result<char, RatError> {
        match self.read() {
            Some(ch) => {
                context.push(ch);
                Ok(ch)
            }
            None => {
                let end_pos = self.source.position();
                context.error(eof_error, end_pos)
            }
        }
    }

    #[inline]
    fn spanned_read<T, F>(&mut self, f: F) -> (T, Span)
    where
        F: FnOnce(&mut Self) -> T,
    {
        let start_pos = self.source.position();
        let value = f(self);
        let end_pos = self.source.position();
        (value, Span::set(start_pos, end_pos))
    }

    #[inline]
    fn spanned_read_from<T, F>(&mut self, start_pos: Position, f: F) -> (T, Span)
    where
        F: FnOnce(&mut Self) -> T,
    {
        let value = f(self);
        let end_pos = self.source.position();
        (value, Span::set(start_pos, end_pos))
    }

    #[inline]
    fn consume_into(&mut self, partial: &mut String) -> char {
        let ch = self
            .read()
            .expect("consume_into() requires read to return Some(_), got None");
        partial.push(ch);
        ch
    }

    #[inline]
    fn read(&mut self) -> Option<char> {
        self.source.read()
    }

    #[inline]
    fn peek(&mut self) -> Option<char> {
        self.source.peek()
    }

    #[inline]
    fn peek_n(&mut self, n: usize) -> Option<&str> {
        self.source.peek_n(n)
    }
}
