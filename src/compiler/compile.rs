use crate::compiler::{Lexer, RatSource, Render};

pub fn compile(filepath: String, verbose: bool) {
    let mut source = match RatSource::init(filepath) {
        Ok(source) => source,
        Err(err) => {
            let mut empty = RatSource::empty();
            let mut render = Render::init(&mut empty);
            render.print(err);
            return;
        }
    };

    let (tokens, errors) = {
        let mut lexer = Lexer::init(&mut source);
        if let Err(err) = lexer.dispatch() {
            let mut render = Render::init(&mut source);
            render.print(err);
            return;
        }
        (lexer.get_tokens().clone(), lexer.get_errors().clone())
    };

    let mut render = Render::init(&mut source);
    for token in &tokens {
        if verbose {
            token.debug_print();
        }
    }
    for err in errors {
        render.print(err);
    }
}
