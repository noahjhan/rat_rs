use crate::compiler::{RatError, Span};

pub struct RatSource {
    source: String,

    line: usize,
    col: usize,
    offset: usize,

    peeked: Option<char>,
}

impl RatSource {
    pub fn init(filepath: String) -> Result<Self, RatError> {
        let source = match std::fs::read_to_string(&filepath) {
            Ok(source) => source,
            Err(_) => {
                return Err(RatError::source(
                    format!("error reading from {}", &filepath),
                    "",
                    Span::new(),
                ));
            }
        };

        Ok(RatSource {
            source: source,
            line: 1,
            col: 1,
            offset: 0,
            peeked: None,
        })
    }

    pub fn read(&mut self) -> Option<char> {
        None
    }

    pub fn peek(&mut self) -> Option<char> {
        None
    }
}

// use crate::compiler::{Position, RatError, Span};
// use std::fs::File;
// use std::io::{BufReader, Read};
//
// const BUF_SIZE: usize = 8 * 1024;
//
// pub struct RatSource {
//     reader: BufReader<File>,
//     buf: [u8; BUF_SIZE],
//     buf_pos: usize,
//     buf_len: usize,
//     offset: usize,
//     line: usize,
//     col: usize,
//
//     peeked: Option<char>,
// }
//
// impl std::fmt::Debug for RatSource {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("RatSource")
//             .field("offset", &self.offset)
//             .field("line", &self.line)
//             .field("col", &self.col)
//             .field("peeked", &self.peeked)
//             .finish_non_exhaustive()
//     }
// }
//
// impl RatSource {
//     pub fn init(filepath: &str) -> Result<Self, RatError> {
//         let file = File::open(filepath)?;
//         Ok(Self {
//             reader: BufReader::new(file),
//             buf: [0; BUF_SIZE],
//             buf_pos: 0,
//             buf_len: 0,
//             offset: 0,
//             line: 1,
//             col: 1,
//             peeked: None,
//         })
//     }
//
//     pub fn read(&mut self) -> Result<Option<char>, RatError> {
//         let ch = match self.peeked.take() {
//             Some(ch) => ch,
//             None => match self.decode_char()? {
//                 Some(ch) => ch,
//                 None => return Ok(None),
//             },
//         };
//         self.consume(ch);
//         Ok(Some(ch))
//     }
//
//     pub fn peek(&mut self) -> Result<Option<char>, RatError> {
//         if self.peeked.is_none() {
//             self.peeked = self.decode_char()?;
//         }
//         Ok(self.peeked)
//     }
//
//     pub fn position(&self) -> Position {
//         Position {
//             line: self.line,
//             col: self.col,
//             offset: self.offset,
//         }
//     }
//
//     fn fill_buf(&mut self) -> Result<bool, RatError> {
//         if self.buf_pos < self.buf_len {
//             return Ok(true);
//         }
//         let n = self.reader.read(&mut self.buf)?;
//         if n == 0 {
//             return Ok(false);
//         }
//         self.buf_pos = 0;
//         self.buf_len = n;
//         Ok(true)
//     }
//
//     fn peek_byte(&mut self) -> Result<Option<u8>, RatError> {
//         if !self.fill_buf()? {
//             return Ok(None);
//         }
//         Ok(Some(self.buf[self.buf_pos]))
//     }
//
//     fn take_byte(&mut self) -> Result<Option<u8>, RatError> {
//         match self.peek_byte()? {
//             Some(b) => {
//                 self.buf_pos += 1;
//                 Ok(Some(b))
//             }
//             None => Ok(None),
//         }
//     }
//
//     fn next_byte(&mut self) -> Result<Option<u8>, RatError> {
//         match self.take_byte()? {
//             Some(b'\r') => {
//                 if matches!(self.peek_byte()?, Some(b'\n')) {
//                     self.buf_pos += 1;
//                 }
//                 Ok(Some(b'\n'))
//             }
//             other => Ok(other),
//         }
//     }
//
//     fn decode_char(&mut self) -> Result<Option<char>, RatError> {
//         let b0 = match self.next_byte()? {
//             Some(b) => b,
//             None => return Ok(None),
//         };
//
//         if b0 < 0x80 {
//             return Ok(Some(b0 as char));
//         }
//
//         let width = match b0.leading_ones() {
//             2 => 2usize,
//             3 => 3,
//             4 => 4,
//             _ => return Err(self.utf8_error()),
//         };
//
//         let mut bytes = [b0, 0, 0, 0];
//         for slot in bytes[1..width].iter_mut() {
//             *slot = self.next_byte()?.ok_or_else(|| self.utf8_error())?;
//         }
//
//         let s = std::str::from_utf8(&bytes[..width]).map_err(|_| self.utf8_error())?;
//         Ok(s.chars().next())
//     }
//
//     fn consume(&mut self, ch: char) {
//         self.offset += ch.len_utf8();
//         if ch == '\n' {
//             self.line += 1;
//             self.col = 1;
//         } else {
//             self.col += 1;
//         }
//     }
//
//     fn utf8_error(&self) -> RatError {
//         let pos = Position {
//             line: self.line,
//             col: self.col,
//             offset: self.offset,
//         };
//         RatError::source(
//             format!("invalid utf8 sequence at offset {}", self.offset),
//             String::new(),
//             Span {
//                 start_pos: pos,
//                 end_pos: pos,
//             },
//         )
//     }
// }
