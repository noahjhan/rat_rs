use crate::compiler::Lexer;
use crate::compiler::RatSource;
use std::collections::VecDeque;

pub fn compile(filepath: &str, verbose: bool) {
    // @todo Add some filepath check here

    let source = match RatSource::init(filepath) {
        Ok(source) => source,
        Err(_) => return,
    };

    // takes a rat source file, returns a lexer with a token queue
    let mut deque = VecDeque::new();
    let mut lexer = Lexer::init(source);

    loop {
        match lexer.advance_token() {
            Ok(Some(token)) => {
                deque.push_back(token);
            }

            Ok(None) => break,
            Err(_) => return,
        }
    }

    if verbose {
        for token in deque {
            println!("{:#?}", token);
            println!();
        }
    }
}
