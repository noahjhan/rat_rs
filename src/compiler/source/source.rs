use crate::compiler::{Position, RatError, Span};

#[derive(Clone)]
pub struct RatSource {
    source: String,

    line: usize,
    col: usize,
    offset: usize,

    peeked: Option<char>,
}

impl RatSource {
    pub fn init(filepath: String) -> Result<Self, RatError> {
        let source = std::fs::read_to_string(&filepath).map_err(|e| RatError::io(&filepath, &e))?;

        Ok(RatSource {
            source,
            line: 1,
            col: 1,
            offset: 0,
            peeked: None,
        })
    }

    pub fn empty() -> Self {
        RatSource {
            source: String::new(),
            line: 1,
            col: 1,
            offset: 0,
            peeked: None,
        }
    }

    pub fn read(&mut self) -> Option<char> {
        let ch = match self.peeked.take() {
            Some(ch) => ch,
            None => self.remaining().chars().next()?,
        };

        self.offset += ch.len_utf8();

        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }

        Some(ch)
    }

    pub fn peek(&mut self) -> Option<char> {
        if self.peeked.is_none() {
            self.peeked = self.remaining().chars().next();
        }

        self.peeked
    }

    pub fn peek_n(&mut self, n: usize) -> Option<&str> {
        self.source.get(self.offset..self.offset + n)
    }

    pub fn position(&self) -> Position {
        Position {
            line: self.line,
            col: self.col,
            offset: self.offset,
        }
    }

    pub fn from_span(&self, span: Span) -> Option<String> {
        let source = self.source.clone();

        let prefix = source.get(..span.start_pos.offset)?;
        let suffix = source.get(span.end_pos.offset..)?;

        let start_pos = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let end_pos = suffix
            .find('\n')
            .map(|i| span.end_pos.offset + i + 1)
            .unwrap_or(source.len());

        let line = source.get(start_pos..end_pos)?;

        if line.len() == 0 {
            return None;
        }

        Some(String::from(line))
    }

    fn remaining(&self) -> &str {
        &self.source[self.offset..]
    }

    pub fn slice(&self, range: std::ops::Range<usize>) -> Option<&str> {
        self.source.get(range)
    }
}
