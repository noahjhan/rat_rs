pub mod ast;
pub mod compile;
pub mod diagnostics;
pub mod lexer;
pub mod parser;
pub mod semantics;
pub mod source;

pub use ast::{generate_ast_primative, Expr, Parameter, Program, Stmt};
pub use compile::compile;

pub use diagnostics::{ErrorKind, LexicalError, ParseError, Position, RatError, Render, Span};

pub use lexer::{
    Category, ConstituentIdentifier, ConstituentKeyword, ConstituentLiteral, ConstituentOperator,
    ConstituentPunctuator, ConstituentType, Kind, Lexer, Recovery, Token,
};

pub use parser::Parser;

pub use semantics::{Scope, Symbol, SymbolTable};

pub use source::context::ReadContext;
pub use source::RatSource;
