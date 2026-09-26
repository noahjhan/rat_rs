use crate::compiler::{Kind, Span, Token};

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    VariableDecl {
        identifier: String,
        span: Span,
        expr: Expr,
    },

    FunctionDecl {
        identifier: Token,
        kind: Kind,
        parameters: Vec<Parameter>,
        return_type: Token,
        body: Program,
    },

    ConditionalStatement {
        keyword: Token,
        kind: Kind,
        expr: Expr,
        next: Box<Stmt>,
        body: Program,
    },

    ExprStmt(Expr),

    Invalid {
        token: Token,
    },

    Unimplemented {},
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub identifier: Token,
    pub param_type: Token,
}

#[derive(Debug, Clone)]
pub enum Expr {
    BinaryExpr {
        span: Span,
        op: Kind,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    UnaryExpr {
        span: Span,
        op: Kind,
        expr: Box<Expr>,
    },

    Identifier {
        identifier: String,
        span: Span,
    },

    Literal {
        value: String,
        span: Span,
        // kind: Kind,
    },

    FunctionCall {
        identifier: String,
        span: Span,
        parameters: Vec<Expr>,
    },

    Invalid {
        token: Token,
    },
}
