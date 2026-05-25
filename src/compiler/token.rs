use crate::compiler::Span;

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

impl Category {
    pub fn is_delimiter(ch: char) -> bool {
        if ch == '"' || ch == '\'' {
            return true;
        }
        if Self::is_punctuator_str(&String::from(ch)).is_some()
            || Self::is_operator_str(&String::from(ch)).is_some()
            || ch.is_whitespace()
        {
            return true;
        }
        !ch.is_ascii()
    }

    pub fn is_any(s: &str) -> Option<Self> {
        Self::is_keyword_str(s)
            .or_else(|| Self::is_operator_str(s))
            .or_else(|| Self::is_punctuator_str(s))
            .or_else(|| Self::is_type_str(s))
    }

    fn is_operator_str(s: &str) -> Option<Self> {
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
                | "::"
                | "."
        )
        .then_some(Category::Operator)
    }

    fn is_keyword_str(s: &str) -> Option<Self> {
        matches!(
            s,
            "let"
                | "op"
                | "fn"
                | "fn_"
                | "fn?"
                | "fn\\"
                | "ret"
                | "rev"
                | "if"
                | "else"
                | "match"
                | "main"
        )
        .then_some(Category::Keyword)
    }

    fn is_punctuator_str(s: &str) -> Option<Self> {
        matches!(s, ":" | "," | "[" | "]" | "{" | "}" | "(" | ")" | "\n")
            .then_some(Category::Punctuator)
    }

    fn is_type_str(s: &str) -> Option<Self> {
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
    ModuleAccess,
    DotAccess,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub category: Category,
    pub value: String,
    pub span: Span,
}

impl Token {
    pub fn new(category: Category, value: String, span: Span) -> Self {
        Token {
            category,
            value,
            span,
        }
    }

    pub fn debug_print(&self) {
        println!("category: {:?}", self.category);
        println!("value:    {:?}", self.value);
        println!("line:     {:?}", self.span.start_pos.line);
        println!("col:      {:?}\n", self.span.start_pos.col);
    }
}
