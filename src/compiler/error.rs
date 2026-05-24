use crate::compiler::{Position, Span};
use std::io;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    Io,
    Source,
    Lexical(LexicalError),
    Parse(ParseError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RatError {
    pub kind: ErrorKind,
    pub value: String,
    pub span: Span,
}

impl RatError {
    pub fn io(_err: io::Error, span: Span) -> Self {
        Self {
            kind: ErrorKind::Io,
            value: String::new(),
            span,
        }
    }

    pub fn source(message: impl Into<String>, value: impl Into<String>, span: Span) -> Self {
        let message = message.into();
        let value = value.into();

        Self {
            kind: ErrorKind::Source,
            value: format!("{message}\n{value}"),
            span,
        }
    }

    pub fn lexical(err: LexicalError, value: impl Into<String>, span: Span) -> Self {
        Self {
            kind: ErrorKind::Lexical(err),
            value: value.into(),
            span,
        }
    }

    pub fn parse(err: ParseError, value: impl Into<String>, span: Span) -> Self {
        Self {
            kind: ErrorKind::Parse(err),
            value: value.into(),
            span,
        }
    }

    pub fn is_fatal(&self) -> bool {
        matches!(self.kind, ErrorKind::Io | ErrorKind::Source)
    }
}

impl std::fmt::Display for RatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::Io => {
                write!(f, "io error")
            }
            ErrorKind::Source => {
                write!(f, "source error")
            }
            ErrorKind::Lexical(err) => {
                write!(f, "lexical error: {}", err)
            }
            ErrorKind::Parse(err) => {
                write!(f, "parse error: {}", err)
            }
        }
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
            start_pos: zero,
            end_pos: zero,
        };

        Self::io(err, span)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexicalError {
    UnterminatedString,
    UnterminatedCharLiteral,
    UnterminatedEscapeSequence,
    UnterminatedUnicodeEscape,
    EmptyCharLiteral,
    MultipleCharsInLiteral,
    NoDigitsInNumericLiteral,
    UnterminatedMultiLineComment,
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
            Self::UnterminatedMultiLineComment => {
                write!(f, "expected closing '*/' in mutli-line comment")
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
                    "U+{cp:06X} is not a valid unicode codepoint, max is U+10FFFF"
                )
            }
            Self::SurrogateCodepoint(cp) => {
                write!(
                    f,
                    "U+{cp:04X} is a surrogate codepoint and cannot be used directly, surrogates are reserved for internal UTF-16 encoding"
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
pub enum ParseError {
    ExpectedGot { expected: String, actual: String },
    ExpectedGotEof { expected: String },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedGot { expected, actual } => {
                write!(f, "expected '{}', got {}", expected, actual)
            }
            Self::ExpectedGotEof { expected } => {
                write!(f, "expected '{}', got EOF", expected)
            }
        }
    }
}
