use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::compiler::code::{Opcode, read_operands};
use crate::object::{HashKey, Object, RuntimeError};
use crate::object::builtins::BUILTINS;
use crate::vm::{FALSE, NULL, RCFrame, TRUE, Vm, VmResult};
use crate::vm::frame::Frame;

impl Vm {
    /// # 执行非运算
    pub fn execute_not_expression(&mut self, value: &Object) -> VmResult<()> {
        if *value == TRUE {
            self.push_stack(self.bool_cache_false.clone())?;
        } else if *value == FALSE {
            self.push_stack(self.bool_cache_true.clone())?;
        } else if *value == NULL {
            self.push_stack(self.null_cache.clone())?;
        } else {
            return Err(RuntimeError::UnSupportedUnOperation(
                Opcode::Not,
                (value).clone(),
            ));
        }
        Ok(())
    }
    /// # 执行赋值操作
    pub fn execute_assign_operation(&mut self, global_index: usize) -> VmResult<()> {
        let obj = &self.get_global(global_index)?;
        match obj.as_ref() {
            //数组赋值
            Object::Array(items) => {
                let val = &self.pop_stack()?;
                let index = &self.pop_stack()?;
                if let Object::Integer(i) = index.as_ref() {
                    items.borrow_mut()[*i as usize] = Object::clone(val);
                } else {
                    return Err(RuntimeError::UnSupportedIndexOperation(
                        Object::clone(obj),
                        Object::clone(index),
                    ));
                }
            }
            // Hash赋值
            Object::Hash(pairs) => {
                let val = &self.pop_stack()?;
                let index = &self.pop_stack()?;
                let key = HashKey::from_object(index)
                    .map_err(|err| RuntimeError::CustomErrMsg(err.to_string()))?;
                pairs.borrow_mut().insert(key, Object::clone(val));
            }
            //普通赋值
            _ => {
                let popped = self.pop_stack()?;
                self.set_global(global_index, popped);
            }
        }
        Ok(())
    }
    /// # 创建数组
    pub fn build_array(&mut self, arr_len: usize) -> VmResult<()> {
        let mut arr = vec![];
        for i in self.sp - arr_len..self.sp {
            let el = &self.stack[i];
            arr.push(Object::clone(el));
        }
        self.sp -= arr_len;
        self.push_stack(Rc::new(Object::Array(RefCell::new(arr))))
    }
    /// # 创建Hash
    pub fn build_hash(&mut self, hash_len: usize) -> VmResult<()> {
        let mut hash = HashMap::new();
        let mut i = self.sp - 2 * hash_len;
        while i < self.sp {
            let k = &self.stack[i];
            let v = &self.stack[i + 1];
            let key = HashKey::from_object(k)
                .map_err(|err| RuntimeError::CustomErrMsg(err.to_string()))?;
            hash.insert(key, Object::clone(v));
            i += 2;
        }
        self.sp -= hash_len;
        self.push_stack(Rc::new(Object::Hash(RefCell::new(hash))))
    }
    /// # 执行二元操作
    #[inline]
    pub fn execute_binary_operation(&mut self, op: &Opcode) -> VmResult {
        let right = &*self.pop_stack()?;
        let left = &*self.pop_stack()?;
        match (left, right) {
            (Object::Integer(left_val), Object::Integer(right_val)) => {
                let r = match op {
                    Opcode::Add => left_val + right_val,
                    Opcode::Sub => left_val - right_val,
                    Opcode::Mul => left_val * right_val,
                    Opcode::Div => {
                        if right_val == &0 {
                            return Err(RuntimeError::ByZero(
                                Object::clone(&left),
                                Object::clone(&right),
                            ));
                        }
                        left_val / right_val
                    }
                    _ => return Err(RuntimeError::UnSupportedBinOperator(op.clone())),
                };
                match self.int_cache.get(r as usize) {
                    None => Ok(Rc::new(Object::Integer(r))),
                    Some(v) => Ok(v.clone()),
                }
            }
            (Object::String(left_val), Object::String(right_val)) => {
                if let Opcode::Add = op {
                    Ok(Rc::new(Object::String(left_val.clone() + right_val)))
                } else {
                    Err(RuntimeError::UnSupportedBinOperation(
                        op.clone(),
                        left.clone(),
                        right.clone(),
                    ))
                }
            }
            _ => Err(RuntimeError::UnSupportedBinOperation(
                op.clone(),
                left.clone(),
                right.clone(),
            )),
        }
    }
    /// # 执行索引操作
    pub fn execute_index_operation(&self, obj: &Object, index: &Object) -> VmResult {
        if let Object::Array(items) = obj {
            if let Object::Integer(index) = index {
                let value = items.borrow().get(*index as usize).cloned().unwrap_or(NULL);
                return Ok(Rc::new(value));
            }
        } else if let Object::Hash(pairs) = obj {
            let key = HashKey::from_object(index)
                .map_err(|err| RuntimeError::CustomErrMsg(err.to_string()))?;
            let value = pairs.borrow().get(&key).cloned().unwrap_or(NULL);
            return Ok(Rc::new(value));
        }
        Err(RuntimeError::UnSupportedIndexOperation(
            obj.clone(),
            index.clone(),
        ))
    }
    /// # 执行比较操作
    pub fn execute_comparison_operation(&mut self, op: &Opcode) -> VmResult {
        let right = self.pop_stack()?;
        let left = self.pop_stack()?;
        if let (Object::Integer(left), Object::Integer(right)) = (left.as_ref(), right.as_ref()) {
            let bool = match op {
                Opcode::GreaterThan => left > right,
                Opcode::LessThan => left < right,
                Opcode::Equal => left == right,
                Opcode::NotEqual => left != right,
                _ => return Err(RuntimeError::UnSupportedBinOperator(op.clone())),
            };
            return Ok(self.get_bool_from_cache(bool));
        }
        match op {
            Opcode::Equal => Ok(self.get_bool_from_cache(left == right)),
            Opcode::NotEqual => Ok(self.get_bool_from_cache(left != right)),
            _ => Err(RuntimeError::UnSupportedBinOperation(
                op.clone(),
                Object::clone(&left),
                Object::clone(&right),
            )),
        }
    }
    pub fn get_bool_from_cache(&self, bool: bool) -> Rc<Object> {
        if bool {
            self.bool_cache_true.clone()
        } else {
            self.bool_cache_false.clone()
        }
    }
    /// 函数调用
    #[inline]
    pub fn call_function(&mut self, arg_nums: usize) -> VmResult<()> {
        self.sp -= arg_nums;
        let callee = &*self.stack[self.sp - 1]; //往回跳过参数个数位置, 当前位置是函数
        match callee {
            Object::Closure(closure) => {
                if arg_nums != closure.compiled_function.num_parameters {
                    return Err(RuntimeError::WrongArgumentCount(
                        closure.compiled_function.num_parameters,
                        arg_nums,
                    ));
                }
                let num_locals = closure.compiled_function.num_locals;
                let frame = Frame::new(closure.clone(), self.sp);
                // Equivalent to
                // self.sp += closure.compiled_function.num_locals;
                self.sp = frame.base_pointer + num_locals;
                self.push_frame(frame); //进入函数内部（下一帧）
            }
            Object::Builtin(builtin_fun) => {
                //内置函数
                let mut v = vec![];
                for i in 0..arg_nums {
                    let rc = &self.stack[self.sp + i];
                    v.push(Object::clone(rc));
                }
                let r = builtin_fun(v).map_err(|e| RuntimeError::CustomErrMsg(e.to_string()))?;
                self.push_stack(Rc::new(r))?;
            }
            _ => {
                return Err(RuntimeError::CustomErrMsg(
                    "calling non-function".to_string(),
                ))
            }
        }
        Ok(())
    }

