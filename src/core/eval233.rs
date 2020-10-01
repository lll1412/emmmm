use crate::core::ast::{Expression, Statement, UnaryOperator};
use crate::core::parser::Parser;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

type Result<T> = std::result::Result<T, EvalError>;

struct Eval {
    parser: Parser,
    map: HashMap<String, Expression>,
}

impl Eval {
    fn new(parser: Parser) -> Self {
        Self {
            parser,
            map: HashMap::new(),
        }
    }

    fn from(input: String) -> Self {
        let parser = Parser::from(input);
        Eval::new(parser)
    }

    fn eval(&mut self) {
        let program = self.parser.parse_program();
        let statements = program.statements;
        for statement in statements {
            let result = match statement {
                Statement::Let(name, expr) => self.eval_let_statement(name, expr),
                Statement::Return(opt) => self.eval_return_statement(opt),
                Statement::Expression(expr) => self.eval_expr_statement(expr),
            };
            print_eval(result);
        }
    }

    fn eval_let_statement(&mut self, name: String, expr: Expression) -> Result<Expression> {
        match self.map.get(&name) {
            None => {
                let val = self.eval_expression(expr)?;
                self.map.insert(name, val.clone());
                Ok(val)
            }
            Some(_) => Err(EvalError::RepeatVariableDeclare(name)),
        }
    }

    fn eval_return_statement(&mut self, opt: Option<Expression>) -> Result<Expression> {
        match opt {
            None => Ok(Expression::StringLiteral("None".to_string())),
            Some(expr) => self.eval_expression(expr),
        }
    }

    fn eval_expr_statement(&mut self, expr: Expression) -> Result<Expression> {
        self.eval_expression(expr)
    }

    fn eval_expression(&mut self, expr: Expression) -> Result<Expression> {
        match expr {
            Expression::Identifier(ident) => self.eval_exists_val(ident.clone()),
            // Expression::IntLiteral(int) => Ok(int.to_string()),
            // Expression::FloatLiteral(float) => Ok(float.to_string()),
            // Expression::StringLiteral(string) => Ok(format!("\"{}\"", string)),
            // Expression::BoolLiteral(bool) => Ok(bool.to_string()),
            Expression::Unary(op, exp) => self.eval_unary_expression(op, *exp),
            Expression::Binary(_, _, _) => {}
            Expression::If(_, _, _) => {}
            Expression::Fun(_, _) => {}
            Expression::Call(_, _) => {}
            expr => Ok(expr),
        }
    }

    fn eval_unary_expression(&mut self, op: UnaryOperator, expr: Expression) -> Result<Expression> {
        let expr = match expr {
            Expression::Identifier(key) => {
                let expr = self.eval_exists_val(key)?;
                match op {
                    UnaryOperator::Not => {}
                    UnaryOperator::Neg => {}
                }
            }
            Expression::IntLiteral(val) => {}
            Expression::FloatLiteral(val) => {}
            Expression::StringLiteral(val) => {}
            Expression::BoolLiteral(val) => {}
            expr => self.eval_unary_expression(op, expr),
        };
        let eval_expr = self.eval_expression(expr)?;
    }

    fn eval_exists_val(&mut self, key: String) -> Result<Expression> {
        match self.map.get(&key) {
            Some(expr) => Ok(expr.clone()),
            None => Err(EvalError::UndeclaredVariable(key)),
        }
    }
}

fn print_eval(result: Result<Expression>) {
    match result {
        Ok(res) => print_eval_value(res.to_string()),
        Err(err) => print_eval_error(err),
    }
}

fn print_eval_value(val: &str) {
    println!("{}", val);
}

fn print_eval_error(err: EvalError) {
    println!("{}", err);
}

#[derive(Debug)]
enum EvalError {
    UndeclaredVariable(String),
    RepeatVariableDeclare(String),
}

impl Display for EvalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::UndeclaredVariable(ident) => write!(f, "undeclared variable: {}.", ident),
            EvalError::RepeatVariableDeclare(ident) => {
                write!(f, "variable: {} has been declared.", ident)
            }
        }
    }
}
