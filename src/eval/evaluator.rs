use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use crate::core::base::ast::{
    BinaryOperator, BlockStatement, Expression, Program, Statement, UnaryOperator,
};
use crate::object::{HashKey, Object, RuntimeError};
use crate::object::builtins::lookup;
use crate::object::environment::Environment;
use crate::object::Object::Boolean;

pub type EvalResult<T = Object> = Result<T, RuntimeError>;
pub type Env = Rc<RefCell<Environment>>;

/// # 程序求值
pub fn eval(program: &Program, env: Env) -> EvalResult {
    eval_statements(&program.statements, env)
}

/// ## 单条语句求值
fn eval_statement(statement: &Statement, env: Env) -> EvalResult {
    match statement {
        Statement::Let(name, expr) => {
            let val = eval_expression(expr, Rc::clone(&env))?;
            env.deref().borrow_mut().set(&name, val.clone());
            Ok(val)
        }
        Statement::Return(option) => {
            option
                .as_ref()
                .map_or(Ok(Object::Return(Box::new(Object::Null))), |expr| {
                    let object = eval_expression(expr, Rc::clone(&env))?;
                    Ok(Object::Return(Box::new(object)))
                })
        }
        Statement::Expression(expr) => eval_expression(expr, Rc::clone(&env)),
        Statement::Comment(comment) => Ok(Object::String(comment.to_string())),
    }
}

/// ## 多条语句求值
fn eval_statements(statements: &[Statement], env: Env) -> EvalResult {
    let mut result = Object::Null;
    for statement in statements {
        result = eval_statement(statement, Rc::clone(&env))?;
        if let Object::Return(obj) = result {
            return Ok(*obj);
        }
    }
    Ok(result)
}

/// ## 语句块求值
fn eval_block_statements(block: &BlockStatement, env: Env) -> EvalResult {
    let mut result = Object::Null;
    for statement in &block.statements {
        result = eval_statement(statement, Rc::clone(&env))?;
        if let Object::Return(_) = result {
            return Ok(result);
        }
    }
    Ok(result)
}

/// #表达式求值
fn eval_expression(expr: &Expression, env: Env) -> EvalResult {
    match expr {
        Expression::IntLiteral(int) => Ok(Object::Integer(*int)),
        Expression::BoolLiteral(bool) => Ok(Object::Boolean(*bool)),
        Expression::StringLiteral(str) => Ok(Object::String(str.clone())),

        Expression::Unary(op, expr) => {
            eval_unary_expression(op, eval_expression(expr, Rc::clone(&env))?)
        }
        Expression::Binary(op, left, right) => {
            eval_binary_expression(op, left, right, Rc::clone(&env))
        }

        Expression::If(cond, block, else_block) => {
            let bool_object = eval_expression(cond, Rc::clone(&env))?;
            let bool = eval_object_to_bool(bool_object);
            if bool {
                eval_block_statements(block, Rc::clone(&env))
            } else {
                else_block.as_ref().map_or(Ok(Object::Null), |else_block| {
                    eval_block_statements(else_block, Rc::clone(&env))
                })
            }
        }

        Expression::Identifier(id) => eval_identifier_expression(Rc::clone(&env), &id),
        Expression::FunctionLiteral(params, block) => Ok(Object::Function(
            params.clone(),
            block.clone(),
            Rc::clone(&env),
        )),
        Expression::Call(fun, params) => eval_call_expression(Rc::clone(&env), fun, params),
        Expression::ArrayLiteral(elements) => eval_array_literal(Rc::clone(&env), elements),
        Expression::Index(arr_expr, idx_expr) => {
            eval_array_index(Rc::clone(&env), arr_expr, idx_expr)
        }
        Expression::HashLiteral(pairs) => eval_hash_expression(Rc::clone(&env), pairs),
        _ => Err(RuntimeError::UnsupportedExpression(expr.clone())),
    }
}
/// ## map表达式求值
fn eval_hash_expression(env: Env, pairs_expr: &[(Expression, Expression)]) -> EvalResult<Object> {
    let mut pairs = HashMap::new();
    for (key_expr, val_expr) in pairs_expr {
        let key = eval_expression(key_expr, Rc::clone(&env))?;
        let val = eval_expression(val_expr, Rc::clone(&env))?;
        pairs.insert(HashKey::from_object(&key)?, val);
    }
    Ok(Object::Hash(RefCell::new(pairs)))
}

