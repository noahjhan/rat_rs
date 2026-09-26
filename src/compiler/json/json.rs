use crate::compiler::Token;

use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;

pub struct Json {
    tokens: VecDeque<Token>,
}

impl Json {
    pub fn init(tokens: VecDeque<Token>) -> Self {
        Json { tokens }
    }

    pub fn emit(self, filename: String) {
        let mut f = match File::create(filename) {
            Ok(f) => f,
            _ => return,
        };

        f.write(b"bonjour le monde");
    }
}
