// errors.rs
use crate::compiler::Span;
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

impl From<std::io::Error> for RatError {
    fn from(err: std::io::Error) -> Self {
        RatError::IoError(ErrorInfo {
            err: err.to_string(),
            value: String::new(),
            span: Span {
                start_line_num: 0,
                start_col_num: 0,
                start_offset: 0,
                end_line_num: 0,
                end_col_num: 0,
                end_offset: 0,
            },
        })
    }
}
