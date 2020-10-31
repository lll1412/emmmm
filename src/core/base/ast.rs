use std::fmt::{Display, Formatter, Result};

use crate::core::parser::Parser;

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Program {
    pub fn _new(input: &str) -> Self {
        Parser::from(input).parse_program()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    // let ident = expr
    Let(String, Expression),
    // for (initial; condition; last) { blockStatement }
    // For(
    //     Option<Expression>,
    //     Option<Expression>,
    //     Option<Expression>,
    //     BlockStatement,
    // ),
    // return
    Return(Option<Expression>),
    Comment(String),
    //
    Expression(Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub struct BlockStatement {
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    // 标识符
    Identifier(String),
    // 整数字面量
    IntLiteral(i64),
    // 浮点数字面量
    FloatLiteral(f64),
    // 字符串字面量
    StringLiteral(String),
    // 布尔值字面量
    BoolLiteral(bool),
    // 函数字面量
    FunctionLiteral(Vec<String>, BlockStatement),
    // 数组字面量
    ArrayLiteral(Vec<Expression>),
    // 索引表达式
    Index(Box<Expression>, Box<Expression>),
    // 映射表
    HashLiteral(Vec<(Expression, Expression)>),

    // 一元表达式
    Unary(UnaryOperator, Box<Expression>),
    // 二元表达式
    Binary(BinaryOperator, Box<Expression>, Box<Expression>),

    // if表达式
    If(Box<Expression>, BlockStatement, Option<BlockStatement>),
    // 函数调用表达式, (函数, 参数)
    Call(Box<Expression>, Vec<Expression>),
    //
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOperator {
    Not,
    Neg,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Mul,
    Div,

    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    NotEq,

    Assign,
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for statement in &self.statements {
            write!(f, "{}", statement)?;
        }
        Ok(())
    }
}

impl Display for BlockStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{{")?;
        for statement in &self.statements {
            write!(f, " {} ", statement)?;
        }
        write!(f, "}}")
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match &self {
            Statement::Let(name, val) => write!(f, "let {} = {}; ", name, val),
            Statement::Return(opt) => {
                let expression = opt
                    .clone()
                    .unwrap_or_else(|| Expression::Identifier(String::new()));
                write!(f, "return {}; ", expression)
            }
            Statement::Expression(exp) => write!(f, "{}", exp),
            // Statement::For(_, _, _, _) => write!(f, ""),
            Statement::Comment(comment) => write!(f, "// {}", comment),
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Expression::IntLiteral(int) => write!(f, "{}", int),
            Expression::FloatLiteral(float) => write!(f, "{}", float),
            Expression::StringLiteral(string) => write!(f, "{}", string),
            Expression::BoolLiteral(boolean) => write!(f, "{}", boolean),
            Expression::Identifier(id) => write!(f, "{}", id),
            Expression::Unary(pfx, expr) => write!(f, "({}{})", pfx, expr),
            Expression::Binary(ifx, left, right) => write!(f, "({} {} {})", left, ifx, right),
            Expression::If(condition, consequence, alternative) => {
                write!(f, "if {} {}", condition, consequence)?;
                if let Some(alternative) = alternative {
                    write!(f, " else {}", alternative)?;
                }
                Ok(())
            }
            Expression::FunctionLiteral(params, blocks) => write!(
                f,
                "fn({params}) {blocks}",
                params = params.join(", "),
                blocks = blocks
            ),
            Expression::Call(fun, exprs) => {
                let exprs: String = exprs
                    .iter()
                    .map(|exp| exp.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{fun}({exprs})", fun = fun, exprs = exprs)
            }
            Expression::ArrayLiteral(elements) => {
                let exprs: String = elements
                    .iter()
                    .map(|ex| ex.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "[{elements}]", elements = exprs)
            }
            Expression::Index(left_expr, index_expr) => write!(
                f,
                "({left}[{index}])",
                left = left_expr.to_string(),
                index = index_expr.to_string()
            ),
            Expression::HashLiteral(hash) => {
                let r = hash
                    .iter()
                    .map(|(k, v)| format!("{}:{}", k, v))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{{{}}}", r)
            }
        }
    }
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            UnaryOperator::Not => write!(f, "!"),
            UnaryOperator::Neg => write!(f, "-"),
        }
    }
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            BinaryOperator::Plus => write!(f, "+"),
            BinaryOperator::Minus => write!(f, "-"),
            BinaryOperator::Mul => write!(f, "*"),
            BinaryOperator::Div => write!(f, "/"),
            BinaryOperator::Gt => write!(f, ">"),
            BinaryOperator::Ge => write!(f, ">="),
            BinaryOperator::Lt => write!(f, "<"),
            BinaryOperator::Le => write!(f, "<="),
            BinaryOperator::Eq => write!(f, "=="),
            BinaryOperator::NotEq => write!(f, "!="),
            BinaryOperator::Assign => write!(f, "="),
        }
    }
}
