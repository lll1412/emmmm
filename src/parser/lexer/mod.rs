use std::iter::Peekable;
use std::str::Chars;

mod r#impl;
mod test;
pub mod token;

#[derive(Debug, Clone)]
pub struct Lexer {
    input: String,
    position: usize,
    ch: char,
    chars: Peekable<Chars<'static>>,
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
