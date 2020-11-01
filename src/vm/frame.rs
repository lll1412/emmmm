use crate::compiler::code::Instructions;
use crate::object::Closure;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Frame {
    pub closure: Closure,
    pub ip: usize,
    pub base_pointer: usize,
}

impl Frame {
    pub fn new(closure: Closure, base_pointer: usize) -> Self {
        Self {
            closure,
            ip: 0,
            base_pointer,
        }
    }
    pub fn instructions(&self) -> Rc<Instructions> {
        self.closure.compiled_function.insts.clone()
    }
}
