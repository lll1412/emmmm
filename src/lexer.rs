use crate::token::*;
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer {
    input: String,
    position: usize,
    ch: char,
    chars: Peekable<Chars<'static>>,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let chars = unsafe { std::mem::transmute(input.chars().peekable()) };
        let input = input.to_string();
        let mut lexer = Self {
            input,
            position: 0,
            ch: EOF,
            chars,
        };
        lexer.read_char();
        lexer
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let token = match self.ch {
            '(' => Token::Lparen,
            ')' => Token::Rparen,
            '{' => Token::Lbrace,
            '}' => Token::Rbrace,

            ',' => Token::Comma,
            ':' => Token::Colon,
            ';' => Token::Semicolon,

            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Asterisk,
            '/' => Token::Slash,
            '>' => Token::Gt,
            '<' => Token::Lt,
            '!' => {
                if self.peek_char() == &'=' {
                    self.read_char();
                    Token::NotEq
                } else {
                    Token::Not
                }
            }
            '=' => {
                if self.peek_char().eq(&'=') {
                    self.read_char();
                    Token::Eq
                } else {
                    Token::Assign
                }
            }
            EOF => Token::Eof,
            ch => {
                return if is_letter(ch) {
                    let id = self.read_identifier();
                    Token::lookup_id(id)
                } else if is_digit(ch) {
                    let num = self.read_number();
                    if num.contains('.') {
                        Token::Float(num.to_string())
                    } else {
                        Token::Int(num.to_string())
                    }
                } else {
                    Token::Illegal(ch)
                }
            }
        };
        self.read_char();
        token
    }

    //读取标识符
    fn read_identifier(&mut self) -> &str {
        let position = self.position;
        //the first char must be a letter
        if is_letter(self.ch) {
            self.read_char();
        }
        while is_letter(self.ch) || is_digit(self.ch) {
            self.read_char();
        }
        &self.input[position..self.position]
    }
    //读取数字
    fn read_number(&mut self) -> &str {
        let position = self.position;
        while is_digit(self.ch) || self.ch == '.' {
            self.read_char();
        }
        &self.input[position..self.position]
    }
    //忽略空格
    fn skip_whitespace(&mut self) {
        while self.ch == ' ' || self.ch == '\r' || self.ch == '\t' || self.ch == '\n' {
            self.read_char();
        }
    }
    //读取一个字符
    fn read_char(&mut self) {
        self.position += if self.ch == EOF {
            0
        } else {
            self.ch.len_utf8()
        };
        self.ch = self.chars.next().unwrap_or(EOF);
    }
    //查看字符
    fn peek_char(&mut self) -> &char {
        self.chars.peek().unwrap_or(&EOF)
    }
}

fn is_letter(ch: char) -> bool {
    ch == '_' ||
        // ch.is_alphabetic()
        (ch >= 'a' && ch <= 'z') ||
        (ch >= 'A' && ch <= 'Z')
}

fn is_digit(ch: char) -> bool {
    ch.is_numeric()
}

#[cfg(test)]
mod token_test {
    use super::*;

    #[test]
    fn test_next_token() {
        let input = r#"
        let five = 5.1;
        let ten = 10;
        let add = fun(x, y) {
            return x + y;
        };
        let result = add(five, ten);
        !-/*5;
        5 < 10 > 5;
        if 5 < 10 {
            return true;
        } else {
            return false;
        }
        10 == 10;
        10 !=9;
"#;

        let tests = [
            Token::Let,
            Token::Ident("five".to_string()),
            Token::Assign,
            Token::Float("5.1".to_string()),
            Token::Semicolon,
            Token::Let,
            Token::Ident("ten".to_string()),
            Token::Assign,
            Token::Int(String::from("10")),
            Token::Semicolon,
            Token::Let,
            Token::Ident("add".to_string()),
            Token::Assign,
            Token::Function,
            Token::Lparen,
            Token::Ident("x".parse().unwrap()),
            Token::Comma,
            Token::Ident("y".to_string()),
            Token::Rparen,
            Token::Lbrace,
            Token::Return,
            Token::Ident("x".to_string()),
            Token::Plus,
            Token::Ident("y".to_string()),
            Token::Semicolon,
            Token::Rbrace,
            Token::Semicolon,
            Token::Let,
            Token::Ident("result".to_string()),
            Token::Assign,
            Token::Ident("add".to_string()),
            Token::Lparen,
            Token::Ident("five".to_string()),
            Token::Comma,
            Token::Ident("ten".to_string()),
            Token::Rparen,
            Token::Semicolon,
            Token::Not,
            Token::Minus,
            Token::Slash,
            Token::Asterisk,
            Token::Int("5".to_string()),
            Token::Semicolon,
            Token::Int("5".to_string()),
            Token::Lt,
            Token::Int("10".to_string()),
            Token::Gt,
            Token::Int("5".to_string()),
            Token::Semicolon,
            Token::If,
            Token::Int("5".to_string()),
            Token::Lt,
            Token::Int("10".to_string()),
            Token::Lbrace,
            Token::Return,
            Token::True,
            Token::Semicolon,
            Token::Rbrace,
            Token::Else,
            Token::Lbrace,
            Token::Return,
            Token::False,
            Token::Semicolon,
            Token::Rbrace,
            Token::Int("10".to_string()),
            Token::Eq,
            Token::Int("10".to_string()),
            Token::Semicolon,
            Token::Int("10".to_string()),
            Token::NotEq,
            Token::Int("9".to_string()),
            Token::Semicolon,
            Token::Eof,
        ];

        let mut lexer = Lexer::new(input.to_string());
        for (i, expected_token) in tests.iter().enumerate() {
            let token = lexer.next_token();
            // println!("{:?}", token);
            assert_eq!(&token, expected_token, "tests[{}] - token", i);
        }
    }
}
