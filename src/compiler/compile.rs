use crate::compiler::{Lexer, Parser, Program, RatError, RatSource, Render, Token};
use std::collections::VecDeque;
use std::time;

pub fn compile(filepath: String, verbose: bool) {
    let mut now = time::Instant::now();
    let mut source = match source(filepath) {
        Some(source) => source,
        None => return,
    };
    let source_duration = now.elapsed();

    now = time::Instant::now();
    let (tokens, errors) = match lexer(&mut source) {
        Some(result) => result,
        None => return,
    };
    let lexer_duration = now.elapsed();

    debug_print_lexer(&mut source, &tokens, errors, verbose);

    now = time::Instant::now();
    let ast = match parser(tokens) {
        Some(ast) => ast,
        None => return,
    };
    let parser_duration = now.elapsed();

    if verbose {
        debug_print_parser(&ast);
    }

    performance_print(source_duration, lexer_duration, parser_duration);
}

fn source(filepath: String) -> Option<RatSource> {
    match RatSource::init(filepath) {
        Ok(source) => Some(source),
        Err(err) => {
            let mut empty = RatSource::empty();
            let mut render = Render::init(&mut empty);
            render.print(err);
            None
        }
    }
}

fn lexer(source: &mut RatSource) -> Option<(VecDeque<Token>, Vec<RatError>)> {
    let mut lexer = Lexer::init(source);
    match lexer.dispatch() {
        Err(err) => {
            let mut render = Render::init(source);
            render.print(err);
            None
        }
        Ok((tokens, errors)) => Some((tokens, errors)),
    }
}

fn parser(tokens: VecDeque<Token>) -> Option<Program> {
    let mut parser = Parser::init(tokens);
    match parser.dispatch() {
        Ok(Some(program)) => Some(program),
        _ => None,
    }
}

fn debug_print_lexer(
    source: &mut RatSource,
    tokens: &VecDeque<Token>,
    errors: Vec<RatError>,
    verbose: bool,
) {
    let mut render = Render::init(source);

    for token in tokens {
        if verbose {
            token.debug_print();
        }
    }

    for err in errors {
        render.print(err);
    }
}

fn debug_print_parser(ast: &Program) {
    println!("{:#?}\n", ast);
}

fn performance_print(
    source_duration: time::Duration,
    lexer_duration: time::Duration,
    parser_duration: time::Duration,
) {
    println!("Source Initialization Time: {:?}", source_duration);
    println!("Lexer Execution Time:       {:?}", lexer_duration);
    println!("Parser Execution Time:      {:?}", parser_duration);
    println!("\n");
}
