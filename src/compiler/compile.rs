use crate::compiler::{Ast, Lexer, Parser, RatSource, Render};

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

    let mut lexer = Lexer::init(&mut source);

    let (tokens, errors) = match lexer.dispatch() {
        Err(err) => {
            let mut render = Render::init(&mut source);
            render.print(err);
            return;
        }
        Ok((tokens, errors)) => (tokens, errors),
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

    let mut parser = Parser::init(tokens);
    let program = match parser.dispatch() {
        Ok(Some(program)) => program,
        _ => return,
    };

    if !verbose {
        return;
    }

    if let Ast::Program { statements } = program {
        for ast in statements {
            if let Ast::Invalid { token } = ast {
                // token.debug_print();
            }
        }
    }
}
