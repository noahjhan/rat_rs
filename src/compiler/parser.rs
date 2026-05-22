use crate::compiler::{Ast, ParseError, RatError, RatSource, Span, Token};
use std::collections::VecDeque;

pub struct Parser {
    source: RatSource,
    tokens: VecDeque<Token>,
}

impl Parser {
    pub fn init(source: RatSource, tokens: VecDeque<Token>) -> Self {
        Parser { source, tokens }
    }

    pub fn dispatch(&mut self) -> Result<Ast, RatError> {
        let _ = self.source;
        let _ = self.tokens;

        Ok(Ast::Program {
            statements: Vec::new(),
        })
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.front()
    }

    fn advance(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }

    fn check(&self, value: &str) -> bool {
        self.peek().map(|t| t.value == value).unwrap_or(false)
    }

    fn expect(&mut self, value: &str) -> Result<Token, RatError> {
        match self.advance() {
            Some(token) if token.value == value => Ok(token),

            Some(token) => Err(RatError::parse(
                ParseError::ExpectedGot {
                    expected: String::from(value),
                    actual: token.value.clone(),
                },
                token.value.clone(),
                token.span,
            )),

            None => Err(RatError::parse(
                ParseError::ExpectedGotEof {
                    expected: String::from(value),
                },
                value,
                Span::new(),
            )),
        }
    }
}
