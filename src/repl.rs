use std::io;
use std::io::Write;
use std::rc::Rc;

use crate::compiler::symbol_table::SymbolTable;
use crate::core::parser::Parser;
use crate::object::{environment, Object};
use crate::{create_rc_ref_cell, exe_with_eval, exe_with_vm, Engine};

const PROMPT: &str = ">> ";
const EXIT: &str = "exit\r\n";
const ENV: &str = "env\r\n";

pub fn start(engine: Engine) {
    println!("Welcome to the 👽 programming language in {}", engine);
    //for eval
    let env = create_rc_ref_cell(environment::Environment::new());
    //for compiler and vm
    let globals = create_rc_ref_cell(Vec::<Rc<Object>>::new());
    let symbol_table = create_rc_ref_cell(SymbolTable::new());
    let constants = create_rc_ref_cell(vec![]);

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
        } else if input == "Global\r\n" {
            println!("Globals: {:#?}", globals);
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
            match engine {
                Engine::Eval => exe_with_eval(&program, &env),
                Engine::Compile => exe_with_vm(
                    &program,
                    symbol_table.clone(),
                    constants.clone(),
                    globals.clone(),
                ),
            }
        }
    }
}
