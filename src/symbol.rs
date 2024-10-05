use std::{collections::HashMap, fmt::Display};

// Scope is an enum that represents the scope of a symbol.
// It has four variants: Global, Local, Builtin, and Free.
// It implements the Clone, Debug, and PartialEq traits.
#[derive(Clone, Debug, PartialEq)]
pub enum Scope {
    Global,
    Local,
    Builtin,
    Free,
    Function,
}

impl Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scope::Global => write!(f, "global"),
            Scope::Local => write!(f, "local"),
            Scope::Builtin => write!(f, "builtin"),
            Scope::Free => write!(f, "free"),
            Scope::Function => write!(f, "function"),
        }
    }
}

// Symbol is a struct that holds the name, scope, and index of a symbol.
// It has a new function that creates a new Symbol with the given name, scope, and index.
// It also implements the Clone, Debug, and PartialEq traits.
#[derive(Clone, Debug, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub scope: Scope,
    pub index: usize,
}

impl Symbol {
    pub fn new(name: String, scope: Scope, index: usize) -> Self {
        Symbol { name, scope, index }
    }
}

// SymbolTable is a struct that holds a HashMap of symbols and a reference to an outer SymbolTable.
// It also has a num_definitions field that keeps track of the number of symbols defined in the table.
#[derive(Clone, Debug)]
pub struct SymbolTable {
    pub outer: Option<Box<SymbolTable>>,
    pub store: HashMap<String, Symbol>,
    pub num_definitions: usize,
    free_symbols: Vec<Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            outer: None,
            store: HashMap::new(),
            num_definitions: 0,
            free_symbols: vec![],
        }
    }

    pub fn new_enclosed(outer: SymbolTable) -> Self {
        SymbolTable {
            outer: Some(Box::new(outer)),
            store: HashMap::new(),
            num_definitions: 0,
            free_symbols: vec![],
        }
    }

    pub fn define(&mut self, name: String) -> Symbol {
        let mut scope = Scope::Local;
        if self.outer.is_none() {
            scope = Scope::Global;
        }

        let symbol = Symbol::new(name.clone(), scope, self.num_definitions);
        self.store.insert(name, symbol.clone());
        self.num_definitions += 1;
        symbol
    }

    pub fn define_builtin(&mut self, index: usize, name: String) -> Symbol {
        let symbol = Symbol::new(name.clone(), Scope::Builtin, index);
        self.store.insert(name, symbol.clone());
        symbol
    }

    pub fn define_free(&mut self, original: Symbol) -> Symbol {
        self.free_symbols.push(original.clone());
        let symbol = Symbol::new(
            original.name.clone(),
            Scope::Free,
            self.free_symbols.len() - 1,
        );
        self.store.insert(original.name.clone(), symbol.clone());
        symbol
    }

    pub fn define_function_name(&mut self, name: String) -> Symbol {
        let symbol = Symbol::new(name.clone(), Scope::Function, 0);
        self.store.insert(name, symbol.clone());
        symbol
    }

    pub fn resolve(&mut self, name: &str) -> Option<Symbol> {
        match self.store.get(name) {
            Some(symbol) => Some(symbol.clone()),
            None => match &self.outer {
                Some(outer) => {
                    let symbol = outer.clone().resolve(name);
                    match symbol {
                        Some(s) => {
                            if s.scope == Scope::Global || s.scope == Scope::Builtin {
                                Some(s)
                            } else {
                                let free = self.define_free(s);
                                Some(free)
                            }
                        }
                        None => None,
                    }
                }
                None => None,
            },
        }
    }
}