    /// # 计算该指令操作数的长度，方便指令指针自增
    pub fn _increment_num(&self, op: Opcode) -> usize {
        op.definition().operand_width.iter().sum()
    }
    /// # 读取一个无符号整数，并返回字节长度

    pub fn read_usize(&self, op_code: Opcode, ip: usize) -> (usize, usize) {
        let (operands, n) = read_operands(
            &op_code.definition(),
            &self.current_frame().instructions()[ip..],
        );
        (operands[0], n)
    }
    #[inline]
    pub fn read_u16(&self, insts: &[u8], start: usize) -> usize {
        u16::from_be_bytes([insts[start], insts[start + 1]]) as usize
    }
    #[inline]
    pub fn _read_u8(&self, insts: &[u8], start: usize) -> usize {
        insts[start] as usize
    }
    /// # 压入栈中
    pub fn push_stack(&mut self, object: Rc<Object>) -> VmResult<()> {
        if self.sp == self.stack.len() {
            self.stack.push(object);
        } else {
            //之前是insert方法，换索引赋值速度快了很多
            self.stack[self.sp] = object;
        }
        self.sp += 1;
        Ok(())
    }
    /// # 弹出栈顶元素
    #[inline]
    pub fn pop_stack(&mut self) -> VmResult {
        let o = &self.stack[self.sp - 1];
        self.sp -= 1;
        Ok(o.clone())
    }

    /// # 存入全局变量
    pub fn set_global(&mut self, global_index: usize, global: Rc<Object>) {
        let mut globals = self.globals.borrow_mut();
        if global_index == globals.len() {
            globals.push(global);
        } else {
            globals[global_index] = global;
        }
    }
    /// # 取出全局变量
    pub fn get_global(&self, global_index: usize) -> VmResult {
        let globals = self.globals.borrow();
        let option = &globals[global_index];
        Ok(option.clone())
    }
    pub fn get_builtin(&self, builtin_index: usize) -> VmResult {
        let builtin_fun = &BUILTINS[builtin_index];
        Ok(Rc::new(builtin_fun.builtin.clone()))
    }
    /// # 最后弹出栈顶的元素
    pub fn last_popped_stack_element(&self) -> VmResult {
        if self.sp >= self.stack.len() {
            Err(RuntimeError::ArrayOutOfBound {
                len: self.stack.len(),
                index: self.sp,
            })
        } else {
            let object = &self.stack[self.sp];
            Ok(object.clone())
        }
    }
    pub fn get_const_object(&self, index: usize) -> Rc<Object> {
        self.constants.borrow()[index].clone()
    }
    #[inline]
    pub fn current_frame_ip_inc(&mut self, n: usize) {
        self.frames.last_mut().unwrap().ip += n;
    }
    pub fn _current_frame_inc_and_get(&mut self, n: usize) -> usize {
        self.frames.last_mut().unwrap().ip += n;
        self.current_frame().ip
    }
    pub fn _set_current_frame_ip(&mut self, n: usize) {
        self.frames.last_mut().unwrap().ip = n;
    }
    pub fn current_frame(&self) -> &RCFrame {
        &self.frames.last().unwrap()
    }
    pub fn push_frame(&mut self, frame: RCFrame) {
        self.frames.push(frame);
    }
    pub fn pop_frame(&mut self) -> RCFrame {
        self.frames.pop().unwrap()
    }
}
