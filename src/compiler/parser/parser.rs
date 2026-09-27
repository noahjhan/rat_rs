use crate::compiler::{Category, Expr, Kind, OperatorKind, Program, RatError, Stmt, Token};

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
        let mut lhs = self.recurse_comparative()?;

        loop {
            let token = match self.peek() {
                Some(token) => token.clone(),
                None => break,
            };

            let op = match token.value.as_str() {
                "&&" => OperatorKind::And,
                "||" => OperatorKind::Or,
                _ => break,
            };

            self.advance();

            let rhs = self.recurse_comparative()?;

            lhs = Expr::BinaryExpr {
                span: token.span,
                op: Kind::Operator(op),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }

        Some(lhs)
    }

    fn recurse_comparative(&mut self) -> Option<Expr> {
        let mut lhs = self.recurse_shift()?;

        loop {
            let token = match self.peek() {
                Some(token) => token.clone(),
                None => break,
            };

            let op = match token.value.as_str() {
                "==" => OperatorKind::Eq,
                "!=" => OperatorKind::Neq,
                "<" => OperatorKind::Lt,
                ">" => OperatorKind::Gt,
                "<=" => OperatorKind::Lte,
                ">=" => OperatorKind::Gte,
                _ => break,
            };

            self.advance();

            let rhs = self.recurse_shift()?;

            lhs = Expr::BinaryExpr {
                span: token.span,
                op: Kind::Operator(op),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }

        Some(lhs)
    }

    fn recurse_shift(&mut self) -> Option<Expr> {
        let mut lhs = self.recurse_additive()?;

        loop {
            let token = match self.peek() {
                Some(token) => token.clone(),
                None => break,
            };

            let op = match token.value.as_str() {
                "<<" => OperatorKind::Shl,
                ">>" => OperatorKind::Shr,
                _ => break,
            };

            self.advance();

            let rhs = self.recurse_additive()?;

            lhs = Expr::BinaryExpr {
                span: token.span,
                op: Kind::Operator(op),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }

        Some(lhs)
    }

    fn recurse_additive(&mut self) -> Option<Expr> {
        let mut lhs = self.recurse_multiplicative()?;

        loop {
            let token = match self.peek() {
                Some(token) => token.clone(),
                None => break,
            };

            let op = match token.value.as_str() {
                "+" => OperatorKind::Add,
                "-" => OperatorKind::Sub,
                _ => break,
            };

            self.advance();

            let rhs = self.recurse_multiplicative()?;

            lhs = Expr::BinaryExpr {
                span: token.span,
                op: Kind::Operator(op),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }

        Some(lhs)
    }

    fn recurse_multiplicative(&mut self) -> Option<Expr> {
        let mut lhs = self.recurse_unary()?;

        loop {
            let token = match self.peek() {
                Some(token) => token.clone(),
                None => break,
            };

            let op = match token.value.as_str() {
                "*" => OperatorKind::Mul,
                "/" => OperatorKind::Div,
                "%" => OperatorKind::Mod,
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
            "!" => OperatorKind::Not,
            "~" => OperatorKind::BitNeg,
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
                    panic!("recurse grouping: unmatched ')'");
                }

                self.advance();
                expr
            }
            _ => self.recurse_primitive(),
        }
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
