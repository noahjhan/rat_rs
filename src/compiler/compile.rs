use crate::compiler::{Lexer, RatSource, Render};

pub fn compile(filepath: String, verbose: bool) {
    let source = match RatSource::init(filepath) {
        Ok(source) => source,
        Err(err) => {
            let mut render = Render::init(RatSource::empty());
            render.print(err);
            return;
        }
    };

    let (tokens, errors) = {
        let mut lexer = Lexer::init(source.clone());

        if let Err(err) = lexer.dispatch() {
            let mut render = Render::init(source.clone());
            render.print(err);
            return;
        }

        (lexer.get_tokens().clone(), lexer.get_errors().clone())
    };

    let mut render = Render::init(source.clone());

    for token in tokens {
        if verbose {
            token.debug_print();
        }
    }

    for err in errors {
        render.print(err);
    }
}
