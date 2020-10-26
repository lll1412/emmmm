use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::compiler::code::{CompiledFunction, Opcode, read_operands};
use crate::object::{HashKey, Object};
use crate::vm::{FALSE, GLOBALS_SIZE, NULL, RCFrame, STACK_SIZE, TRUE, Vm, VmError, VmResult};
use crate::vm::frame::Frame;

impl Vm {
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
                    return Err(VmError::UnSupportedIndexOperation(
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
                    .map_err(|err| VmError::CustomErrMsg(err.to_string()))?;
                pairs.borrow_mut().insert(key, Object::clone(val));
            }
            //普通赋值
            _ => {
                let popped = self.pop_stack()?;
                self.set_global(global_index, popped)?;
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
            let key =
                HashKey::from_object(k).map_err(|err| VmError::CustomErrMsg(err.to_string()))?;
            hash.insert(key, Object::clone(v));
            i += 2;
        }
        self.sp -= hash_len;
        self.push_stack(Rc::new(Object::Hash(RefCell::new(hash))))
    }
    /// # 执行二元操作
    pub fn execute_binary_operation(&mut self, op: Opcode) -> VmResult {
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
                            return Err(VmError::ByZero(
                                Object::clone(&left),
                                Object::clone(&right),
                            ));
                        }
                        left_val / right_val
                    }
                    _ => return Err(VmError::UnSupportedBinOperator(op)),
                };
                Ok(Rc::new(Object::Integer(r)))
            }
            (Object::String(left_val), Object::String(right_val)) => {
                if let Opcode::Add = op {
                    Ok(Rc::new(Object::String(left_val.clone() + right_val)))
                } else {
                    Err(VmError::UnSupportedBinOperation(
                        op,
                        left.clone(),
                        right.clone(),
                    ))
                }
            }
            _ => Err(VmError::UnSupportedBinOperation(
                op,
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
                .map_err(|err| VmError::CustomErrMsg(err.to_string()))?;
            let value = pairs.borrow().get(&key).cloned().unwrap_or(NULL);
            return Ok(Rc::new(value));
        }
        Err(VmError::UnSupportedIndexOperation(
            obj.clone(),
            index.clone(),
        ))
    }
    /// # 执行比较操作
    pub fn execute_comparison_operation(&mut self, op: Opcode) -> VmResult {
        let right = self.pop_stack()?;
        let left = self.pop_stack()?;
        if let Object::Integer(left) = *left {
            if let Object::Integer(right) = *right {
                let bool = match op {
                    Opcode::GreaterThan => left > right,
                    Opcode::LessThan => left < right,
                    Opcode::Equal => left == right,
                    Opcode::NotEqual => left != right,
                    _ => return Err(VmError::UnSupportedBinOperator(op)),
                };
                return Ok(if bool { Rc::new(TRUE) } else { Rc::new(FALSE) });
            }
        }
        let r = match op {
            Opcode::Equal => {
                if left == right {
                    TRUE
                } else {
                    FALSE
                }
            }
            Opcode::NotEqual => {
                if left != right {
                    TRUE
                } else {
                    FALSE
                }
            }
            _ => {
                return Err(VmError::UnSupportedBinOperation(
                    op,
                    Object::clone(&left),
                    Object::clone(&right),
                ));
            }
        };
        Ok(Rc::new(r))
    }
    /// 函数调用
    pub fn call_function(&mut self, arg_nums: usize) -> VmResult<()> {
        self.sp -= arg_nums;
        let cf = &*self.stack[self.sp - 1]; //往回跳过参数个数位置, 当前位置是函数
        if let Object::CompiledFunction(insts, num_locals, num_parameters) = cf {
            if arg_nums != *num_parameters {
                return Err(VmError::WrongArgumentCount(*num_parameters, arg_nums));
            }
            let frame = Frame::new(
                CompiledFunction::new(insts.clone(), *num_locals, arg_nums),
                self.sp,
            );
            //Equivalent to
            self.sp += num_locals;
            // self.sp = frame.base_pointer + num_locals;
            self.push_frame(frame); //进入函数内部（下一帧）
            Ok(())
        } else if let Object::Builtin(builtin_fun) = cf {
            //内置函数
            let mut v = vec![];
            for i in 0..arg_nums {
                let rc = &self.stack[self.sp + i];
                v.push(Object::clone(rc));
            }
            let r = builtin_fun(v).map_err(|e| VmError::CustomErrMsg(e.to_string()))?;
            self.push_stack(Rc::new(r))?;
            Ok(())
        } else {
            Err(VmError::CustomErrMsg("calling non-function".to_string()))
        }
    }

    /// # 计算该指令操作数的长度，方便指令指针自增
    pub fn increment_num(&self, op: Opcode) -> usize {
        op.definition().operand_width.iter().sum()
    }
    /// # 读取一个无符号整数，并返回字节长度
    pub fn read_usize(&self, op_code: Opcode, ip: usize) -> (usize, usize) {
        let (operands, n) = read_operands(
            op_code.definition(),
            &self.current_frame().instructions()[ip..],
        );
        (operands[0], n)
    }
    /// # 压入栈中
    pub fn push_stack(&mut self, object: Rc<Object>) -> VmResult<()> {
        if self.sp == STACK_SIZE {
            Err(VmError::StackOverflow)
        } else {
            if self.sp >= self.stack.len() {
                self.stack.push(object);
            } else {
                self.stack.insert(self.sp, object);
            }
            self.sp += 1;
            Ok(())
        }
    }
    /// # 弹出栈顶元素
    pub fn pop_stack(&mut self) -> VmResult {
        if self.sp == 0 {
            Err(VmError::StackNoElement)
        } else {
            let o = &self.stack[self.sp - 1];
            self.sp -= 1;
            Ok(o.clone())
        }
    }

    /// # 存入全局变量
    pub fn set_global(&mut self, global_index: usize, global: Rc<Object>) -> VmResult<()> {
        if global_index == GLOBALS_SIZE {
            Err(VmError::StackOverflow)
        } else {
            if global_index == self.globals.borrow().len() {
                self.globals.borrow_mut().push(global);
            } else {
                self.globals.borrow_mut()[global_index] = global;
            }
            Ok(())
        }
    }
    /// # 取出全局变量
    pub fn get_global(&self, global_index: usize) -> VmResult {
        let globals = self.globals.borrow();
        let option = globals.get(global_index);
        if let Some(object) = option {
            Ok(object.clone())
        } else {
            Err(VmError::CustomErrMsg(format!(
                "global has not such element. index: {}",
                global_index
            )))
        }
    }
    pub fn get_builtin(&self, builtin_index: usize) -> VmResult {
        let option = self.builtins.get(builtin_index);
        if let Some(builtin_fun) = option {
            Ok(builtin_fun.clone())
        } else {
            Err(VmError::CustomErrMsg(format!(
                "builtin has not such element. index: {}",
                builtin_index
            )))
        }
    }
    /// # 最后弹出栈顶的元素
    pub fn last_popped_stack_element(&self) -> VmResult {
        if self.sp >= self.stack.len() {
            Err(VmError::ArrayOutOfBound {
                len: self.stack.len(),
                index: self.sp,
            })
        } else {
            let object = &self.stack[self.sp];
            Ok(object.clone())
        }
    }
    pub fn get_const_object(&self, index: usize) -> Object {
        self.constants.borrow()[index].to_object()
    }
    pub fn current_frame_ip_inc(&mut self, n: usize) -> usize {
        self.frames.last_mut().expect("empty frames").ip += n;
        self.current_frame().ip
    }
    pub fn set_current_frame_ip(&mut self, n: usize) {
        self.frames.last_mut().expect("empty frames").ip = n;
    }
    pub fn current_frame_instructions_len(&self) -> usize {
        self.current_frame().instructions().len()
    }
    pub fn current_frame(&self) -> &RCFrame {
        self.frames.last().expect("empty frames")
    }
    pub fn current_frame_bp(&self) -> usize {
        self.current_frame().base_pointer
    }
    pub fn push_frame(&mut self, frame: RCFrame) {
        self.frames.push(frame);
    }
    pub fn pop_frame(&mut self) -> RCFrame {
        self.frames.pop().expect("empty frames")
    }
}
