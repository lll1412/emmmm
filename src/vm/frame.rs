use std::rc::Rc;

use crate::compiler::code::Instructions;
use crate::object::Object;

#[derive(Debug, Clone)]
pub struct Frame {
    pub closure: Rc<Object>,
    pub ip: usize,
    pub base_pointer: usize,
}

impl Frame {
    pub fn new(closure: Rc<Object>, base_pointer: usize) -> Self {
        Self {
            closure,
            ip: 0,
            base_pointer,
        }
    }
    pub fn instructions(&self) -> Rc<Instructions> {
        if let Object::Closure(closure) = &*self.closure {
            closure.compiled_function.insts.clone()
        } else {
            panic!()
        }
        // self.closure.compiled_function.insts.clone()
    }
    pub fn get_free(&self, free_index: usize) -> Rc<Object> {
        if let Object::Closure(closure) = &*self.closure {
            return closure.free_variables[free_index].clone();
        } else {
            panic!()
        }
    }
}
