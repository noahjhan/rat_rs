use crate::compiler::Lexer;
use crate::compiler::RatSource;

pub fn compile(filepath: &str, verbose: bool) {
    // @todo Add some filepath check here

    let source = match RatSource::init(filepath) {
        Ok(source) => source,
        Err(_) => return,
    };

    // takes a rat source file, returns a lexer with a token queue
    let mut lexer = Lexer::init(source);
    let deque = lexer.tokens();
    let errors = lexer.errors();

    if !errors.is_empty() {
        for err in errors {
            println!("{:?}\n", err);
        }
        // return;
    }

    for token in deque {
        match token {
            Ok(mut token) => {
                if verbose {
                    token.debug_print();
                }
            }
            Err(err) => {
                println!("{:?}\n", err);
            }
        }
    }
}
