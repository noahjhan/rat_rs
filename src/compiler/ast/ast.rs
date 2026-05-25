use crate::compiler::{Kind, Token};

#[derive(Debug, Clone)]
pub enum Ast {
    Program {
        statements: Vec<Ast>,
    },

    VariableDecl {
        identifier: Token,
        kind: Kind,
        expr: Expr,
    },

    FunctionDecl {
        identifier: Token,
        kind: Kind,
        parameters: Vec<Parameter>,
        return_type: Token,
        body: Box<Ast>,
    },

    ConditionalStatement {
        keyword: Token,
        kind: Kind,
        expr: Expr,
        next: Option<Box<Ast>>,
        body: Box<Ast>,
    },

    Invalid {
        token: Token,
    },
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub identifier: Token,
    pub param_type: Token,
}

#[derive(Debug, Clone)]
pub enum Expr {
    BinaryExpr {
        lhs: Box<Expr>,
        op: Token,
        rhs: Box<Expr>,
        kind: Kind,
    },

    UnaryExpr {
        expr: Box<Expr>,
        op: Token,
        kind: Kind,
    },

    NumericLiteral {
        value: Token,
        kind: Kind,
    },

    StringLiteral {
        value: Token,
        kind: Kind,
    },

    Identifier {
        value: Token,
        kind: Kind,
    },

    FunctionCall {
        identifier: Token,
        kind: Kind,
        parameters: Vec<Expr>,
    },
}
