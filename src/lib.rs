use compiler::code::Opcode;
use std::cell::RefCell;
use std::env;
use std::fmt;
use std::rc::Rc;
use std::time::Instant;

use crate::compiler::{Compiler, Constants, RcSymbolTable};
use crate::eval::evaluator;
use crate::eval::evaluator::Env;
use crate::parser::base::ast::Program;
use crate::vm::{Globals, Vm};

pub mod benchmark;
mod compiler;
mod eval;
mod object;
mod parser;
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
    env::args().any(|arg| arg == flag)
}
pub fn eval_or_compile() -> Engine {
    if has_flag("--eval") {
        Engine::Eval
    } else {
        Engine::Compile
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
        Err(err) => eprintln!("Error: {}", err),
    }
}

pub fn parse_file() -> Option<String> {
    for arg in env::args() {
        if arg.ends_with(".my") {
            return Some(arg);
        }
    }
    None
}
pub fn exe_with_vm(
    program: &Program,
    symbol_table: RcSymbolTable,
    constants: Constants,
    globals: Globals,
) {
    let mut compiler = Compiler::with_state(symbol_table, constants);
    let result = compiler.compile(program);
    match result {
        Ok(byte_code) => {
            let mut vm = Vm::with_global_store(byte_code, globals.clone());
            let _start = Instant::now();
            let result = vm.run();
            match result {
                Ok(object) => {
                    println!("{}", object);
                    // println!("takes {} ms", _start.elapsed().as_millis());
                    // println!("globals: \n{:#?}", globals);
                }
                Err(vm_err) => eprintln!("Error: {:?}", vm_err),
            }
        }
        Err(com_err) => eprintln!("{:?}", com_err),
    }
}
pub struct TimeRecorder {
    _start: std::time::Instant,
    _record_map: std::collections::HashMap<u8, RefCell<Pair>>,
}
#[derive(Default, Debug)]
struct Pair<T = u128, C = u64> {
    time: T,
    count: C,
}
impl TimeRecorder {
    fn _new() -> Self {
        Self {
            _start: std::time::Instant::now(),
            _record_map: Default::default(),
        }
    }
    fn _tick(&mut self, key: Opcode) {
        let key = key as u8;
        let time = self._start.elapsed().as_nanos();
        if self._record_map.contains_key(&key) {
            let mut pair = self._record_map[&key].borrow_mut();
            pair.count += 1;
            pair.time += time;
        } else {
            self._record_map.insert(key, RefCell::new(Pair::default()));
        }
        self._start = std::time::Instant::now();
    }
    fn _print_sorted_record(&self) {
        let mut map = std::collections::BTreeMap::new();
        let mut time_all = 0;
        for (k, v) in self._record_map.iter() {
            let v = v.borrow();
            time_all += v.time;
            map.insert(
                std::time::Duration::from_nanos(v.time as u64),
                Pair {
                    time: Opcode::from_byte(*k).unwrap(),
                    count: v,
                },
            );
            // println!(
            //     "{:?} ==> {:?}",
            //     op,
            //     std::time::Duration::from_nanos(*v as u64)
            // );
        }
        for (k, v) in map.iter_mut() {
            println!("{:?} ==> {:?}", v, k);
        }
        println!(
            "解析运行指令用时: {:?}",
            std::time::Duration::from_nanos(time_all as u64)
        );
    }
}
