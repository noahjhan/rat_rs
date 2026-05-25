use crate::compiler::{Ast, Category, ParseError, RatError, RatSource, Span, SymbolTable, Token};
use std::collections::VecDeque;

pub struct Parser {
    source: RatSource,
    tokens: VecDeque<Token>,
    symbol_table: SymbolTable,
}

impl Parser {
    pub fn init(source: RatSource, tokens: VecDeque<Token>) -> Self {
        Parser {
            source,
            tokens,
            symbol_table: SymbolTable::init(),
        }
    }

    pub fn dispatch(&mut self) -> Result<Ast, RatError> {
        self.check_enter_scope();
        self.check_exit_scope();

        {
            // remove this
            let _ = self.source;
            let _ = self.tokens;
            let _ = self.check("");
            let _ = self.expect("");
        }

        Ok(Ast::Program {
            statements: Vec::new(),
        })
    }

    fn check_enter_scope(&mut self) {
        let token = match self.peek() {
            Some(token) => token,
            None => return,
        };

        if token.kind == Category::Punctuator && token.value == "{" {
            self.advance();
            self.symbol_table.enter_scope();
        }
    }

    fn check_exit_scope(&mut self) {
        let token = match self.peek() {
            Some(token) => token,
            None => return,
        };

        if token.kind == Category::Punctuator && token.value == "}" {
            self.advance();
            self.symbol_table.exit_scope();
        }
    }

    #[inline]
    fn peek(&self) -> Option<&Token> {
        self.tokens.front()
    }

    #[inline]
    fn advance(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }

    #[inline]
    fn check(&self, value: &str) -> bool {
        self.peek().map(|t| t.value == value).unwrap_or(false)
    }

    #[inline]
    fn expect(&mut self, value: &str) -> Result<Token, RatError> {
        match self.advance() {
            Some(token) if token.value == value => Ok(token),

            Some(token) => self.parse_error(
                ParseError::ExpectedGot {
                    expected: String::from(value),
                    actual: token.value.clone(),
                },
                token.value,
                token.span,
            ),

            None => self.parse_error(
                ParseError::ExpectedGotEof {
                    expected: String::from(value),
                },
                value,
                Span::new(),
            ),
        }
    }

    #[inline]
    fn parse_error<T>(
        &mut self,
        err: ParseError,
        value: impl Into<String>,
        span: Span,
    ) -> Result<T, RatError> {
        Err(RatError::parse(err, value.into(), span))
    }
}
