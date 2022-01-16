use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::compiler::RcSymbolTable;
use crate::object::builtins::Builtin;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum SymbolScope {
    Global,
    Local,
    Builtin,
    Free,
    Function,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Symbol {
    pub name: String,
    pub scope: SymbolScope,
    pub index: usize,
}

#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    pub outer: Option<Rc<RefCell<SymbolTable>>>,
    pub store: HashMap<String, Rc<Symbol>>,
    pub num_definitions: usize,
    pub free_symbols: Vec<Rc<Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            outer: None,
            store: Default::default(),
            num_definitions: 0,
            free_symbols: vec![],
        }
    }
    pub fn define(&mut self, name: &str) -> Rc<Symbol> {
        let scope = if self.outer.is_none() {
            SymbolScope::Global
        } else {
            SymbolScope::Local
        };
        let symbol = Symbol {
            name: String::from(name),
            scope,
            index: self.num_definitions,
        };
        let symbol = Rc::new(symbol);
        self.store.insert(name.to_string(), symbol.clone());
        self.num_definitions += 1;
        symbol
    }
    fn define_free(&mut self, original: Rc<Symbol>) -> Rc<Symbol> {
        let name = original.name.clone();
        //加入自由变量表
        self.free_symbols.push(original);
        let symbol = Symbol {
            name: name.clone(),
            scope: SymbolScope::Free,
            index: self.free_symbols.len() - 1,
        };
        let symbol = Rc::new(symbol);
        //作用域改为free，加入符号表
        self.store.insert(name, symbol.clone());
        symbol
    }
    pub fn define_builtin(&mut self, index: usize, builtin: &Builtin) {
        let name = builtin.name.to_string();
        let symbol = Symbol {
            name: name.clone(),
            scope: SymbolScope::Builtin,
            index,
        };
        let symbol = Rc::new(symbol);
        self.store.insert(name, symbol);
    }
    pub fn define_self(&mut self, fun_name: Option<String>){
        let name = fun_name.unwrap_or_else(|| "this".to_string());
        let self_symbol = Rc::new(Symbol {
            name: name.clone(),
            scope: SymbolScope::Function,
            index: 0,
        });
        self.store.insert(name, self_symbol);
    }
    /// 先从当前符号表查找
    /// 没有则解析父符号表
    pub fn resolve(&mut self, name: &str) -> Option<Rc<Symbol>> {
        self.store
            .get(name)
            .cloned()
            .or_else(|| self.resolve_outer(name))
    }
    fn resolve_outer(&mut self, name: &str) -> Option<Rc<Symbol>> {
        self.outer
            .as_ref()
            .and_then(|s| s.borrow_mut().resolve(name))
            .map(|s| self.resolve_free(s))
    }
    fn resolve_free(&mut self, s: Rc<Symbol>) -> Rc<Symbol> {
        if s.scope == SymbolScope::Global || s.scope == SymbolScope::Builtin {
            s
        } else {
            self.define_free(s)
        }
    }

    pub fn new_enclosed(parent: RcSymbolTable) -> RcSymbolTable {
        let num_definitions = 0;
        let free_symbols = parent.borrow().free_symbols.clone();
        let outer = Some(parent);
        let store = Default::default();
        Rc::new(RefCell::new(Self {
            outer,
            store,
            num_definitions,
            free_symbols,
        }))
    }
}
