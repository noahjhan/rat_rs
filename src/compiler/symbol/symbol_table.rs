use crate::compiler::{RatError, SemanticError, Symbol};
use std::collections::HashMap;

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
        children: Vec<SymbolTable>,
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

        if globals.contains_key(&symbol.identifier) {
            return Err(RatError::semantic(
                SemanticError::GlobalRedeclaration,
                symbol.identifier,
                symbol.span,
            ));
        }

        globals.insert(symbol.identifier.clone(), symbol);

        Ok(())
    }

    pub fn insert_function_declaration(&mut self, symbol: Symbol) -> Result<(), RatError> {
        let SymbolTable::ProgramTable { functions, .. } = self else {
            panic!("insert_function_declaration called on non-program symbol table");
        };

        if functions.contains_key(&symbol.identifier) {
            return Err(RatError::semantic(
                SemanticError::FunctionRedeclaration,
                symbol.identifier,
                symbol.span,
            ));
        }

        let function = SymbolTable::FunctionTable {
            parameters: HashMap::new(),
            body: Box::new(SymbolTable::Scope {
                children: Vec::new(),
                symbols: HashMap::new(),
            }),
        };

        functions.insert(symbol.identifier, function);

        Ok(())
    }

    pub fn add_parameter(&mut self, fn_identifier: String, symbol: Symbol) -> Result<(), RatError> {
        let SymbolTable::ProgramTable { functions, .. } = self else {
            panic!("add_parameter called on non-program symbol table");
        };

        let Some(SymbolTable::FunctionTable { parameters, .. }) = functions.get_mut(&fn_identifier)
        else {
            panic!("invalid function table instantiation")
        };

        if parameters.contains_key(&symbol.identifier) {
            return Err(RatError::semantic(
                SemanticError::ParameterRedeclaration,
                symbol.identifier,
                symbol.span,
            ));
        }

        parameters.insert(symbol.identifier.clone(), symbol);

        Ok(())
    }
}
