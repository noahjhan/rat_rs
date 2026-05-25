use crate::compiler::Token;
use std::collections::HashMap;

pub struct Scope {
    symbols: HashMap<String, Vec<Token>>,
}

impl Scope {
    pub fn init() -> Self {
        Scope {
            symbols: HashMap::new(),
        }
    }

    pub fn insert_symbol(&mut self, token: Token) {
        match self.symbols.get_mut(&token.value) {
            Some(vec) => vec.push(token.clone()),
            None => {
                let mut vec = Vec::new();
                vec.push(token.clone());
                self.symbols.insert(token.value.clone(), vec);
            }
        }
    }
}
