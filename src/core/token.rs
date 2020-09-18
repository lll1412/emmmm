use std::fmt;

pub const EOF: char = '\u{0}';

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    /// 非法字符
    Illegal(char),
    /// End Of File
    Eof,
    //标识符和字面量
    Ident(String),
    Int(String),
    Float(String),
    String(String),
    //操作符
    /// =
    Assign,
    /// +
    Plus,
    /// -
    Minus,
    /// !
    Bang,
    /// *
    Asterisk,
    /// /
    Slash,
    /// <
    Lt,
    /// >
    Gt,
    /// ==
    Eq,
    /// !=
    NotEq,
    /// <=
    Le,
    /// >=
    Ge,
    //分隔符等其他符号
    /// ,
    Comma,
    /// :
    Colon,
    /// ;
    Semicolon,
    /// (
    Lparen,
    /// )
    Rparen,
    /// {
    Lbrace,
    /// }
    Rbrace,
    /// [
    Lbracket,
    /// ]
    Rbracket,

    //关键字
    /// fun
    Function,
    /// let
    Let,
    /// true
    True,
    /// false
    False,
    /// if
    If,
    /// else
    Else,
    /// return
    Return,
}

impl Token {
    pub fn lookup_id(key: &str) -> Self {
        match key {
            "fun" => Token::Function,
            "let" => Token::Let,
            "true" => Token::True,
            "false" => Token::False,
            "if" => Token::If,
            "else" => Token::Else,
            "return" => Token::Return,
            id => Token::Ident(id.to_string()),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Illegal(ch) => write!(f, "ILLEGAL: {}", ch),
            Token::Eof => write!(f, "EOF"),
            Token::Ident(id) => write!(f, "Ident({})", id),
            Token::Int(int) => write!(f, "Int({})", int),
            Token::Float(float) => write!(f, "Float({})", float),
            Token::String(string) => write!(f, "String(\"{}\")", string),
            Token::Assign => write!(f, "="),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Bang => write!(f, "!"),
            Token::Asterisk => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Lt => write!(f, "<"),
            Token::Gt => write!(f, ">"),
            Token::Eq => write!(f, "=="),
            Token::NotEq => write!(f, "!="),
            Token::Le => write!(f, "<="),
            Token::Ge => write!(f, ">="),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
            Token::Lparen => write!(f, "("),
            Token::Rparen => write!(f, ")"),
            Token::Lbrace => write!(f, "{{"),
            Token::Rbrace => write!(f, "}}"),
            Token::Lbracket => write!(f, "["),
            Token::Rbracket => write!(f, "]"),
            Token::Function => write!(f, "fun"),
            Token::Let => write!(f, "let"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::Return => write!(f, "return"),
        }
    }
}
