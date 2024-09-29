use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub scope: String,
    pub index: usize,
}

impl Symbol {
    pub fn new(name: String, scope: String, index: usize) -> Self {
        Symbol { name, scope, index }
    }
}

pub struct SymbolTable {
    pub outer: Option<Box<SymbolTable>>,
    pub store: HashMap<String, Symbol>,
    pub num_definitions: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            outer: None,
            store: HashMap::new(),
            num_definitions: 0,
        }
    }

    pub fn define(&mut self, name: String) -> Symbol {
        let symbol = Symbol::new(name.clone(), self.scope_name(), self.num_definitions);
        self.store.insert(name, symbol.clone());
        self.num_definitions += 1;
        symbol
    }

    pub fn resolve(&self, name: &str) -> Option<Symbol> {
        match self.store.get(name) {
            Some(symbol) => Some(symbol.clone()),
            None => match &self.outer {
                Some(outer) => outer.resolve(name),
                None => None,
            },
        }
    }

    pub fn scope_name(&self) -> String {
        match &self.outer {
            Some(outer) => format!("{}{}", outer.scope_name(), self.num_definitions),
            None => "".to_string(),
        }
    }
}