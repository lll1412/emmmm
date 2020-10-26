use crate::compiler::RcSymbolTable;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum SymbolScope {
    GLOBAL,
    LOCAL,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Symbol {
    pub name: String,
    pub scope: SymbolScope,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub outer: Option<Rc<RefCell<SymbolTable>>>,
    pub store: HashMap<String, Symbol>,
    pub num_definitions: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            outer: None,
            store: Default::default(),
            num_definitions: 0,
        }
    }
    pub fn define(&mut self, name: &str) -> Symbol {
        let scope = if self.outer.is_none() {
            SymbolScope::GLOBAL
        } else {
            SymbolScope::LOCAL
        };
        let symbol = Symbol {
            name: String::from(name),
            scope,
            index: self.num_definitions,
        };
        self.store.insert(name.to_string(), symbol.clone());
        self.num_definitions += 1;
        symbol
    }

    pub fn resolve(&self, name: &str) -> Option<Symbol> {
        self.store.get(name).cloned().or_else(|| {
            self.outer
                .clone()
                .and_then(|s| s.as_ref().borrow().resolve(name))
        })
    }

    pub fn new_enclosed(global: RcSymbolTable) -> RcSymbolTable {
        let num_definitions = 0;
        let outer = Some(global);
        let store = Default::default();
        Rc::new(RefCell::new(Self {
            outer,
            store,
            num_definitions,
        }))
    }
}
