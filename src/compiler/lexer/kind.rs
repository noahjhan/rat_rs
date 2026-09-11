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

// impl Kind {
//     pub fn is_identifier(&self) -> bool {
//         matches!(self, Kind::Identifier(_))
//     }
//     pub fn is_keyword(&self) -> bool {
//         matches!(self, Kind::Keyword(_))
//     }
//     pub fn is_literal(&self) -> bool {
//         matches!(self, Kind::Literal(_))
//     }
//     pub fn is_punctuator(&self) -> bool {
//         matches!(self, Kind::Punctuator(_))
//     }
//     pub fn is_operator(&self) -> bool {
//         matches!(self, Kind::Operator(_))
//     }
//     pub fn is_type(&self) -> bool {
//         matches!(self, Kind::Type(_))
//     }
//     pub fn is_invalid(&self) -> bool {
//         matches!(self, Kind::Invalid)
//     }
// }

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
