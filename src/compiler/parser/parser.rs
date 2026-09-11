use crate::compiler::{Category, ConstituentOperator, Expr, Kind, Program, RatError, Stmt, Token};

use std::collections::VecDeque;

pub struct Parser {
    tokens: VecDeque<Token>,
    program: Program,
}

impl Parser {
    pub fn init(tokens: VecDeque<Token>) -> Self {
        Parser {
            tokens,
            program: Program {
                statements: Vec::new(),
            },
        }
    }

    pub fn dispatch(&mut self) -> Result<Option<Program>, RatError> {
        if self.tokens.is_empty() {
            return Ok(None);
        }

        loop {
            let token = match self.peek() {
                Some(t) => t,
                None => break,
            };

            let stmt = match token.category {
                Category::Literal | Category::Identifier | Category::Operator => {
                    Stmt::ExprStmt(self.recurse_expr().unwrap())
                } // fix unwrap

                // Category::Identifier => Stmt::ExprStmt(self.generate_ast_primative(token.clone())),
                // Category::Literal => Stmt::ExprStmt(self.generate_ast_primative(token.clone())),
                Category::Invalid => Stmt::Invalid {
                    token: token.clone(),
                },
                _ => Stmt::Unimplemented {},
            };

            self.program.statements.push(stmt);
            self.advance();
        }

        Ok(Some(self.program.clone()))
    }

    fn recurse_expr(&mut self) -> Option<Expr> {
        self.recurse_logical()
    }

    fn recurse_logical(&mut self) -> Option<Expr> {
        self.recurse_comparative()
    }

    fn recurse_comparative(&mut self) -> Option<Expr> {
        self.recurse_shift()
    }

    fn recurse_shift(&mut self) -> Option<Expr> {
        self.recurse_additive()
    }

    fn recurse_additive(&mut self) -> Option<Expr> {
        self.recurse_multiplicative()
    }

