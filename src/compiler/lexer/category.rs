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

const KEYWORDS: &[&str] = &[
    "let", "op", "class", "fn", "fn_", "fn?", "fn\\", "ret", "rev", "if", "else", "match", "main",
];

const OPERATORS: &[&str] = &[
    "=", "+", "-", "*", "/", "%", "==", "!=", "<", ">", "<=", ">=", "&&", "||", "!", "&", "|", "^",
    "~", "<<", ">>", "->", "=>", "::", ".",
];

const PUNCTUATORS: &[&str] = &[":", ",", "[", "]", "{", "}", "(", ")", "\n"];

const TYPES: &[&str] = &[
    "int", "float", "double", "bool", "char", "long", "short", "pointer", "uint", "ulong",
    "ushort", "uchar", "string", "void",
];

const DELIMITERS: &[char] = &[
    '"', '\'', ':', ',', '[', ']', '{', '}', '(', ')', '\n', '=', '+', '-', '*', '/', '%', '<',
    '>', '&', '|', '!', '^', '~', '.',
];

impl Category {
    pub fn is_any(s: &str) -> Option<Self> {
        Self::is_keyword_str(s)
            .or_else(|| Self::is_operator_str(s))
            .or_else(|| Self::is_punctuator_str(s))
            .or_else(|| Self::is_type_str(s))
    }

    fn is_keyword_str(s: &str) -> Option<Self> {
        KEYWORDS.contains(&s).then_some(Category::Keyword)
    }

    fn is_operator_str(s: &str) -> Option<Self> {
        OPERATORS.contains(&s).then_some(Category::Operator)
    }

    fn is_punctuator_str(s: &str) -> Option<Self> {
        PUNCTUATORS.contains(&s).then_some(Category::Punctuator)
    }

    fn is_type_str(s: &str) -> Option<Self> {
        TYPES.contains(&s).then_some(Category::Type)
    }

    pub fn is_delimiter(ch: char) -> bool {
        DELIMITERS.contains(&ch) || ch.is_whitespace() || !ch.is_ascii()
    }
}
