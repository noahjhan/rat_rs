use crate::compiler::{RatError, Span};
use std::fs::File;
use std::io::{BufReader, Read};

const BUF_SIZE: usize = 8 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub line: usize,
    pub col: usize,
    pub offset: usize,
}

#[derive(Debug)]
pub struct RatSource {
    reader: BufReader<File>,
    buf: [u8; BUF_SIZE],
    pos: usize,
    len: usize,

    offset: usize,
    line: usize,
    col: usize,

    peeked: Option<char>,
}

impl RatSource {
    pub fn init(filepath: &str) -> Result<Self, RatError> {
        let file = File::open(filepath)?;
        Ok(Self {
            reader: BufReader::new(file),
            buf: [0; BUF_SIZE],
            pos: 0,
            len: 0,
            offset: 0,
            line: 1,
            col: 1,
            peeked: None,
        })
    }

    pub fn read(&mut self) -> Result<Option<char>, RatError> {
        let ch = match self.peeked.take() {
            Some(ch) => ch,
            None => match self.decode_char()? {
                Some(ch) => ch,
                None => return Ok(None),
            },
        };

        self.consume_char(ch);
        Ok(Some(ch))
    }

    pub fn peek(&mut self) -> Result<Option<char>, RatError> {
        if self.peeked.is_none() {
            self.peeked = self.decode_char()?;
        }
        Ok(self.peeked)
    }

    pub fn position(&self) -> Position {
        Position {
            line: self.line,
            col: self.col,
            offset: self.offset,
        }
    }

    fn fill_buf(&mut self) -> Result<bool, RatError> {
        if self.pos < self.len {
            return Ok(true);
        }
        let b = self.reader.read(&mut self.buf)?;
        if b == 0 {
            return Ok(false);
        }
        self.pos = 0;
        self.len = b;
        Ok(true)
    }

    fn peek_buf_byte(&mut self) -> Result<Option<u8>, RatError> {
        if !self.fill_buf()? {
            return Ok(None);
        }
        Ok(Some(self.buf[self.pos]))
    }

    fn take_buf_byte(&mut self) -> Result<Option<u8>, RatError> {
        match self.peek_buf_byte()? {
            Some(b) => {
                self.pos += 1;
                Ok(Some(b))
            }
            None => Ok(None),
        }
    }

    fn next_byte(&mut self) -> Result<Option<u8>, RatError> {
        let b = match self.take_buf_byte()? {
            Some(b) => b,
            None => return Ok(None),
        };

        if b == b'\r' {
            if matches!(self.peek_buf_byte()?, Some(b'\n')) {
                self.pos += 1;
            }
            return Ok(Some(b'\n'));
        }

        Ok(Some(b))
    }

    fn decode_char(&mut self) -> Result<Option<char>, RatError> {
        let b = match self.next_byte()? {
            Some(b) => b,
            None => return Ok(None),
        };

        if b < 0x80 {
            return Ok(Some(b as char));
        }

        let (width, mut cp) = self.decode_utf8_lead(b)?;

        for _ in 1..width {
            let cont = self
                .next_byte()?
                .ok_or_else(|| self.invalid_utf8(self.offset))?;
            if cont & 0xC0 != 0x80 {
                return Err(self.invalid_utf8(self.offset));
            }
            cp = (cp << 6) | (cont & 0x3F) as u32;
        }

        let ch = char::from_u32(cp).ok_or_else(|| self.invalid_utf8(self.offset))?;
        Ok(Some(ch))
    }

    fn consume_char(&mut self, ch: char) {
        self.offset += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
    }

    fn decode_utf8_lead(&self, b: u8) -> Result<(usize, u32), RatError> {
        match b.leading_ones() {
            2 => Ok((2, (b & 0x1F) as u32)),
            3 => Ok((3, (b & 0x0F) as u32)),
            4 => Ok((4, (b & 0x07) as u32)),
            _ => Err(self.invalid_utf8(self.offset)),
        }
    }

    fn invalid_utf8(&self, offset: usize) -> RatError {
        let pos = Position {
            line: self.line,
            col: self.col,
            offset,
        };

        RatError::source(
            format!("invalid byte sequence at offset: {:?}", offset),
            String::new(),
            Span {
                start: pos,
                end: pos,
            },
        )
    }
}
