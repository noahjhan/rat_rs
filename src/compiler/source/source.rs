use std::fs::File;
use std::io::Read;
use std::io::{self, BufReader};

#[derive(Debug)]
pub struct RatSource {
    reader: BufReader<File>,

    line_num: usize,
    col_num: usize,
    offset: usize,
    // TODO: optional lookahead buffer for peek support
    // lookahead: Option<char>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line_num: usize,
    pub col_num: usize,
    pub offset: usize,
}

impl RatSource {
    // TODO: at runtime set cwd for safer filepath
    pub fn init(filepath: &str) -> io::Result<Self> {
        let file = File::open(filepath)?;

        Ok(Self {
            reader: BufReader::new(file),
            line_num: 1,
            col_num: 1,
            offset: 0,
        })
    }

    pub fn read(&mut self) -> io::Result<Option<u8>> {
        let mut byte = [0u8; 1];

        match self.reader.read(&mut byte)? {
            0 => Ok(None),
            _ => {
                let ch = byte[0];

                self.offset += 1;

                if ch == b'\n' {
                    self.line_num += 1;
                    self.col_num = 1;
                } else {
                    self.col_num += 1;
                }

                Ok(Some(ch))
            }
        }
    }

    // TODO: peek next character without consuming
    pub fn peek(&mut self) -> io::Result<Option<u8>> {
        unimplemented!()
    }

    // TODO: skip whitespace, track newlines
    pub fn advance_whitespace(&mut self) -> io::Result<bool> {
        unimplemented!()
    }

    // TODO: check current position
    pub fn position(&self) -> Position {
        return Position {
            line_num: self.line_num,
            col_num: self.col_num,
            offset: self.offset,
        };
    }
}
