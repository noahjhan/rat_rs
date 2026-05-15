use crate::compiler::{Lexer, RatSource};

pub fn compile(filepath: &str, verbose: bool) {
    let source = match RatSource::init(filepath) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("error: {}", err);
            return;
        }
    };

    let mut lexer = Lexer::init(source);
    let tokens = lexer.get_tokens();

    for err in lexer.errors() {
        eprintln!("warning: {}", err);
    }

    for result in tokens {
        match result {
            Ok(token) if verbose => token.debug_print(),
            Ok(_) => {}
            Err(err) => {
                eprintln!("error: {}", err);
                return;
            }
        }
    }
}
