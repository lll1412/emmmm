use crate::parser::{
    base::{ast::Expression, token::Token},
    lexer::Lexer,
};

pub mod base;
mod r#impl;
mod lexer;
mod test;

type ParseResult<T = Expression> = std::result::Result<T, ParserError>;
type UnaryParseFn = fn(&mut Parser) -> ParseResult<Expression>;
type BinaryParseFn = fn(&mut Parser, Expression) -> ParseResult<Expression>;

#[derive(Debug)]
pub struct Parser {
    lexer: Lexer,
    token: Token,
    peek_token: Token,
    errors: Vec<ParserError>,
}

/// 优先级
#[derive(PartialOrd, PartialEq, Ord, Eq, Debug, Clone)]
enum Precedence {
    Lowest,
    /// =
    Assign,
    /// ==
    Equals,
    /// \> or <
    LessGreater,
    /// `+`
    Sum,
    /// `*`
    Product,
    /// -x or !x
    Prefix,
    /// my_fun
    Call,

    Index,
}
/// 解析错误类
#[derive(Debug)]
pub enum ParserError {
    /// expected, actual
    Expected(Token, Token),

    ExpectedUnaryOp(Token),
    ExpectedBinaryOp(Token),

    ExpectedAssign(Token),

    ExpectedIdentifier(Token),
    ExpectedInteger(Token),
    ExpectedFloat(Token),
    ExpectedString(Token),
    ExpectedBoolean(Token),

    ParseInt(String),
    // ParseFloat(String),
}
