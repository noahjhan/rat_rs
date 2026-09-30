pub mod ast;
pub mod compile;
pub mod diagnostics;
pub mod lexer;
pub mod parser;
pub mod source;
pub mod symbol;
pub mod token;

pub use compile::compile;

pub use ast::{Expr, Parameter, Program, Stmt};

pub use diagnostics::{
    ErrorKind, LexicalError, ParseError, Position, RatError, Render, SemanticError, Span,
};

pub use lexer::{Lexer, Recovery};

pub use token::{
    Category, IdentifierKind, KeywordKind, Kind, LiteralKind, OperatorKind, PunctuatorKind, Token,
    TypeKind,
};

pub use parser::Parser;

pub use symbol::{Symbol, SymbolTable};

pub use source::RatSource;

pub use source::context::ReadContext;
