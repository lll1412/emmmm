#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Program {}

#[derive(Debug)]
pub enum Statement {
    Let(String, Expression),
    Return(Option<Expression>),
    Expression(Expression),
}

#[derive(Debug)]
pub enum Expression {
    // Identifier(String),
    IntegerLiteral(i64),
}
