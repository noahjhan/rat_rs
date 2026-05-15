use crate::compiler::Position;

mod sealed {
    pub trait Sealed: std::fmt::Debug {}
    impl Sealed for super::Category {}
    impl Sealed for super::Kind {}
}

pub use sealed::Sealed;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    Identifier,
    Keyword,
    Literal,
    Punctuator,
    Operator,
    Type,
    Invalid,
}

// :  ->  identifier: type
// {} ->  around functions and initializer lists
// () ->  function calls & expressions
// .  ->  access but NOT the first in numeric literals
// ,  ->  between parameters
// \  ->  likely not

impl Category {
    pub fn is_delimiter(ch: char) -> bool {
        if ch == '"' || ch == '\'' {
            return true;
        }
        if Category::is_punctuator(&String::from(ch)).is_some()
            || Category::is_operator(&String::from(ch)).is_some()
            || ch.is_whitespace()
        {
            return true;
        } else if ch.is_ascii() {
            return false;
        }
        true
    }

    fn is_operator(s: &str) -> Option<Self> {
        matches!(
            s,
            "=" | "+"
                | "-"
                | "*"
                | "/"
                | "%"
                | "=="
                | "!="
                | "<"
                | ">"
                | "<="
                | ">="
                | "&&"
                | "||"
                | "!"
                | "&"
                | "|"
                | "^"
                | "~"
                | "<<"
                | ">>"
                | "->"
                | "=>"
        )
        .then_some(Category::Operator)
    }

    fn is_keyword(s: &str) -> Option<Self> {
        matches!(
            s,
            "let"
                | "op"
                | "fn"
                | "fn_"
                | "fn?"
                | "fn/"
                | "ret"
                | "rev"
                | "if"
                | "else"
                // | "else if"
                | "match"
                | "main"
        )
        .then_some(Category::Keyword)
    }

    fn is_punctuator(s: &str) -> Option<Self> {
        matches!(
            s,
            ":" | "," | "." | "[" | "]" | "{" | "}" | "(" | ")" | "//" | "/*" | "*/" | "\n"
        )
        .then_some(Category::Punctuator)
    }

    fn is_type(s: &str) -> Option<Self> {
        matches!(
            s,
            "int"
                | "float"
                | "double"
                | "bool"
                | "char"
                | "long"
                | "short"
                | "pointer"
                | "uint"
                | "ulong"
                | "ushort"
                | "uchar"
                | "string"
                | "void"
        )
        .then_some(Category::Type)
    }

    pub fn is_any(s: &str) -> Option<Self> {
        Self::is_keyword(s)
            .or_else(|| Self::is_operator(s))
            .or_else(|| Self::is_punctuator(s))
            .or_else(|| Self::is_type(s))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Kind {
    Identifier(ConstituentIdentifier),
    Keyword(ConstituentKeyword),
    Literal(ConstituentLiteral),
    Punctuator(ConstituentPunctuator),
    Operator(ConstituentOperator),
    Type(ConstituentType),
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConstituentIdentifier {
    Variable,
    Function,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConstituentKeyword {
    Let,
    LetOptional,
    Function,
    FunctionVoid,
    FunctionOptional,
    FunctionLambda,
    Return,
    ReturnVoid,
    If,
    Else,
    // ElseIf,
    Match,
    Main,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConstituentLiteral {
    Numeric,
    String,
    Boolean,
    Char,
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConstituentPunctuator {
    Colon,
    SingleQuote,
    DoubleQuote,
    Comma,
    BracketOpen,
    BracketClose,
    BraceOpen,
    BraceClose,
    ParenOpen,
    ParenClose,
    CommentLine,
    CommentBlockOpen,
    CommentBlockClose,
    Newline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConstituentOperator {
    Assign,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    And,
    Or,
    Not,
    BitAnd,
    BitOr,
    BitXor,
    BitNeg,
    Shl,
    Shr,
    Arrow,
    FatArrow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConstituentType {
    Int,
    Long,
    Short,
    Char,
    Float,
    Double,
    Bool,
    Uint,
    Ulong,
    Ushort,
    Uchar,
    String,
    Pointer,
    Void,
}

impl Kind {
    pub fn is_identifier(&self) -> bool {
        matches!(self, Kind::Identifier(_))
    }

    pub fn is_keyword(&self) -> bool {
        matches!(self, Kind::Keyword(_))
    }

    pub fn is_literal(&self) -> bool {
        matches!(self, Kind::Literal(_))
    }

    pub fn is_punctuator(&self) -> bool {
        matches!(self, Kind::Punctuator(_))
    }

    pub fn is_operator(&self) -> bool {
        matches!(self, Kind::Operator(_))
    }

    pub fn is_type(&self) -> bool {
        matches!(self, Kind::Type(_))
    }

    pub fn is_invalid(&self) -> bool {
        matches!(self, Kind::Invalid)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn set(start: Position, end: Position) -> Self {
        Span { start, end }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token<T: Sealed = Category> {
    pub kind: T,
    pub value: String,
    pub span: Span,
}

impl<T: Sealed> Token<T> {
    pub fn new(kind: T, value: String, span: Span) -> Self {
        Token { kind, value, span }
    }

    pub fn debug_print(&mut self) {
        println!("Kind:  {:?}", self.kind);
        println!("Value: {:?}\n", self.value);
    }
}
