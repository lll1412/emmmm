use std::cell::RefCell;
use std::rc::Rc;

use crate::compiler::code::{CompiledFunction, Instructions};

#[derive(Debug, Clone)]
pub struct Frame {
    fun: CompiledFunction,
    pub ip: usize,
    pub base_pointer: usize,
}

impl Frame {
    pub fn new(fun: CompiledFunction, base_pointer: usize) -> Self {
        Self {
            fun,
            ip: 0,
            base_pointer,
        }
    }
    pub fn _new_rc(fun: CompiledFunction) -> Rc<Self> {
        Rc::new(Self::new(fun, 0))
    }
    pub fn _new_ref(fun: CompiledFunction) -> RefCell<Self> {
        RefCell::new(Self::new(fun, 0))
    }
    pub fn instructions(&self) -> Instructions {
        self.fun.insts.clone()
    }
}
