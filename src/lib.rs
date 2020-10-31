use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use crate::eval::evaluator;
use crate::core::base::ast::Program;
use crate::eval::evaluator::Env;
use crate::compiler::Compiler;
use crate::vm::{Globals, Vm};

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

pub enum Engine {
    Eval,
    Compile,
}
pub enum Mode {
    Benchmark,
    Run,
}
impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Engine::Eval => write!(f, "eval mode"),
            Engine::Compile => write!(f, "compile mode"),
        }
    }
}
fn has_flag(flag: &str) -> bool {
    std::env::args().any(|arg| arg == flag)
}
pub fn eval_or_compile() -> Engine {
    if has_flag("--benchmark") {
        Engine::Compile
    } else {
        Engine::Eval
    }
}
pub fn current_mode() -> Mode {
    if has_flag("--benchmark") {
        Mode::Benchmark
    } else {
        Mode::Run
    }
}
pub fn exe_with_eval(program: &Program, env: &Env) {
    let result = evaluator::eval(program, env.clone());
    match result {
        Ok(object) => println!("{}", object),
        Err(err) => eprintln!("{}", err),
    }
}

pub fn exe_with_vm(program: &Program, compiler: &mut Compiler, globals: Globals) {
    let result = compiler.compile(program);
    match result {
        Ok(byte_code) => {
            let mut vm = Vm::with_global_store(byte_code, globals.clone());
            let result = vm.run();
            match result {
                Ok(object) => println!("{}", object),
                Err(vm_err) => eprintln!("{:?}", vm_err),
            }
        }
        Err(com_err) => eprintln!("{:?}", com_err),
    }
}