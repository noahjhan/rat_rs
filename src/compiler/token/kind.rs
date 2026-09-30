#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Kind {
    Identifier(IdentifierKind),
    Keyword(KeywordKind),
    Literal(LiteralKind),
    Punctuator(PunctuatorKind),
    Operator(OperatorKind),
    Type(TypeKind),
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentifierKind {
    Variable,
    Function,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeywordKind {
    Let,
    LetOptional,
    Function,
    FunctionVoid,
    FunctionError,
    FunctionLambda,
    Return,
    ReturnVoid,
    If,
    Else,
    Match,
    Main,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LiteralKind {
    Numeric,
    String,
    Boolean,
    Char,
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PunctuatorKind {
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
pub enum OperatorKind {
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
pub enum TypeKind {
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
