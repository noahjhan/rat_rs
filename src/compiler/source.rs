use std::fs::File;
use std::io::Read;
use std::io::{self, BufReader};

#[derive(Debug)]
pub struct Source {
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

impl Source {
    // TODO: at runtime set cwd for safer filepath
    pub fn init(filepath: &str) -> io::Result<Self> {
        let file = File::open(filepath)?;

        Ok(Self {
            reader: BufReader::new(file),
            line_num: 0,
            col_num: 0,
            offset: 0,
        })
    }

    // TODO: cleanup
    pub fn close(&self) {
        unimplemented!()
    }

    pub fn read(&mut self) -> io::Result<Option<u8>> {
        let mut buf = [0u8; 1];

        let n = self.reader.read(&mut buf)?;

        // EOF
        if n == 0 {
            return Ok(None);
        }

        let c = buf[0];

        self.offset += 1;

        if c == b'\n' {
            self.line_num += 1;
            self.col_num = 0;
        } else {
            self.col_num += 1;
        }

        Ok(Some(c))
    }

    // TODO: peek next character without consuming
    pub fn peek(&mut self) -> io::Result<Option<char>> {
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
