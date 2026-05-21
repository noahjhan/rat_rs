use crate::compiler::{Position, Span};
use std::io;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexicalError {
    UnterminatedString,
    UnterminatedCharLiteral,
    UnterminatedEscapeSequence,
    UnterminatedUnicodeEscape,
    EmptyCharLiteral,
    MultipleCharsInLiteral,
    NoDigitsInNumericLiteral,
    InvalidEscapeSequence(char),
    InvalidUnicodeEscapeOpener(char),
    InvalidUnicodeEscapeDigit(char),
    InvalidUnicodeDigitCount(usize),
    InvalidUnicodeCodepoint(u32),
    SurrogateCodepoint(u32),
    UnexpectedChar(char),
    UnexpectedCharAfterNumeric(char),
}

impl std::fmt::Display for LexicalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnterminatedString => {
                write!(f, "unterminated string literal")
            }

            Self::UnterminatedCharLiteral => {
                write!(f, "unterminated character literal")
            }

            Self::UnterminatedEscapeSequence => {
                write!(f, "unterminated escape sequence at EOF")
            }

            Self::UnterminatedUnicodeEscape => {
                write!(f, "unterminated unicode escape sequence")
            }

            Self::EmptyCharLiteral => {
                write!(f, "empty character literal")
            }

            Self::MultipleCharsInLiteral => {
                write!(f, "character literal should only contain one character")
            }

            Self::NoDigitsInNumericLiteral => {
                write!(f, "expected ascii digits in numeric literal")
            }

            Self::InvalidEscapeSequence(ch) => {
                write!(f, "invalid escape sequence '\\{ch}'")
            }

            Self::InvalidUnicodeEscapeOpener(ch) => {
                write!(f, "expected '{{' after '\\u' in unicode escape, got '{ch}'")
            }

            Self::InvalidUnicodeEscapeDigit(ch) => {
                write!(f, "expected hex digit in unicode escape, got '{ch}'")
            }

            Self::InvalidUnicodeDigitCount(n) => {
                write!(
                    f,
                    "unicode escape must include between 1 and 6 digits, got {n}"
                )
            }

            Self::InvalidUnicodeCodepoint(cp) => {
                write!(
                    f,
                    "U+{cp:06X} is not a valid unicode codepoint, \
                     max is U+10FFFF"
                )
            }

            Self::SurrogateCodepoint(cp) => {
                write!(
                    f,
                    "U+{cp:04X} is a surrogate codepoint and cannot \
                     be used directly, surrogates are reserved for \
                     internal UTF-16 encoding"
                )
            }

            Self::UnexpectedChar(ch) => {
                write!(f, "unexpected character '{ch}'")
            }

            Self::UnexpectedCharAfterNumeric(ch) => {
                write!(f, "unexpected character '{ch}' after numeric literal")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InternalError {
    ExpectedGot {
        function_name: String,
        expected: char,
        actual: char,
    },

    ExpectedGotEof {
        function_name: String,
        expected: char,
    },

    UnmatchedRegex {
        function_name: String,
        actual: String,
    },
}

impl std::fmt::Display for InternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedGot {
                function_name,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "in {:?} expected {:?}, got {:?}",
                    function_name, expected, actual
                )
            }

            Self::ExpectedGotEof {
                function_name,
                expected,
            } => {
                write!(f, "in {:?} expected {:?}, got EOF", function_name, expected)
            }

            Self::UnmatchedRegex {
                function_name,
                actual,
            } => {
                write!(
                    f,
                    "in {:?} {:?} did not match regex specification",
                    function_name, actual
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RatError {
    pub kind: ErrorKind,
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    Io,
    Source,
    Lexical(LexicalError),
    Internal(InternalError),
}

impl RatError {
    pub fn io(_err: io::Error, span: Span) -> Self {
        RatError {
            kind: ErrorKind::Io,
            value: String::new(),
            span,
        }
    }

    pub fn source(message: impl Into<String>, value: impl Into<String>, span: Span) -> Self {
        RatError {
            kind: ErrorKind::Source,
            value: format!("{}\n{}", message.into(), value.into()),
            span,
        }
    }

    pub fn lexical(err: LexicalError, value: impl Into<String>, span: Span) -> Self {
        RatError {
            kind: ErrorKind::Lexical(err),
            value: value.into(),
            span,
        }
    }

    pub fn internal(err: InternalError, value: impl Into<String>, span: Span) -> Self {
        RatError {
            kind: ErrorKind::Internal(err),
            value: value.into(),
            span,
        }
    }

    pub fn is_fatal(&self) -> bool {
        matches!(
            self.kind,
            ErrorKind::Io | ErrorKind::Source | ErrorKind::Internal(_)
        )
    }
}

impl std::fmt::Display for RatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match &self.kind {
            ErrorKind::Io => "io error".to_string(),

            ErrorKind::Source => "source error".to_string(),

            ErrorKind::Lexical(err) => {
                format!("lexical error: {}", err)
            }

            ErrorKind::Internal(err) => {
                format!("internal error: {}", err)
            }
        };

        let pos = self.span.start;

        write!(f, "{kind} at {}:{}\n", pos.line, pos.col)?;

        if !self.value.is_empty() {
            write!(f, "{}", self.value)?;
        }

        Ok(())
    }
}

impl std::error::Error for RatError {}

impl From<io::Error> for RatError {
    fn from(err: io::Error) -> Self {
        let zero = Position {
            line: 0,
            col: 0,
            offset: 0,
        };

        let span = Span {
            start: zero,
            end: zero,
        };

        RatError::io(err, span)
    }
}
