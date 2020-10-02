use std::cell::RefCell;
use std::io;
use std::io::Write;
use std::rc::Rc;

use crate::core::parser::Parser;
use crate::eval::{environment, evaluator};

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
            let result = evaluator::eval(&program, Rc::clone(&env));
            match result {
                Ok(object) => println!("{}", object),
                Err(err) => eprintln!("{}", err),
            }
        }
    }
}
