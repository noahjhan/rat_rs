use crate::compiler::{
    Ast, Category, ConstituentIdentifier, ConstituentKeyword, ConstituentLiteral,
    ConstituentPunctuator, ConstituentType, ConstituientOperator, Expr, Kind, ParseError, RatError,
    RatSource, Span, SymbolTable, Token,
};
use std::collections::VecDeque;

pub struct Parser {
    tokens: VecDeque<Token>,
    symbol_table: SymbolTable,
    program: Ast,
}

impl Parser {
    pub fn init(tokens: VecDeque<Token>) -> Self {
        Parser {
            tokens,
            symbol_table: SymbolTable::init(),
            program: Ast::Program {
                statements: Vec::new(),
            },
        }
    }

    pub fn dispatch(&mut self) -> Result<Option<Ast>, RatError> {
        if self.tokens.is_empty() {
            return Ok(None);
        }

        loop {
            self.parse_newline();
            self.parse_scope();

            let mut a = 10;
            print!("{}", a);
            let b = a = 5;
            print!("{}", a);
            print!("{:?}", b);

            let token = match self.peek() {
                Some(t) => t,
                None => break,
            };

            let ast = match token.category {
                Category::Identifier => {
                    unimplemented!("parse identifier")
                }
                Category::Keyword => {
                    unimplemented!("parse keyword")
                }
                Category::Literal => {
                    unimplemented!("parse literal")
                }
                Category::Punctuator => {
                    unimplemented!("parse punctuator")
                }
                Category::Operator => {
                    unimplemented!("parse operator")
                }
                Category::Type => {
                    unimplemented!("parse type")
                }
                Category::Invalid => Ast::Invalid {
                    token: self.advance().expect("unreachable").clone(),
                },
            };

            if let Ast::Program { statements } = &mut self.program {
                statements.push(ast);
            } else {
                panic!("top level program must be a statements vector")
            }
        }

        Ok(Some(self.program.clone()))
    }

    fn parse_expr(&mut self, _min_bp: u8) -> Expr {
        let token = self.advance().expect("unreachable");
        let lhs = match token.category {
            Category::Identifier => Expr::Identifier {
                identifier: token.value,
                span: token.span,
                kind: Kind::Identifier(ConstituentIdentifier::Variable),
            },
            Category::Literal => match token.value.chars().next() {
                Some('\'') => Expr::CharacterLiteral {
                    literal: token.value,
                    span: token.span,
                    kind: Kind::Literal(ConstituentLiteral::Char),
                },
                Some('"') => Expr::StringLiteral {
                    literal: token.value,
                    span: token.span,
                    kind: Kind::Literal(ConstituentLiteral::String),
                },
                Some('t' | 'f') => Expr::BooleanLiteral {
                    keyword: token.value,
                    span: token.span,
                    kind: Kind::Literal(ConstituentLiteral::Boolean),
                },
                Some('n') => Expr::NullLiteral {
                    span: token.span,
                    kind: Kind::Literal(ConstituentLiteral::Null),
                },
                Some(ch) if ch.is_ascii_digit() => Expr::NumericLiteral {
                    literal: token.value,
                    span: token.span,
                    kind: Kind::Literal(ConstituentLiteral::Numeric),
                },
                _ => panic!("unrecognized literal"),
            },
            _ => unimplemented!("unrecognized atomic"),
        };

        // let mut lhs = match self.advance() {
        //     _ => { unimplemented!(""); },
        // };
        //
        // Expr::Identifier {
        //
        // }

        lhs
    }

    fn parse_newline(&mut self) {
        loop {
            let token = match self.peek() {
                Some(token) => token,
                None => return,
            };

            match (token.category, token.value.as_str()) {
                (Category::Punctuator, "\n") => {
                    self.advance();
                }
                _ => return,
            }
        }
    }

    fn parse_scope(&mut self) {
        let token = match self.peek() {
            Some(token) => token,
            None => return,
        };

        match (token.category, token.value.as_str()) {
            (Category::Punctuator, "}") => {
                self.advance();
                self.symbol_table.exit_scope();
            }

            (Category::Punctuator, "{") => {
                self.advance();
                self.symbol_table.enter_scope();
            }

            _ => {}
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

    // #[inline]
    // fn check(&self, value: &str) -> bool {
    //     self.peek().map(|t| t.value == value).unwrap_or(false)
    // }
    //
    // #[inline]
    // fn expect(&mut self, value: &str) -> Result<Token, RatError> {
    //     match self.advance() {
    //         Some(token) if token.value == value => Ok(token),
    //
    //         Some(token) => self.parse_error(
    //             ParseError::ExpectedGot {
    //                 expected: String::from(value),
    //                 actual: token.value.clone(),
    //             },
    //             token.value,
    //             token.span,
    //         ),
    //
    //         None => self.parse_error(
    //             ParseError::ExpectedGotEof {
    //                 expected: String::from(value),
    //             },
    //             value,
    //             Span::new(),
    //         ),
    //     }
    // }
    //
    // #[inline]
    // fn parse_error<T>(
    //     &mut self,
    //     err: ParseError,
    //     value: impl Into<String>,
    //     span: Span,
    // ) -> Result<T, RatError> {
    //     Err(RatError::parse(err, value.into(), span))
    // }
}
