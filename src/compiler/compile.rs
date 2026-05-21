use crate::compiler::{Lexer, RatSource};

pub fn compile(filepath: &str, verbose: bool) {
    let source = match RatSource::init(filepath) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("error: {:?}", err);
            return;
        }
    };

    let mut lexer = Lexer::init(source);
    match lexer.advance_tokens() {
        Err(err) => {
            eprintln!("error: {:?}", err);
            return;
        }
        _ => {}
    }

    let tokens = lexer.get_tokens();

    for err in lexer.get_errors() {
        eprintln!("warning: {:?}", err);
    }

    for token in tokens {
        if verbose {
            token.debug_print()
        }
    }
}
