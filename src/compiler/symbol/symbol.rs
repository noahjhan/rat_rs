use crate::compiler::{Kind, Span};

pub struct Symbol {
    pub identifier: String,
    pub span: Span, // span of decl
    pub kind: Kind,
}

impl Symbol {
    pub fn new(identifier: String, span: Span, kind: Kind) -> Self {
        Symbol {
            identifier,
            span,
            kind,
        }
    }
}
