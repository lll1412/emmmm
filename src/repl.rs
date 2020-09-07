use crate::lexer::Lexer;
use std::io;
use crate::token::Token;
use std::io::Write;

const PROMPT: &str = ">> ";

pub fn start() {
    loop {
        { print!("{}", PROMPT); }
        io::stdout().flush().unwrap();
        let reader = io::stdin();
        let mut buffer: String = String::new();

        let i = reader.read_line(&mut buffer).unwrap();

        if i == 0 || buffer == "exit\n" {
            println!("Bye!");
            return;
        }

        let mut lexer = Lexer::new(buffer);

        loop {
            let token = lexer.next_token();
            if token == Token::Eof { break; }
            println!("{:?}", token);
        }
    }
}