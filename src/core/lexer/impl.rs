use crate::core::lexer::Lexer;
use crate::core::token::*;
use std::iter::Peekable;
use std::str::Chars;

impl Lexer {
    pub fn new(input: String) -> Self {
        let chars: Peekable<Chars> = unsafe { std::mem::transmute(input.chars().peekable()) };
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
    /// 读取Token
    pub fn parse_token(&mut self) -> Token {
        self.skip_whitespace();
        // println!("peek: {:?}", self.peek_char());
        let token = match self.ch {
            '(' => Token::Lparen,
            ')' => Token::Rparen,
            '{' => Token::Lbrace,
            '}' => Token::Rbrace,
            '[' => Token::Lbracket,
            ']' => Token::Rbracket,

            ',' => Token::Comma,
            ':' => Token::Colon,
            ';' => Token::Semicolon,

            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Asterisk,
            '/' => Token::Slash,
            '>' => {
                if self.peek_char() == &'=' {
                    Token::Ge
                } else {
                    Token::Gt
                }
            }
            '<' => {
                if self.peek_char() == &'=' {
                    Token::Le
                } else {
                    Token::Lt
                }
            }
            '!' => {
                if self.peek_char() == &'=' {
                    self.read_char();
                    Token::NotEq
                } else {
                    Token::Bang
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
            c => {
                return if is_letter(c) {
                    let id = self.read_identifier();
                    Token::lookup_id(id)
                } else if is_digit(c) {
                    let num = self.read_number();
                    if num.contains('.') {
                        Token::Float(num.to_string())
                    } else {
                        Token::Int(num.to_string())
                    }
                } else if c == '"' || c == '`' {
                    //may be string
                    let string = self.read_string();
                    Token::String(string.to_string())
                } else {
                    Token::Illegal(c)
                };
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
    //读取字符串
    fn read_string(&mut self) -> &str {
        let around_ch = self.ch;
        self.read_char(); // start " or `
        let position_start = self.position;
        while self.ch != around_ch {
            self.read_char();
        }
        let position_end = self.position;
        self.read_char(); // end " or `
        &self.input[position_start..position_end]
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
