use crate::eval::evaluator::EvalResult;
use crate::object::{EvalError, Object};
use std::cell::RefCell;
macro_rules! builtin {
    ($name:ident) => {
        Builtin {
            name: stringify!($name),
            builtin: Object::Builtin($name),
        }
    };
}

pub struct Builtin {
    name: &'static str,
    builtin: Object,
}

const BUILTINS: &[Builtin] = &[
    builtin!(len),
    builtin!(first),
    builtin!(last),
    builtin!(rest),
    builtin!(push),
    builtin!(print),
];

pub fn lookup(name: &str) -> Option<Object> {
    for x in BUILTINS.iter() {
        if x.name == name {
            return Some(x.builtin.clone());
        }
    }
    None
}

pub fn len(args: Vec<Object>) -> EvalResult {
    assert_argument_count(1, &args)?;
    let len = match &args[0] {
        Object::String(str) => str.len(),
        Object::Array(items) => items.borrow().len(),
        _ => return Err(EvalError::BuiltinUnSupportedArg("len".to_string(), args)),
    };
    Ok(Object::Integer(len as i64))
}

pub fn first(args: Vec<Object>) -> EvalResult {
    assert_argument_count(1, &args)?;
    let first = match &args[0] {
        Object::String(str) => str
            .chars()
            .next()
            .map_or(Object::Null, |c| Object::String(c.to_string())),
        Object::Array(items) => items.borrow().first().unwrap_or(&Object::Null).clone(),
        _ => return Err(EvalError::BuiltinUnSupportedArg("first".to_string(), args)),
    };
    Ok(first)
}

pub fn last(args: Vec<Object>) -> EvalResult {
    assert_argument_count(1, &args)?;
    let last = match &args[0] {
        Object::String(str) => str
            .chars()
            .last()
            .map_or(Object::Null, |c| Object::String(c.to_string())),
        Object::Array(items) => items.borrow().last().unwrap_or(&Object::Null).clone(),
        _ => return Err(EvalError::BuiltinUnSupportedArg("last".to_string(), args)),
    };
    Ok(last)
}

pub fn rest(args: Vec<Object>) -> EvalResult {
    assert_argument_count(1, &args)?;
    let rest = match &args[0] {
        Object::String(str) => Object::String(str[1..].to_string()),
        Object::Array(items) => {
            let x = items.borrow()[1..].to_vec();
            Object::Array(RefCell::new(x))
        }
        _ => return Err(EvalError::BuiltinUnSupportedArg("last".to_string(), args)),
    };
    Ok(rest)
}
//toto 现在数组暂时还是不可对原数组增减的
pub fn push(args: Vec<Object>) -> EvalResult {
    let push = match &args[0] {
        Object::Array(items) => {
            for x in args[1..].to_vec() {
                items.borrow_mut().push(x)
            }
            Object::Array(items.clone())
        }
        Object::String(str) => {
            let mut str = str.clone();
            for x in &args[1..] {
                str.push_str(&x.to_string())
            }
            Object::String(str)
        }
        _ => return Err(EvalError::BuiltinUnSupportedArg("push".to_string(), args)),
    };
    Ok(push)
}

pub fn print(args: Vec<Object>) -> EvalResult {
    args.iter()
        .map(|arg| arg.to_string())
        .for_each(|s| println!("{}", s));
    Ok(Object::Null)
}

fn assert_argument_count(expected: usize, args: &[Object]) -> EvalResult<()> {
    if expected != args.len() {
        Err(EvalError::BuiltinIncorrectArgNum(expected, args.len()))
    } else {
        Ok(())
    }
}
