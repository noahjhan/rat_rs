use crate::compiler::{Kind, Span, Token};

#[derive(Debug, Clone)]
pub enum Ast {
    Program {
        statements: Vec<Ast>,
    },

    VariableDecl {
        identifier: String,
        span: Span,
        kind: Kind,
        expr: Expr,
    },

    FunctionDecl {
        identifier: String,
        span: Span,
        kind: Kind,
        parameters: Vec<Parameter>,
        return_type: Token,
        body: Box<Ast>,
    },

    ConditionalStatement {
        /// TODO: replace with enum
        keyword: String,
        span: Span,
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
    pub token: Token,
    pub param_type: Token,
}

#[derive(Debug, Clone)]
pub enum Expr {
    BinaryExpr {
        lhs: Box<Expr>,
        /// TODO: replace with enum
        op: String,
        rhs: Box<Expr>,
        span: Span,
        kind: Kind,
    },

    UnaryExpr {
        expr: Box<Expr>,
        /// TODO: replace with enum
        op: String,
        span: Span,
        kind: Kind,
    },

    NumericLiteral {
        literal: String,
        span: Span,
        kind: Kind,
    },

    StringLiteral {
        literal: String,
        span: Span,
        kind: Kind,
    },

    CharacterLiteral {
        literal: String,
        span: Span,
        kind: Kind,
    },

    BooleanLiteral {
        /// TODO: replace with enum
        keyword: String,
        span: Span,
        kind: Kind,
    },

    NullLiteral {
        span: Span,
        kind: Kind,
    },

    Identifier {
        identifier: String,
        span: Span,
        kind: Kind,
    },

    FunctionCall {
        identifier: String,
        span: Span,
        kind: Kind,
        parameters: Vec<Expr>,
    },
}
