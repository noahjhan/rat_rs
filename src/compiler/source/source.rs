use crate::compiler::RatError;
use std::fs::File;
use std::io::Read;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct RatSource {
    reader: BufReader<File>,
    line_num: usize,
    col_num: usize,
    offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line_num: usize,
    pub col_num: usize,
    pub offset: usize,
}

impl RatSource {
    pub fn init(filepath: &str) -> Result<Self, RatError> {
        let file = File::open(filepath)?;
        Ok(Self {
            reader: BufReader::new(file),
            line_num: 1,
            col_num: 1,
            offset: 0,
        })
    }

    pub fn read(&mut self) -> Result<Option<u8>, RatError> {
        let mut byte = [0u8; 1];
        match self.reader.read(&mut byte)? {
            0 => Ok(None),
            _ => {
                let b = byte[0];
                self.offset += 1;
                if b == b'\n' {
                    self.line_num += 1;
                    self.col_num = 1;
                } else {
                    self.col_num += 1;
                }
                Ok(Some(b))
            }
        }
    }

    pub fn peek(&mut self) -> Result<Option<u8>, RatError> {
        let buffer = self.reader.fill_buf()?;
        Ok(buffer.get(0).copied())
    }

    pub fn position(&self) -> Position {
        Position {
            line_num: self.line_num,
            col_num: self.col_num,
            offset: self.offset,
        }
    }

    pub fn get_line_num(&self) -> usize {
        self.line_num
    }

    pub fn get_col_num(&self) -> usize {
        self.col_num
    }

    pub fn get_offset(&self) -> usize {
        self.offset
    }
}
