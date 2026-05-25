use crate::compiler::{Kind, Token};

#[derive(Clone)]
pub struct Symbol {
    pub token: Token,
    pub kind: Kind,
}

impl Symbol {
    pub fn new(token: Token, kind: Kind) -> Self {
        Symbol { token, kind }
    }
}
