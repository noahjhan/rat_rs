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
    pub fn error(&self, kind: LexicalError, end_pos: Position) -> RatError {
        RatError::new(kind, Span::set(self.start_pos, end_pos))
    }
}
