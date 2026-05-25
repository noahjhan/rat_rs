use crate::compiler::{Kind, Token};

#[derive(Clone)]
pub struct Symbol {
    pub token: Token,
    pub kind: Kind,
}
