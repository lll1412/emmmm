use std::iter::Peekable;
use std::str::Chars;

use crate::parser::lexer::token::*;
use crate::parser::lexer::{is_digit, is_letter, Lexer};

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars = input.chars().collect::<Vec<_>>();
        let mut lexer = Self {
            input: String::from(input),
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
            '/' => {
                if self.peek_char() == &'/' {
                    let line = self.read_line();
                    Token::Comment(line.to_string())
                } else {
                    Token::Slash
                }
            }
            '>' => self.peek_is_or('=', Token::Ge, Token::Gt),
            '<' => self.peek_is_or('=', Token::Le, Token::Lt),
            '!' => self.peek_is_eat_or('=', Token::NotEq, Token::Bang),
            '=' => self.peek_is_eat_or('=', Token::Eq, Token::Assign),
            '"' | '`' => {
                //may be string
                let string = self.read_string();
                Token::String(string)
            }
            EOF => Token::Eof,
            c => {
                return if is_letter(c) {
                    //标识符
                    let id = self.read_identifier();
                    Token::lookup_id(id)
                } else if is_digit(c) {
                    //数字
                    let num = self.read_number();
                    if num.contains('.') {
                        Token::Float(num.to_string())
                    } else {
                        Token::Int(num.to_string())
                    }
                } else {
                    //非法字符
                    return Token::Illegal;
                };
            }
        };
        self.read_char();
        token
    }
    fn read_line(&mut self) -> &str {
        let position = self.position;
        while self.peek_char() != &'\n' {
            self.read_char();
        }
        &self.input[position..self.position]
    }
    //跳过整行
    fn _skip_line(&mut self) {
        while self.peek_char() != &'\n' {
            self.read_char();
        }
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
    fn read_string(&mut self) -> String {
        let around_ch = self.ch;
        self.read_char(); // start with " or `
        let mut last_char = EOF;
        let mut result = String::new();
        loop {
            if last_char == '\\' {
                //escape
                let escaped_str = self.escape_char(self.ch);
                if escaped_str != EOF {
                    result.push(escaped_str);
                    if escaped_str == '\\' {
                        last_char = '\u{0}';
                        continue;
                    }
                }
            } else if self.ch == '\\' {
            } else {
                if self.ch == around_ch {
                    break;
                }
                result.push(self.ch);
            }
            last_char = self.ch;
            self.read_char();
        }
        result
    }
    //
    fn escape_char(&mut self, c: char) -> char {
        match c {
            't' => '\t',
            'n' => '\n',
            '"' => '\"',
            '`' => '`',
            '\\' => {
                self.read_char();
                '\\'
            }
            _ => EOF,
        }
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
        self.ch = self.chars.get(self.position).cloned().unwrap_or(EOF);
    }
    //查看字符
    fn peek_char(&mut self) -> &char {
        self.chars.get(self.position + 1).unwrap_or(&EOF)
    }
    //预检下个字符是否为期待字符，是则返回期待Token，并向后读取一个字符，否则返回默认Token
    fn peek_is_eat_or(&mut self, c: char, expect_token: Token, default_token: Token) -> Token {
        if self.peek_char().eq(&c) {
            self.read_char();
            expect_token
        } else {
            default_token
        }
    }
    //预检下个字符是否为期待字符，是则返回期待Token，否则返回默认Token
    fn peek_is_or(&mut self, c: char, expect_token: Token, default_token: Token) -> Token {
        if self.peek_char().eq(&c) {
            expect_token
        } else {
            default_token
        }
    }
}