/// ## 数组/hash索引求值
fn eval_array_index(env: Env, obj_expr: &Expression, idx_expr: &Expression) -> EvalResult {
    let obj = eval_expression(obj_expr, Rc::clone(&env))?;
    let index = eval_expression(idx_expr, Rc::clone(&env))?;
    eval_index_expression(&obj, &index)
}
/// ## 索引表达式求值
fn eval_index_expression(obj: &Object, idx: &Object) -> EvalResult {
    match obj {
        Object::Array(items) => {
            if let Object::Integer(i) = *idx {
                if i >= items.borrow().len() as i64 || i < 0 {
                    return Ok(Object::Null);
                }
                let i = i as usize;
                let r = &items.borrow()[i];
                Ok(r.clone())
            } else {
                Err(RuntimeError::IndexUnsupported(obj.clone()))
            }
        }
        Object::Hash(pairs) => {
            let pairs = pairs.borrow();
            let key = &HashKey::from_object(idx)?;
            let r = pairs.get(key).unwrap_or(&Object::Null);
            Ok(r.clone())
        }
        _ => Err(RuntimeError::IndexUnsupported(obj.clone())),
    }
}

/// ## 数组字面量求值
fn eval_array_literal(env: Env, elements: &[Expression]) -> EvalResult {
    let mut array = vec![];
    for element in elements {
        let object = eval_expression(element, Rc::clone(&env))?;
        array.push(object);
    }
    Ok(Object::Array(RefCell::new(array)))
}

/// ##多条表达式求值
fn eval_expressions(exprs: &[Expression], env: Env) -> EvalResult<Vec<Object>> {
    let mut result = vec![];
    for expr in exprs {
        let obj = eval_expression(expr, Rc::clone(&env))?;
        result.push(obj);
    }
    Ok(result)
}

/// ## 函数表达式求值
fn apply_function(fun: Object, param_values: Vec<Object>) -> EvalResult {
    match fun {
        Object::Function(param_names, block, parent_env) => {
            let env = Rc::new(RefCell::new(Environment::extend(parent_env)));
            for (i, param) in param_names.iter().enumerate() {
                env.deref()
                    .borrow_mut()
                    .set(param, param_values.get(i).unwrap_or(&Object::Null).clone());
            }
            let evaluated = eval_block_statements(&block, Rc::clone(&env))?;
            match evaluated {
                Object::Return(ret) => Ok(*ret),
                _ => Ok(evaluated),
            }
        }
        Object::Builtin(builtin_fun) => builtin_fun(param_values),
        _ => Err(RuntimeError::NotCallable(fun)),
    }
}

/// ## 标识符表达式求值
fn eval_identifier_expression(env: Env, id: &str) -> EvalResult {
    //优先先去上下文中查找
    let option = env.as_ref().borrow().get(&id);
    if let Some(obj) = option.as_deref() {
        return Ok(obj.borrow().clone());
    }
    //内置函数中查找
    if let Some(builtin) = lookup(&id) {
        return Ok(builtin);
    }
    //否则报错
    Err(RuntimeError::IdentifierNotFound(id.to_string()))
}

/// ## 函数调用表达式求值
fn eval_call_expression(
    env: Env,
    fun: &Expression,
    params: &[Expression],
) -> Result<Object, RuntimeError> {
    let fun = eval_expression(&fun, Rc::clone(&env))?;
    let args = eval_expressions(params, Rc::clone(&env))?;
    apply_function(fun, args)
}
/// # 二元表达式求值
fn eval_binary_expression(
    operator: &BinaryOperator,
    left: &Expression,
    right: &Expression,
    env: Env,
) -> EvalResult {
    match left {
        //变量赋值
        Expression::Identifier(id) if operator == &BinaryOperator::Assign => {
            if env.deref().borrow().contains(&id) {
                let new_val = eval_expression(right, Rc::clone(&env))?;
                env.borrow_mut().set(&id, new_val.clone());
                Ok(new_val)
            } else {
                Err(RuntimeError::IdentifierNotFound(id.clone()))
            }
        }
        //数组/hash索引赋值
        Expression::Index(arr, key) if operator == &BinaryOperator::Assign => {
            //只能通过数组名索引来修改
            if let Expression::Identifier(obj_name) = arr.deref() {
                let option = env.deref().borrow().get(&obj_name);
                if let Some(obj) = option {
                    //是否存在
                    let ref_obj = obj.borrow();
                    match ref_obj.deref() {
                        Object::Array(items) => {
                            let index = eval_expression(key, Rc::clone(&env))?;
                            //索引是否为整数
                            if let Object::Integer(i) = index {
                                let i = i as usize;
                                if i >= items.borrow().len() {
                                    return Ok(Object::Null);
                                }
                                let val = eval_expression(right, Rc::clone(&env))?;
                                items.borrow_mut()[i] = val.clone();
                                return Ok(val);
                            }
                        }
                        Object::Hash(pairs) => {
                            let index = eval_expression(key, Rc::clone(&env))?;
                            let hash_key = &HashKey::from_object(&index)?;
                            let val = eval_expression(right, Rc::clone(&env))?;
                            let old_val = pairs
                                .borrow_mut()
                                .insert(hash_key.clone(), val)
                                .unwrap_or(Object::Null);
                            return Ok(old_val);
                        }
                        _ => {}
                    }
                }
            }
            Err(RuntimeError::AssignUnsupported(left.clone(), right.clone()))
        }
        //普通二元运算
        _ => {
            let left = eval_expression(left, Rc::clone(&env))?;
            let right = eval_expression(right, Rc::clone(&env))?;
            if let Object::Integer(left) = left {
                //整数运算
                if let Object::Integer(right) = right {
                    return eval_integer_binary_expression(operator, left, right);
                }
            } else if let Object::Boolean(left) = left {
                //布尔运算
                if let Object::Boolean(right) = right {
                    return eval_boolean_binary_expression(operator, left, right);
                }
            } else if let Object::String(left) = &left {
                //字符串运算
                if let Object::String(right) = &right {
                    return eval_string_binary_expression(operator, left, right);
                }
            }
            Err(RuntimeError::TypeMismatch(operator.clone(), left, right))
        }
    }
}

