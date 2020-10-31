use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result};
use std::rc::Rc;

use crate::compiler::code::{_print_instructions, Instructions, Opcode};
use crate::core::base::ast::{BinaryOperator, BlockStatement, Expression, UnaryOperator};
use crate::eval::evaluator::EvalResult;
use crate::object::environment::Environment;

pub mod builtins;
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
    CompiledFunction(Instructions, usize, usize),
    Builtin(BuiltinFunction),
    /// compiled function, free variables
    Closure(CompiledFunction, Vec<Rc<Object>>),
    Return(Box<Object>),
    Null,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    pub compiled_function: CompiledFunction,
    pub free_variables: Vec<Rc<Object>>,
}

impl Closure {
    pub fn new(compiled_function: CompiledFunction, free_variables: Vec<Rc<Object>>) -> Self {
        Self {
            compiled_function,
            free_variables,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CompiledFunction {
    pub insts: Instructions,
    pub num_locals: usize,
    pub num_parameters: usize,
}

impl CompiledFunction {
    pub fn new(insts: Instructions, num_locals: usize, num_parameters: usize) -> Self {
        Self {
            insts,
            num_locals,
            num_parameters,
        }
    }
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
            _ => Err(RuntimeError::UnsupportedHashKey(obj.clone())),
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
pub enum RuntimeError {
    StackNoElement,
    StackOverflow,
    ArrayOutOfBound { len: usize, index: usize },
    UnSupportedBinOperation(Opcode, Object, Object),
    UnSupportedBinOperator(Opcode),
    ByZero(Object, Object),

    UnSupportedUnOperation(Opcode, Object),
    UnSupportedIndexOperation(Object, Object),
    UnKnownOpCode(Opcode),

    CustomErrMsg(String),
    /// (expect, actual)
    WrongArgumentCount(usize, usize),
    NotFunction(Object),
    /*--------------------------*/
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

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            RuntimeError::TypeMismatch(op, l, r) => write!(
                f,
                "type mismatch: {} {} {}",
                l.type_name(),
                op,
                r.type_name()
            ),
            RuntimeError::UnknownUnaryOperator(op, ex) => {
                write!(f, "unknown operator: {}{}", op, ex.type_name())
            }
            RuntimeError::UnknownBinaryOperator(op, l, r) => write!(
                f,
                "unknown operator: {} {} {}",
                l.type_name(),
                op,
                r.type_name()
            ),
            RuntimeError::IdentifierNotFound(id) => write!(f, "identifier not found: {}", id),
            RuntimeError::NotCallable(fun) => write!(f, "not callable fun: {}", fun),
            RuntimeError::BuiltinUnSupportedArg(name, args) => write!(
                f,
                "argument to `{}` not supported, got {}",
                name,
                args.iter()
                    .map(|a| a.type_name())
                    .collect::<Vec<&str>>()
                    .join(", ")
            ),
            RuntimeError::BuiltinIncorrectArgNum(expected_num, actual_num) => write!(
                f,
                "wrong number of arguments. got {}, want {}",
                actual_num, expected_num
            ),
            RuntimeError::UnsupportedExpression(expr) => {
                write!(f, "unsupported expression: {}", expr)
            }
            RuntimeError::IndexUnsupported(left) => {
                write!(f, "index operator not support: {}", left)
            }
            RuntimeError::AssignUnsupported(left, val) => write!(
                f,
                "unsupported assign expression.{} can't assign to {}",
                val, left
            ),
            RuntimeError::UnsupportedHashKey(obj) => {
                write!(f, "can't used as a hash key: {}", obj.type_name())
            }
            RuntimeError::StackNoElement => write!(f, "stack is empty"),
            RuntimeError::StackOverflow => write!(f, "stack overflow"),
            RuntimeError::ArrayOutOfBound { len, index } => {
                write!(f, "array out of bound. index({}) >= len({})", index, len)
            }
            RuntimeError::UnSupportedBinOperation(op, l, r) => {
                write!(f, "unsupported binary operation: {} {} {}", l, op, r)
            }
            RuntimeError::UnSupportedBinOperator(op) => {
                write!(f, "unsupported binary operator: {}", op)
            }
            RuntimeError::ByZero(a, b) => write!(f, "by zero: {} / {}", a, b),
            RuntimeError::UnSupportedUnOperation(op, operand) => {
                write!(f, "unsupported unary operation: {} {}", op, operand)
            }
            RuntimeError::UnSupportedIndexOperation(arr, index) => {
                write!(f, "unsupported index operator: {}[{}]", arr, index)
            }
            RuntimeError::UnKnownOpCode(op) => write!(f, "unknown opcode: {}", op),
            RuntimeError::CustomErrMsg(err_msg) => write!(f, "{}", err_msg),
            RuntimeError::WrongArgumentCount(exp, act) => {
                write!(f, "wrong argument count: expected: {}, got: {}", exp, act)
            }
            RuntimeError::NotFunction(obj) => write!(f, "{} not a function", obj),
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
            Object::CompiledFunction(insts, _num_locals, _num_parameters) => {
                write!(f, "{}", _print_instructions(insts))
            }
            Object::Closure(cf, _free) => write!(f, "{}", _print_instructions(&cf.insts)),
        }
    }
}
