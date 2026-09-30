use crate::compiler::Span;
use std::io;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{kind}")]
pub struct RatError {
    pub kind: ErrorKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErrorKind {
    #[error("could not read '{path}': {kind}")]
    Io { path: String, kind: io::ErrorKind },

    #[error("lexical error: {0}")]
    Lexical(#[from] LexicalError),

    #[error("parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("semantic error: {0}")]
    Semantic(#[from] SemanticError),
}

impl RatError {
    pub fn new(kind: impl Into<ErrorKind>, span: Span) -> Self {
        Self {
            kind: kind.into(),
            span,
        }
    }

    pub fn io(path: impl Into<String>, err: &io::Error) -> Self {
        Self::new(
            ErrorKind::Io {
                path: path.into(),
                kind: err.kind(),
            },
            Span::new(),
        )
    }

    pub fn is_fatal(&self) -> bool {
        matches!(self.kind, ErrorKind::Io { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LexicalError {
    #[error("unterminated string literal")]
    UnterminatedString,
    #[error("unterminated character literal")]
    UnterminatedCharLiteral,
    #[error("unterminated escape sequence")]
    UnterminatedEscapeSequence,
    #[error("unterminated unicode escape")]
    UnterminatedUnicodeEscape,
    #[error("empty character literal")]
    EmptyCharLiteral,
    #[error("character literal should only contain one character")]
    MultipleCharsInLiteral,
    #[error("expected ascii digits in numeric literal")]
    NoDigitsInNumericLiteral,
    #[error("expected closing '*/' in multi-line comment")]
    UnterminatedMultiLineComment,
    #[error("invalid escape sequence '\\{0}'")]
    InvalidEscapeSequence(char),
    #[error("expected '{{' after '\\u' in unicode escape, got '{0}'")]
    InvalidUnicodeEscapeOpener(char),
    #[error("expected hex digit in unicode escape, got '{0}'")]
    InvalidUnicodeEscapeDigit(char),
    #[error("unicode escape must include between 1 and 6 digits, got {0}")]
    InvalidUnicodeDigitCount(usize),
    #[error("U+{0:06X} is not a valid unicode codepoint, max is U+10FFFF")]
    InvalidUnicodeCodepoint(u32),
    #[error(
        "U+{0:04X} is a surrogate codepoint and cannot be used directly, \
         surrogates are reserved for internal UTF-16 encoding"
    )]
    SurrogateCodepoint(u32),
    #[error("unexpected character '{0}'")]
    UnexpectedChar(char),
    #[error("unexpected character '{0}' after numeric literal")]
    UnexpectedCharAfterNumeric(char),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseError {
    #[error("expected '{expected}', got {}", .found.as_deref().unwrap_or("EOF"))]
    Expected {
        expected: String,
        found: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SemanticError {
    #[error("parameter '{0}' is already declared")]
    ParameterRedeclaration(String),
    #[error("function '{0}' is already declared")]
    FunctionRedeclaration(String),
    #[error("global '{0}' is already declared")]
    GlobalRedeclaration(String),
    #[error("unknown function '{0}'")]
    UnknownFunction(String),
    #[error("unknown identifier '{0}'")]
    UnknownIdentifier(String),
    #[error("identifier '{0}' is already declared")]
    IdentifierRedeclaration(String),
}
