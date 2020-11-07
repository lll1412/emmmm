use std::io;
use std::io::Write;
use std::rc::Rc;

use crate::compiler::symbol_table::SymbolTable;
use crate::eval::Environment;
use crate::object::Object;
use crate::parser::Parser;
use crate::{create_rc_ref_cell, exe_with_eval, exe_with_vm, Engine};

const PROMPT: &str = ">> ";
const TAB: &str = "   ";
const EXIT: &str = "exit\r\n";
const ENV: &str = "env\r\n";
const RT: &str = "\r\n";

pub fn start(engine: Engine) {
    println!("Welcome to the 👽 programming language in {}", engine);
    //for eval
    let env = create_rc_ref_cell(Environment::new());
    //for compiler and vm
    let globals = Vec::<Rc<Object>>::new();
    let symbol_table = create_rc_ref_cell(SymbolTable::new());
    let constants = vec![];

    let reader = io::stdin();
    let mut input = String::new();
    let mut new_statement = true;
    loop {
        if new_statement {
            print!("{}", PROMPT);
        } else {
            print!("{}", TAB);
        }
        io::stdout().flush().unwrap();

        let i = reader.read_line(&mut input).unwrap();
        if i == 0 || input == EXIT {
            println!("\nBye!");
            return;
        }

        if input == ENV {
            println!("Exists Variables: {:?}", env.borrow().keys());
            continue;
        } else if input == "Global\r\n" {
            println!("Globals: {:#?}", globals);
            continue;
        }

        let mut parser = Parser::from(&input);
        let program = parser.parse_program();
        let errors = parser.errors();
        if errors.is_empty() {
            match engine {
                Engine::Eval => exe_with_eval(&program, &env),
                Engine::Compile => exe_with_vm(
                    &program,
                    symbol_table.clone(),
                    constants.clone(),
                    globals.clone(),
                ),
            }
            new_statement = true;
            input.clear()
        } else {
            // println!("parser errors:");
            // for err in errors {
            //     println!("\t{:?}", err);
            // }
            if input.ends_with(RT) {
                input.pop();
                input.pop();
            }
            new_statement = false;
        }
    }
}
