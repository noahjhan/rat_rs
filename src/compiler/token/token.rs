use crate::compiler::{Category, Span};

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