    fn recurse_multiplicative(&mut self) -> Option<Expr> {
        let mut lhs = self.recurse_unary()?;

        loop {
            let token = match self.peek() {
                Some(token) => token.clone(),
                None => break,
            };

            let op = match token.value.as_str() {
                "*" => ConstituentOperator::Mul,
                "/" => ConstituentOperator::Div,
                "%" => ConstituentOperator::Mod,
                _ => break,
            };

            self.advance();

            let rhs = self.recurse_unary()?;

            lhs = Expr::BinaryExpr {
                span: token.span,
                op: Kind::Operator(op),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }

        Some(lhs)
    }

    fn recurse_unary(&mut self) -> Option<Expr> {
        let token = match self.peek() {
            Some(token) => token.clone(),
            None => return None,
        };

        let op = match token.value.as_str() {
            "!" => ConstituentOperator::Not,
            "~" => ConstituentOperator::BitNeg,
            _ => return self.recurse_grouping(),
        };

        self.advance();

        Some(Expr::UnaryExpr {
            span: token.span,
            op: Kind::Operator(op),
            expr: Box::new(self.recurse_unary().unwrap()),
        })
    }

    fn recurse_grouping(&mut self) -> Option<Expr> {
        let token = self.peek()?.clone();
        match token.value.as_str() {
            "(" => {
                self.advance();
                let expr = self.recurse_expr();

                if !self.check(")") {
                    panic!("handle error here");
                }

                self.advance();
                return expr;
            }
            _ => {}
        };

        return self.recurse_primitive();
    }

    fn recurse_primitive(&mut self) -> Option<Expr> {
        let token = match self.peek() {
            Some(token) => self.generate_ast_primative(token.clone()),
            None => return None,
        };

        self.advance();
        Some(token)
    }

    fn generate_ast_primative(&self, token: Token) -> Expr {
        match token.category {
            Category::Identifier => Expr::Identifier {
                identifier: token.value,
                span: token.span,
            },
            Category::Literal => Expr::Literal {
                value: token.value,
                span: token.span,
            },
            _ => Expr::Invalid {
                token: token.clone(),
            },
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
}

// use crate::compiler::{
//     Ast, Category, ConstituentIdentifier, ConstituentLiteral, Expr, Kind, Program, RatError,
//     SymbolTable, Token,
// };
// use std::collections::VecDeque;
//
// pub struct Parser {
//     tokens: VecDeque<Token>,
//     symbol_table: SymbolTable,
//     program: Program,
// }
//
// impl Parser {
//     pub fn init(tokens: VecDeque<Token>) -> Self {
//         Parser {
//             tokens,
//             symbol_table: SymbolTable::init(),
//             program: Program {
//                 statements: Vec::new(),
//             },
//         }
//     }
//
//     pub fn dispatch(&mut self) -> Result<Option<Program>, RatError> {
//         if self.tokens.is_empty() {
//             return Ok(None);
//         }
//
//         loop {
//             self.parse_newline();
//             self.parse_scope();
//
//             let mut a = 10;
//             print!("{}", a);
//             let b = a = 5;
//             print!("{}", a);
//             print!("{:?}", b);
//
//             let token = match self.peek() {
//                 Some(t) => t,
//                 None => break,
//             };
//
//             let ast = match token.category {
//                 Category::Identifier => {
//                     unimplemented!("parse identifier")
//                 }
//                 Category::Keyword => {
//                     unimplemented!("parse keyword")
//                 }
//                 Category::Literal => {
//                     unimplemented!("parse literal")
//                 }
//                 Category::Punctuator => {
//                     unimplemented!("parse punctuator")
//                 }
//                 Category::Operator => {
//                     unimplemented!("parse operator")
//                 }
//                 Category::Type => {
//                     unimplemented!("parse type")
//                 }
//                 Category::Invalid => Ast::Invalid {
//                     token: self.advance().expect("unreachable").clone(),
//                 },
//             };
//
//             self.program.statements.push(ast);
//         }
//         return Ok(None);
//     }
//
//     fn parse_expr(&mut self, _min_bp: u8) -> Expr {
//         let token = self.advance().expect("unreachable");
//         let lhs = match token.category {
//             Category::Identifier => Expr::Identifier {
//                 identifier: token.value,
//                 span: token.span,
//                 kind: Kind::Identifier(ConstituentIdentifier::Variable),
//             },
//             Category::Literal => match token.value.chars().next() {
//                 Some('\'') => Expr::CharacterLiteral {
//                     literal: token.value,
//                     span: token.span,
//                     kind: Kind::Literal(ConstituentLiteral::Char),
//                 },
//                 Some('"') => Expr::StringLiteral {
//                     literal: token.value,
//                     span: token.span,
//                     kind: Kind::Literal(ConstituentLiteral::String),
//                 },
//                 Some('t' | 'f') => Expr::BooleanLiteral {
//                     keyword: token.value,
//                     span: token.span,
//                     kind: Kind::Literal(ConstituentLiteral::Boolean),
//                 },
//                 Some('n') => Expr::NullLiteral {
//                     span: token.span,
//                     kind: Kind::Literal(ConstituentLiteral::Null),
//                 },
//                 Some(ch) if ch.is_ascii_digit() => Expr::NumericLiteral {
//                     literal: token.value,
//                     span: token.span,
//                     kind: Kind::Literal(ConstituentLiteral::Numeric),
//                 },
//                 _ => panic!("unrecognized literal"),
//             },
//             _ => unimplemented!("unrecognized atomic"),
//         };
//
//         // let mut lhs = match self.advance() {
//         //     _ => { unimplemented!(""); },
//         // };
//         //
//         // Expr::Identifier {
//         //
//         // }
//
//         lhs
//     }
//
//     fn parse_newline(&mut self) {
//         loop {
//             let token = match self.peek() {
//                 Some(token) => token,
//                 None => return,
//             };
//
//             match (token.category, token.value.as_str()) {
//                 (Category::Punctuator, "\n") => {
//                     self.advance();
//                 }
//                 _ => return,
//             }
//         }
//     }
//
//     fn parse_scope(&mut self) {
//         let token = match self.peek() {
//             Some(token) => token,
//             None => return,
//         };
//
//         match (token.category, token.value.as_str()) {
//             (Category::Punctuator, "}") => {
//                 self.advance();
//                 self.symbol_table.exit_scope();
//             }
//
//             (Category::Punctuator, "{") => {
//                 self.advance();
//                 self.symbol_table.enter_scope();
//             }
//
//             _ => {}
//         }
//     }
//
//     #[inline]
//     fn peek(&self) -> Option<&Token> {
//         self.tokens.front()
//     }
//
//     #[inline]
//     fn advance(&mut self) -> Option<Token> {
//         self.tokens.pop_front()
//     }
//
//     // #[inline]
//     // fn check(&self, value: &str) -> bool {
//     //     self.peek().map(|t| t.value == value).unwrap_or(false)
//     // }
//     //
//     // #[inline]
//     // fn expect(&mut self, value: &str) -> Result<Token, RatError> {
//     //     match self.advance() {
//     //         Some(token) if token.value == value => Ok(token),
//     //
//     //         Some(token) => self.parse_error(
//     //             ParseError::ExpectedGot {
//     //                 expected: String::from(value),
//     //                 actual: token.value.clone(),
//     //             },
//     //             token.value,
//     //             token.span,
//     //         ),
//     //
//     //         None => self.parse_error(
//     //             ParseError::ExpectedGotEof {
//     //                 expected: String::from(value),
//     //             },
//     //             value,
//     //             Span::new(),
//     //         ),
//     //     }
//     // }
//     //
//     // #[inline]
//     // fn parse_error<T>(
//     //     &mut self,
//     //     err: ParseError,
//     //     value: impl Into<String>,
//     //     span: Span,
//     // ) -> Result<T, RatError> {
//     //     Err(RatError::parse(err, value.into(), span))
//     // }
// }
