use crate::core::parser::Parser;
use std::io;
use std::io::Write;

const PROMPT: &str = ">> ";

pub fn start() {
    loop {
        print!("{}", PROMPT);
        io::stdout().flush().unwrap();
        let reader = io::stdin();
        let mut input: String = String::new();

        let i = reader.read_line(&mut input).unwrap();

        if i == 0 || input == "exit\n" {
            println!("Bye!");
            return;
        }

        let mut parser = Parser::from(input);
        let program = parser.parse_program();
        let errors = parser.errors();
        if errors.len() != 0 {
            println!("parser errors:");
            for err in errors {
                println!("\t{:?}", err);
            }
        } else {
            let statements = program.statements;
            for statement in statements {
                println!("{}", statement);
            }
        }
    }
}
