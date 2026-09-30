use crate::compiler::{Lexer, Parser, Program, RatError, RatSource, Render, Token};
use std::collections::VecDeque;
use std::time;

/// Compiler stages composition
pub fn compile(filepath: String, verbose: bool) {
    let mut now = time::Instant::now();
    let mut source = match source(filepath) {
        Some(source) => source,
        None => return,
    };
    let source_duration = now.elapsed();

    now = time::Instant::now();
    let (tokens, errors) = lexer(&mut source);
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

    if verbose {
        performance_print(source_duration, lexer_duration, parser_duration);
    }
}

/// Create a readable RatSource from a .rat file
fn source(filepath: String) -> Option<RatSource> {
    RatSource::init(filepath)
        .map_err(|err| eprintln!("error: {err}"))
        .ok()
}

/// Create a lexer and call dispatch to return the tokenized source
fn lexer(source: &mut RatSource) -> (VecDeque<Token>, Vec<RatError>) {
    Lexer::init(source).dispatch()
}

/// Returns an AST from the parsed token deque
fn parser(tokens: VecDeque<Token>) -> Option<Program> {
    let mut parser = Parser::init(tokens);
    match parser.dispatch() {
        Ok(Some(program)) => Some(program),
        _ => None,
    }
}

/// Prints generated token deque and lexical errors after lexing
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

/// Prints generated AST after parsing
fn debug_print_parser(ast: &Program) {
    println!("{:#?}\n", ast);
}

/// Used to compare pipeline stage performance
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
