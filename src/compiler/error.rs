use crate::compiler::{Position, Span};
use std::io;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RatError {
    pub kind: ErrorKind,
    pub message: String,
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Io,
    Source,
    Lexical,
    Internal,
}

impl RatError {
    pub fn io(err: io::Error, span: Span) -> Self {
        RatError {
            kind: ErrorKind::Io,
            message: err.to_string(),
            value: String::new(),
            span,
        }
    }

    pub fn source(message: impl Into<String>, value: impl Into<String>, span: Span) -> Self {
        RatError {
            kind: ErrorKind::Source,
            message: message.into(),
            value: value.into(),
            span,
        }
    }

    pub fn lexical(message: impl Into<String>, value: impl Into<String>, span: Span) -> Self {
        RatError {
            kind: ErrorKind::Lexical,
            message: message.into(),
            value: value.into(),
            span,
        }
    }

    pub fn internal(message: impl Into<String>, value: impl Into<String>, span: Span) -> Self {
        RatError {
            kind: ErrorKind::Internal,
            message: message.into(),
            value: value.into(),
            span,
        }
    }

    pub fn is_fatal(&self) -> bool {
        matches!(
            self.kind,
            ErrorKind::Io | ErrorKind::Source | ErrorKind::Internal
        )
    }
}

impl std::fmt::Display for RatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self.kind {
            ErrorKind::Io => "IO error",
            ErrorKind::Source => "source error",
            ErrorKind::Lexical => "lexical error",
            ErrorKind::Internal => "internal error",
        };
        let pos = self.span.start;
        write!(f, "{kind} at {}:{}\n{}\n", pos.line, pos.col, self.message)?;
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