fn eval_string_binary_expression(operator: &BinaryOperator, left: &str, right: &str) -> EvalResult {
    match operator {
        BinaryOperator::Plus => {
            //字符串拼接
            let s = String::from(left) + right;
            Ok(Object::String(s))
        }
        BinaryOperator::Eq => Ok(Object::Boolean(left == right)),
        BinaryOperator::NotEq => Ok(Object::Boolean(left != right)),
        _ => Err(RuntimeError::UnknownBinaryOperator(
            operator.clone(),
            Object::String(left.to_string()),
            Object::String(right.to_string()),
        )),
    }
}

/// ## 整数二元表达式求值
fn eval_integer_binary_expression(operator: &BinaryOperator, left: i64, right: i64) -> EvalResult {
    match operator {
        BinaryOperator::Plus => Ok(Object::Integer(left + right)),
        BinaryOperator::Minus => Ok(Object::Integer(left - right)),
        BinaryOperator::Mul => Ok(Object::Integer(left * right)),
        BinaryOperator::Div => Ok(Object::Integer(left / right)),
        BinaryOperator::Gt => Ok(Object::Boolean(left > right)),
        BinaryOperator::Ge => Ok(Object::Boolean(left >= right)),
        BinaryOperator::Lt => Ok(Object::Boolean(left < right)),
        BinaryOperator::Le => Ok(Object::Boolean(left <= right)),
        BinaryOperator::Eq => Ok(Object::Boolean(left == right)),
        BinaryOperator::NotEq => Ok(Object::Boolean(left != right)),
        _ => Err(RuntimeError::UnknownBinaryOperator(
            operator.clone(),
            Object::Integer(left),
            Object::Integer(right),
        )),
    }
}

/// ## 布尔二元表达式求值
fn eval_boolean_binary_expression(
    operator: &BinaryOperator,
    left: bool,
    right: bool,
) -> EvalResult {
    match operator {
        BinaryOperator::Gt => Ok(Object::Boolean(left & !right)),
        BinaryOperator::Ge => Ok(Object::Boolean(left >= right)),
        BinaryOperator::Lt => Ok(Object::Boolean(!left & right)),
        BinaryOperator::Le => Ok(Object::Boolean(left <= right)),
        BinaryOperator::Eq => Ok(Object::Boolean(left == right)),
        BinaryOperator::NotEq => Ok(Object::Boolean(left != right)),
        _ => Err(RuntimeError::UnknownBinaryOperator(
            operator.clone(),
            Boolean(left),
            Boolean(right),
        )),
    }
}

/// ## object转bool
fn eval_object_to_bool(object: Object) -> bool {
    match object {
        Object::Integer(i) => i != 0,
        Object::Boolean(bool) => bool,
        Object::String(str) => !str.trim().is_empty(),
        Object::_Float(f) => f.ne(&0.0) && !f.is_nan(),
        Object::Null => false,
        _ => panic!("错误对象:{}，不能转换为bool", object),
    }
}

/// # 一元表达式求职
fn eval_unary_expression(operator: &UnaryOperator, operand: Object) -> EvalResult {
    match operator {
        UnaryOperator::Not => eval_not_operator_expression(operand),
        UnaryOperator::Neg => eval_neg_operator_expression(operand),
    }
}

/// ## 取非
fn eval_not_operator_expression(operand: Object) -> EvalResult {
    Ok(Object::Boolean(!eval_object_to_bool(operand)))
}

/// ## 取反
fn eval_neg_operator_expression(operand: Object) -> EvalResult {
    match operand {
        Object::Integer(i) => Ok(Object::Integer(-i)),
        _ => Err(RuntimeError::UnknownUnaryOperator(
            UnaryOperator::Neg,
            operand,
        )),
    }
}
