use std::io;
use std::io::Write;
use std::rc::Rc;

use vm::Vm;

use crate::compiler::code::Constant;
use crate::compiler::symbol_table::SymbolTable;
use crate::compiler::Compiler;
use crate::core::base::ast::Program;
use crate::core::parser::Parser;
use crate::eval::evaluator;
use crate::eval::evaluator::Env;
use crate::object::{environment, Object};
use crate::vm::Globals;
use crate::{create_rc_ref_cell, vm};

const PROMPT: &str = ">> ";
const EXIT: &str = "exit\r\n";
const ENV: &str = "env\r\n";

pub fn start() {
    let env = create_rc_ref_cell(environment::Environment::new());
    let symbol_table = create_rc_ref_cell(SymbolTable::new());
    let constants = create_rc_ref_cell(Vec::<Constant>::new());
    let globals = create_rc_ref_cell(Vec::<Rc<Object>>::new());
    let reader = io::stdin();
    loop {
        print!("{}", PROMPT);
        io::stdout().flush().unwrap();
        let mut input = String::new();

        let i = reader.read_line(&mut input).unwrap();
        if i == 0 || input == EXIT {
            println!("Bye!");
            return;
        }

        if input == ENV {
            println!("Exists Variables: {:?}", env.borrow().keys());
            continue;
        }

        let mut parser = Parser::from(&input);
        let program = parser.parse_program();
        let errors = parser.errors();
        if !errors.is_empty() {
            println!("parser errors:");
            for err in errors {
                println!("\t{:?}", err);
            }
        } else {
            // exe_with_eval(&program, &env);
            let mut compiler = Compiler::with_state(symbol_table.clone(), constants.clone());
            exe_with_vm(program, &mut compiler, globals.clone());
        }
    }
}

fn _exe_with_eval(program: &Program, env: &Env) {
    let result = evaluator::eval(&program, env.clone());
    match result {
        Ok(object) => println!("{}", object),
        Err(err) => eprintln!("{}", err),
    }
}

fn exe_with_vm(program: Program, compiler: &mut Compiler, globals: Globals) {
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
