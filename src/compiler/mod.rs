pub mod ast;
pub mod compile;
pub mod diagnostics;
pub mod lexer;
pub mod parser;
pub mod semantics;
pub mod source;

pub use ast::{Expr, Parameter, Program, Stmt};
pub use compile::compile;

pub use diagnostics::{ErrorKind, LexicalError, ParseError, Position, RatError, Render, Span};

pub use lexer::{
    Category, IdentifierKind, KeywordKind, Kind, Lexer, LiteralKind, OperatorKind, PunctuatorKind,
    Recovery, Token, TypeKind,
};

pub use parser::Parser;

pub use semantics::{Scope, Symbol, SymbolTable};

pub use source::RatSource;
pub use source::context::ReadContext;
