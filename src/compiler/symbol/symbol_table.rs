use crate::compiler::{RatError, SemanticError, Symbol};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::atomic::{AtomicU64, Ordering};

static GLOBAL_ID: AtomicU64 = AtomicU64::new(0);

pub enum SymbolTable {
    ProgramTable {
        functions: HashMap<String, SymbolTable>,
        globals: HashMap<String, Symbol>,
    },

    FunctionTable {
        parameters: HashMap<String, Symbol>,
        body: Box<SymbolTable>,
    },

    Scope {
        id: u64,
        parent: Option<u64>,
        children: HashMap<u64, SymbolTable>,
        symbols: HashMap<String, Symbol>,
    },
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable::ProgramTable {
            functions: HashMap::new(),
            globals: HashMap::new(),
        }
    }

    pub fn insert_global(&mut self, symbol: Symbol) -> Result<(), RatError> {
        let SymbolTable::ProgramTable { globals, .. } = self else {
            panic!("insert_global called on non-program symbol table");
        };

        match globals.entry(symbol.identifier.clone()) {
            Entry::Occupied(_) => Err(RatError::new(
                SemanticError::GlobalRedeclaration(symbol.identifier),
                symbol.span,
            )),
            Entry::Vacant(slot) => {
                slot.insert(symbol);
                Ok(())
            }
        }
    }

    pub fn insert_function_declaration(&mut self, symbol: Symbol) -> Result<(), RatError> {
        let SymbolTable::ProgramTable { functions, .. } = self else {
            panic!("insert_function_declaration called on non-program symbol table");
        };

        match functions.entry(symbol.identifier.clone()) {
            Entry::Occupied(_) => Err(RatError::new(
                SemanticError::FunctionRedeclaration(symbol.identifier),
                symbol.span,
            )),
            Entry::Vacant(slot) => {
                slot.insert(SymbolTable::FunctionTable {
                    parameters: HashMap::new(),
                    body: Box::new(SymbolTable::Scope {
                        parent: None,
                        id: get_next_id(),
                        children: HashMap::new(),
                        symbols: HashMap::new(),
                    }),
                });
                Ok(())
            }
        }
    }

    pub fn add_parameter(&mut self, fn_identifier: String, symbol: Symbol) -> Result<(), RatError> {
        let SymbolTable::ProgramTable { functions, .. } = self else {
            panic!("add_parameter called on non-program symbol table");
        };

        let Some(SymbolTable::FunctionTable { parameters, .. }) = functions.get_mut(&fn_identifier)
        else {
            panic!("invalid function table instantiation")
        };

        match parameters.entry(symbol.identifier.clone()) {
            Entry::Occupied(_) => Err(RatError::new(
                SemanticError::ParameterRedeclaration(symbol.identifier),
                symbol.span,
            )),
            Entry::Vacant(slot) => {
                slot.insert(symbol);
                Ok(())
            }
        }
    }

    pub fn enter_scope(&mut self) {}
}

pub fn get_next_id() -> u64 {
    GLOBAL_ID.fetch_add(1, Ordering::Relaxed)
}
