use crate::compiler::{Scope, Symbol};

pub struct SymbolTable {
    stack: Vec<Scope>,
}

impl SymbolTable {
    pub fn init() -> Self {
        let scope = Scope::init();
        let vec = Vec::new();

        let mut symbol_table = SymbolTable { stack: vec };
        symbol_table.stack.push(scope);

        symbol_table
    }

    pub fn enter_scope(&mut self) {
        self.stack.push(Scope::init());
    }

    pub fn exit_scope(&mut self) {
        self.stack.pop();

        #[cfg(debug_assertions)]
        assert!(!self.stack.is_empty(), "cannot exit global scope")
    }

    pub fn contains(&self, key: &str) -> bool {
        self.stack.iter().rev().any(|scope| scope.contains(key))
    }

    pub fn lookup(&self, key: &str) -> Option<&Symbol> {
        self.stack.iter().rev().find_map(|scope| scope.lookup(key))
    }

    pub fn insert_symbol(&mut self, symbol: Symbol) {
        match self.stack.last_mut() {
            Some(scope) => {
                scope.insert_symbol(symbol);
            }
            None => {
                panic!("symbol table must always contain global scope")
            }
        }
    }
}
