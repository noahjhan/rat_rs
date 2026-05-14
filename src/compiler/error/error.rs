use crate::compiler::{Position, Span};
use std::io;

#[derive(Debug)]
pub struct ErrorInfo {
    pub err: String,
    pub value: String,
    pub span: Span,
}

#[derive(Debug)]
pub enum RatError {
    IoError(ErrorInfo),
    SourceError(ErrorInfo),
    LexicalError(ErrorInfo),
    InternalError(ErrorInfo),
}

impl RatError {
    pub fn io(err: io::Error, value: impl Into<String>, span: Span) -> Self {
        RatError::IoError(ErrorInfo {
            err: err.to_string(),
            value: value.into(),
            span,
        })
    }

    pub fn source(err: impl Into<String>, value: impl Into<String>, span: Span) -> Self {
        RatError::SourceError(ErrorInfo {
            err: err.into(),
            value: value.into(),
            span,
        })
    }

    pub fn lexical(err: impl Into<String>, value: impl Into<String>, span: Span) -> Self {
        RatError::LexicalError(ErrorInfo {
            err: err.into(),
            value: value.into(),
            span,
        })
    }

    pub fn internal(err: impl Into<String>, value: impl Into<String>, span: Span) -> Self {
        RatError::InternalError(ErrorInfo {
            err: err.into(),
            value: value.into(),
            span,
        })
    }
}

impl From<io::Error> for RatError {
    fn from(err: io::Error) -> Self {
        let zero = Position {
            line: 0,
            col: 0,
            offset: 0,
        };

        RatError::IoError(ErrorInfo {
            err: err.to_string(),
            value: String::new(),
            span: Span {
                start: zero,
                end: zero,
            },
        })
    }
}
