use crate::compiler::Token;
use std::vec::Vec;

#[derive(Debug)]
pub enum Ast {
    Program {
        statements: Vec<Ast>,
    },

    VariableDecl {
        identifier: Token,
        expr: Expr,
    },

    FunctionDecl {
        identifier: Token,
        parameters: Vec<Parameter>,
        return_type: Token,
        body: Box<Ast>,
    },

    ConditionalStatement {
        keyword: Token,
        expr: Expr,
        next: Option<Box<Ast>>,
        body: Box<Ast>,
    },
}

#[derive(Debug)]
pub struct Parameter {
    pub identifier: Token,
    pub param_type: Token,
}

#[derive(Debug)]
pub enum Expr {
    BinaryExpr {
        lhs: Box<Expr>,
        op: Token,
        rhs: Box<Expr>,
    },

    UnaryExpr {
        expr: Box<Expr>,
        op: Token,
    },

    NumericLiteral {
        value: Token,
    },

    StringLiteral {
        value: Token,
    },

    Identifier {
        value: Token,
    },

    FunctionCall {
        identifier: Token,
        parameters: Vec<Expr>,
    },
}
