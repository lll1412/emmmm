use std::cell::RefCell;
use std::rc::Rc;

pub mod benchmark;
mod compiler;
mod core;
mod eval;
mod object;
pub mod repl;
mod vm;

fn create_rc_ref_cell<T>(t: T) -> Rc<RefCell<T>> {
    Rc::new(RefCell::new(t))
}
