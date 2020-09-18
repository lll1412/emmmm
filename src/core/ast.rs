use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Let(String, Expression),
    Return(Option<Expression>),
    Expression(Expression),
    // Block(Vec<Statement>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct BlockStatement {
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    /// 标识符
    Identifier(String),
    // 整数字面量
    IntLiteral(i64),
    // 浮点数字面量
    FloatLiteral(f64),
    // 字符串字面量
    StringLiteral(String),
    // 布尔值字面量
    BoolLiteral(bool),
    // 一元表达式
    Unary(UnaryOperator, Box<Expression>),
    // 二元表达式
    Binary(BinaryOperator, Box<Expression>, Box<Expression>),

    // Condition(Box<Expression>),
    // if表达式
    If(Box<Expression>, BlockStatement, Option<BlockStatement>),
    // 函数表达式
    Fun(Vec<String>, BlockStatement),
    // 函数调用表达式
    Call(Box<Expression>, Vec<Expression>),
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
        writeln!(f, "{{")?;
        for statement in &self.statements {
            writeln!(f, "\t{}", statement)?;
        }
        write!(f, "}}")
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match &self {
            Statement::Let(name, val) => write!(f, "let {} = {};", name, val),
            Statement::Return(opt) => {
                let expression = opt.clone().unwrap_or(Expression::Identifier(String::new()));
                write!(f, "return {};", expression)
            }
            Statement::Expression(exp) => write!(f, "{}", exp),
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
            Expression::Fun(params, blocks) => write!(
                f,
                "fun({params}) {blocks}",
                params = params.join(", "),
                blocks = blocks
            ),
            Expression::Call(fun, exprs) => {
                let exprs: Vec<String> = exprs.into_iter().map(|exp| exp.to_string()).collect();
                write!(f, "{fun}({exprs})", fun = fun, exprs = exprs.join(","))
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
