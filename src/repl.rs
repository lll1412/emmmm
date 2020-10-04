use std::cell::RefCell;
use std::io;
use std::io::Write;
use std::rc::Rc;

use vm::Vm;

use crate::compiler::Compiler;
use crate::core::base::ast::Program;
use crate::core::parser::Parser;
use crate::eval::evaluator;
use crate::eval::evaluator::Env;
use crate::object::environment;
use crate::vm;

const PROMPT: &str = ">> ";

pub fn start() {
    let env = Rc::new(RefCell::new(environment::Environment::new()));
    let reader = io::stdin();
    loop {
        print!("{}", PROMPT);
        io::stdout().flush().unwrap();
        let mut input: String = String::new();

        let i = reader.read_line(&mut input).unwrap();
        if i == 0 || input == "exit\r\n" {
            println!("Bye!");
            return;
        }

        if input == "env\r\n" {
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
            exe_with_vm(program, &env);
        }
    }
}

fn _exe_with_eval(program: &Program, env: &Env) {
    let result = evaluator::eval(&program, Rc::clone(env));
    match result {
        Ok(object) => println!("{}", object),
        Err(err) => eprintln!("{}", err),
    }
}

fn exe_with_vm(program: Program, _env: &Env) {
    let mut compiler = Compiler::new();
    let result = compiler.compile(program);
    match result {
        Ok(byte_code) => {
            let mut vm = Vm::new(byte_code);
            let result = vm.run();
            match result {
                Ok(object) => println!("{}", object),
                Err(vm_err) => eprintln!("{:?}", vm_err),
            }
        }
        Err(com_err) => eprintln!("{:?}", com_err),
    }
}
