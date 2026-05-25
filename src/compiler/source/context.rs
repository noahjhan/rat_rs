use crate::compiler::{LexicalError, Position, RatError, Span};

pub struct ReadContext {
    pub partial: String,
    pub start_pos: Position,
}

impl ReadContext {
    pub fn new(start_pos: Position) -> Self {
        ReadContext {
            partial: String::new(),
            start_pos,
        }
    }

    pub fn with_prefix(start_pos: Position, prefix: impl Into<String>) -> Self {
        ReadContext {
            partial: prefix.into(),
            start_pos,
        }
    }

    #[inline]
    pub fn push(&mut self, ch: char) {
        self.partial.push(ch);
    }

    #[inline]
    pub fn push_str(&mut self, s: &str) {
        self.partial.push_str(s);
    }

    #[inline]
    pub fn error<T>(&self, kind: LexicalError, end_pos: Position) -> Result<T, RatError> {
        Err(RatError::lexical(
            kind,
            self.partial.clone(),
            Span::set(self.start_pos, end_pos),
        ))
    }

    #[inline]
    pub fn error_with_suffix<T>(
        &self,
        kind: LexicalError,
        suffix: impl Into<String>,
        end_pos: Position,
    ) -> Result<T, RatError> {
        let mut value = self.partial.clone();
        value.push_str(&suffix.into());
        Err(RatError::lexical(
            kind,
            value,
            Span::set(self.start_pos, end_pos),
        ))
    }
}
