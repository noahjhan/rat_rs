use crate::compiler::RatError;
use crate::compiler::RatSource;
use crate::compiler::Token;

extern crate queues;
use queues::*;

// @todo
fn lexer(source: &RatSource) -> Result<Queue<Token>, RatError> {
    unimplemented!();
}
