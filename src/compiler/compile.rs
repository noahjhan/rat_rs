use crate::compiler::{Lexer, RatSource};

pub fn compile(filepath: &str, verbose: bool) {
    let source = match RatSource::init(filepath) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("error:\n{}\n", err);
            return;
        }
    };

    let mut lexer = Lexer::init(source);
    match lexer.dispatch() {
        Err(err) => {
            eprintln!("error:\n{}\n", err);
            return;
        }
        _ => {}
    }

    let tokens = lexer.get_tokens();

    for err in lexer.get_errors() {
        eprintln!("warning:\n{}\n", err);
    }

    for token in tokens {
        if verbose {
            token.debug_print()
        }
    }
}
