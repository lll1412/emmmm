use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result};
use std::rc::Rc;

use crate::core::base::ast::{BinaryOperator, BlockStatement, Expression, UnaryOperator};
use crate::eval::evaluator::EvalResult;
use crate::object::environment::Environment;

pub mod environment;

type BuiltinFunction = fn(Vec<Object>) -> EvalResult<Object>;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Integer(i64),
    _Float(f64),
    Boolean(bool),
    String(String),
    Array(RefCell<Vec<Object>>),
    Hash(RefCell<HashMap<HashKey, Object>>),
    Function(Vec<String>, BlockStatement, Rc<RefCell<Environment>>),
    Builtin(BuiltinFunction),
    Return(Box<Object>),
    Null,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum HashKey {
    Integer(i64),
    String(String),
    Boolean(bool),
}

impl HashKey {
    pub fn from_object(obj: &Object) -> EvalResult<HashKey> {
        match obj {
            Object::String(str) => Ok(HashKey::String(str.to_string())),
            Object::Integer(int) => Ok(HashKey::Integer(*int)),
            Object::Boolean(bool) => Ok(HashKey::Boolean(*bool)),
            _ => Err(EvalError::UnsupportedHashKey(obj.clone())),
        }
    }
}

impl Display for HashKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            HashKey::Integer(i) => write!(f, "{}", i),
            HashKey::String(s) => write!(f, "\"{}\"", s),
            HashKey::Boolean(b) => write!(f, "{}", b),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum EvalError {
    TypeMismatch(BinaryOperator, Object, Object),
    UnknownUnaryOperator(UnaryOperator, Object),
    UnknownBinaryOperator(BinaryOperator, Object, Object),
    IdentifierNotFound(String),

    NotCallable(Object),
    UnsupportedExpression(Expression),
    // (fun name, arg)
    BuiltinUnSupportedArg(String, Vec<Object>),
    // wrong arguments num(expected, given).
    BuiltinIncorrectArgNum(usize, usize),

    IndexUnsupported(Object),
    AssignUnsupported(Expression, Expression),

    UnsupportedHashKey(Object),
}

impl Display for EvalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            EvalError::TypeMismatch(op, l, r) => write!(
                f,
                "type mismatch: {} {} {}",
                l.type_name(),
                op,
                r.type_name()
            ),
            EvalError::UnknownUnaryOperator(op, ex) => {
                write!(f, "unknown operator: {}{}", op, ex.type_name())
            }
            EvalError::UnknownBinaryOperator(op, l, r) => write!(
                f,
                "unknown operator: {} {} {}",
                l.type_name(),
                op,
                r.type_name()
            ),
            EvalError::IdentifierNotFound(id) => write!(f, "identifier not found: {}", id),
            EvalError::NotCallable(fun) => write!(f, "not callable fun: {}", fun),
            EvalError::BuiltinUnSupportedArg(name, args) => write!(
                f,
                "argument to `{}` not supported, got {}",
                name,
                args.iter()
                    .map(|a| a.type_name())
                    .collect::<Vec<&str>>()
                    .join(", ")
            ),
            EvalError::BuiltinIncorrectArgNum(expected_num, actual_num) => write!(
                f,
                "wrong number of arguments. got {}, want {}",
                actual_num, expected_num
            ),
            EvalError::UnsupportedExpression(expr) => write!(f, "unsupported expression: {}", expr),
            EvalError::IndexUnsupported(left) => write!(f, "index operator not support: {}", left),
            EvalError::AssignUnsupported(left, val) => write!(
                f,
                "unsupported assign expression.{} can't assign to {}",
                val, left
            ),
            EvalError::UnsupportedHashKey(obj) => {
                write!(f, "can't used as a hash key: {}", obj.type_name())
            }
        }
    }
}

impl Object {
    fn type_name(&self) -> &str {
        match self {
            Object::Integer(_) => "INTEGER",
            Object::_Float(_) => "FLOAT",
            Object::Boolean(_) => "BOOLEAN",
            Object::String(_) => "STRING",
            Object::Array(_) => "ARRAY",
            Object::Hash(_) => "HASH",
            Object::Function(_, _, _) => "FUNCTION",
            Object::Builtin(_) => "BUILTIN_FUNCTION",
            Object::Null => "NULL",
            _ => "UNKNOWN",
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Object::Integer(int) => write!(f, "{}", int),
            Object::_Float(float) => write!(f, "{}", float),
            Object::Boolean(bool) => write!(f, "{}", bool),
            Object::String(string) => write!(f, "\"{}\"", string),
            Object::Array(elements) => {
                let objs = elements
                    .borrow()
                    .iter()
                    .map(|obj| obj.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "[{elements}]", elements = objs)
            }
            Object::Null => write!(f, "null"),
            Object::Return(obj) => write!(f, "{}", obj),
            Object::Function(_params, _block, _env) => {
                // write!(f, "fun({}){}", params.join(", "), block)
                write!(f, "Function")
            }
            Object::Builtin(_) => write!(f, "Builtin Function"),
            Object::Hash(hash) => {
                let x = hash
                    .borrow()
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{{{map}}}", map = x)
            }
        }
    }
}
