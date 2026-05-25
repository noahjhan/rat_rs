use crate::compiler::Symbol;

use std::collections::HashMap;

pub struct Scope {
    symbols: HashMap<String, Vec<Symbol>>,
}

impl Scope {
    pub fn init() -> Self {
        Scope {
            symbols: HashMap::new(),
        }
    }

    pub fn insert_symbol(&mut self, symbol: Symbol) {
        let value = symbol.token.value.clone();

        match self.symbols.get_mut(&value) {
            Some(vec) => vec.push(symbol.clone()),
            None => {
                let mut vec = Vec::new();
                vec.push(symbol.clone());
                self.symbols.insert(value, vec);
            }
        }
    }
}
