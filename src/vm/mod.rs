use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::compiler::code::{read_operands, Instructions, OPS};
use crate::object::builtins::BUILTINS;
use crate::object::HashKey;
use crate::{
    compiler::{code::Opcode, ByteCode, Constants},
    object::{Closure, CompiledFunction, Object, RuntimeError},
    vm::frame::Frame,
};

mod frame;
mod test;

pub type VmResult<T = Rc<Object>> = Result<T, RuntimeError>;
// pub type Globals = Rc<RefCell<Vec<Rc<Object>>>>;
pub type Globals = Vec<Rc<Object>>;
pub type Frames = Vec<Frame>;
pub type Stack = Vec<Rc<Object>>;

const GLOBALS_SIZE: usize = 0xFFFF;
const STACK_SIZE: usize = 2048;
const MAX_FRAMES: usize = 1024;
const MAX_INT_CACHE: usize = 128;

pub const TRUE: Object = Object::Boolean(true);
pub const FALSE: Object = Object::Boolean(false);
pub const NULL: Object = Object::Null;

#[derive(Debug)]
pub struct Vm {
    // 常量池
    constants: Constants,
    // 操作数栈
    stack: Stack,
    // stack pointer
    sp: usize,
    // 全局变量
    globals: Globals,
    //栈帧
    frames: Frames,
    //缓存
    int_cache: Stack,
    bool_cache_true: Rc<Object>,
    bool_cache_false: Rc<Object>,
    null_cache: Rc<Object>,
}

impl Vm {
    pub fn new(byte_code: ByteCode) -> Self {
        // let globals = create_rc_ref_cell(Vec::with_capacity(GLOBALS_SIZE));
        Vm::with_global_store(byte_code, Vec::with_capacity(GLOBALS_SIZE))
    }
    pub fn with_global_store(byte_code: ByteCode, globals: Globals) -> Self {
        //
        let mut stack = Vec::with_capacity(STACK_SIZE);
        let null_cache = Rc::new(NULL);
        for _ in 0..STACK_SIZE {
            stack.push(null_cache.clone())
        }
        //
        let mut int_cache = Vec::with_capacity(MAX_INT_CACHE);
        for i in 0..MAX_INT_CACHE {
            int_cache.push(Rc::new(Object::Integer(i as i64)))
        }
        //
        let main_fn = CompiledFunction::new(Rc::new(byte_code.instructions), 0, 0);
        let main_closure = Closure::new(main_fn, vec![]);
        let main_frame = Frame::new(Rc::new(Object::Closure(main_closure)), 0);
        let mut frames = Vec::with_capacity(MAX_FRAMES);
        frames.push(main_frame);
        Self {
            constants: byte_code.constants,
            stack,
            sp: 0,
            globals,
            frames,
            int_cache,
            bool_cache_true: Rc::new(TRUE),
            bool_cache_false: Rc::new(FALSE),
            null_cache,
        }
    }
    pub fn run(&mut self) -> VmResult {
        // let mut _time_recorder = crate::TimeRecorder::_new();
        // ip means instruction_pointer
        while self.current_frame().ip < self.current_frame().instructions().len() {
            let frame = self.frames.last_mut().unwrap();
            let ins = frame.instructions();
            // let op_code = Opcode::from_byte(ins[frame.ip]).unwrap();
            let op_code = OPS[ins[frame.ip] as usize];
            //从操作码后面一个位置开始
            frame.ip += 1;
            let ip = frame.ip;
            match op_code {
                Opcode::Constant => {
                    let const_index = self.read_u16(&ins, ip);
                    // let constant = self.constants[const_index].clone();
                    self.push_stack(self.constants[const_index].clone());
                    self.current_frame_ip_inc(2);
                }
                Opcode::ConstantOne => {
                    let const_index = ins[ip] as usize;
                    // let constant = self.constants[const_index].clone();
                    self.push_stack(self.constants[const_index].clone());
                    self.frames.last_mut().unwrap().ip += 1;
                }
                Opcode::Constant0 => {
                    // let constant = self.constants[0].clone();
                    self.push_stack(self.constants[0].clone());
                }
                Opcode::Constant1 => {
                    // let constant = self.constants[1].clone();
                    self.push_stack(self.constants[1].clone());
                }
                Opcode::Constant2 => {
                    // let constant = self.constants[2].clone();
                    self.push_stack(self.constants[2].clone());
                }
                Opcode::Constant3 => {
                    // let constant = self.constants[3].clone();
                    self.push_stack(self.constants[3].clone());
                }
                Opcode::Constant4 => {
                    // let constant = self.constants[4].clone();
                    self.push_stack(self.constants[4].clone());
                }

                Opcode::Array => {
                    let (arr_len, n) = self.read_usize(op_code, ip);
                    self.build_array(arr_len);
                    self.current_frame_ip_inc(n);
                }

                Opcode::Hash => {
                    let (hash_len, n) = self.read_usize(op_code, ip);
                    self.build_hash(hash_len)?;
                    self.current_frame_ip_inc(n);
                }

                Opcode::Index => {
                    let index = self.pop_stack();
                    let obj = self.pop_stack();
                    let result = self.execute_index_operation(&obj, &index)?;
                    self.push_stack(result);
                }

                Opcode::Add => {
                    self.execute_add_operation()?;
                }
                Opcode::Sub | Opcode::Mul | Opcode::Div => {
                    self.execute_binary_operation(&op_code)?;
                }

                Opcode::Pop => {
                    if self.sp > 0 {
                        self.sp -= 1;
                    }
                }

                Opcode::True => {
                    self.push_stack(self.bool_cache_true.clone());
                }
                Opcode::False => {
                    self.push_stack(self.bool_cache_false.clone());
                }

                Opcode::Equal
                | Opcode::NotEqual
                | Opcode::GreaterThan
                | Opcode::LessThan
                | Opcode::GreaterEq
                | Opcode::LessEq => {
                    let bool_object = self.execute_comparison_operation(&op_code)?;
                    self.push_stack(bool_object);
                }

                Opcode::Neg => {
                    let value = self.pop_stack();
                    if let Object::Integer(val) = *value {
                        self.push_stack(Rc::new(Object::Integer(-val)));
                    } else {
                        return Err(RuntimeError::UnSupportedUnOperation(
                            op_code,
                            (*value).clone(),
                        ));
                    }
                }
                Opcode::Not => {
                    let value = self.pop_stack();
                    self.execute_not_expression(&value)?;
                }

                Opcode::JumpAlways => {
                    self.frames.last_mut().unwrap().ip = self.read_u16(&ins, ip);
                }
                Opcode::JumpIfNotLess => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();
                    match (&*left, &*right) {
                        (Object::Integer(l), Object::Integer(r)) => {
                            self.jump_if(l < r, &ins, ip);
                        }
                        _ => {
                            return Err(RuntimeError::CustomErrMsg(
                                "unsupported compare".to_string(),
                            ));
                        }
                    }
                }
                Opcode::JumpIfNotTruthy => {
                    let is_truthy = self.pop_stack();
                    if let Object::Boolean(truthy) = *is_truthy {
                        self.jump_if(truthy, &ins, ip);
                    }
                }

                Opcode::Null => {
                    self.push_stack(self.null_cache.clone());
                    self.sp -= 1;
                }
                // set global
                Opcode::SetGlobal => {
                    let global_index = self.read_u16(&ins, ip);
                    self.execute_assign_operation_or_pop_and_set_global(global_index, false)?;
                    self.frames.last_mut().unwrap().ip += 2;
                }
                Opcode::SetGlobal0 => {
                    self.execute_assign_operation_or_pop_and_set_global(0, false)?;
                }
                Opcode::SetGlobal1 => {
                    self.execute_assign_operation_or_pop_and_set_global(1, false)?;
                }
                Opcode::SetGlobal2 => {
                    self.execute_assign_operation_or_pop_and_set_global(2, false)?;
                }
                Opcode::SetGlobal3 => {
                    self.execute_assign_operation_or_pop_and_set_global(3, false)?;
                }
                Opcode::SetGlobal4 => {
                    self.execute_assign_operation_or_pop_and_set_global(4, false)?;
                }
                // get global
                Opcode::GetGlobal => {
                    let global_index = self.read_u16(&ins, ip);
                    self.get_global_and_push(global_index);
                    self.current_frame_ip_inc(2);
                }
                Opcode::GetGlobal0 => {
                    // self.get_global_and_push(0);
                    let obj = self.globals[0].clone();
                    self.push_stack(obj);
                }
                Opcode::GetGlobal1 => {
                    self.get_global_and_push(1);
                }
                Opcode::GetGlobal2 => {
                    self.get_global_and_push(2);
                }
                Opcode::GetGlobal3 => {
                    self.get_global_and_push(3);
                }
                Opcode::GetGlobal4 => {
                    self.get_global_and_push(4);
                }
                // set local
                Opcode::SetLocal => {
                    frame.ip += 2;
                    self.pop_and_set_local(ins[ip] as usize);
                }
                Opcode::SetLocal0 => self.pop_and_set_local(0),
                Opcode::SetLocal1 => self.pop_and_set_local(1),
                Opcode::SetLocal2 => self.pop_and_set_local(2),
                Opcode::SetLocal3 => self.pop_and_set_local(3),
                Opcode::SetLocal4 => self.pop_and_set_local(4),
                // get local
                Opcode::GetLocal => {
                    frame.ip += 1;
                    let local_index = ins[ip] as usize;
                    self.get_local_and_push(local_index);
                }
                Opcode::GetLocal0 => {
                    let frame = self.frames.last().unwrap();
                    let object = self.stack[frame.base_pointer].clone();
                    self.push_stack(object)
                }
                Opcode::GetLocal1 => {
                    self.get_local_and_push(1);
                }
                Opcode::GetLocal2 => {
                    self.get_local_and_push(2);
                }
                Opcode::GetLocal3 => {
                    self.get_local_and_push(3);
                }
                Opcode::GetLocal4 => {
                    self.get_local_and_push(4);
                }

                Opcode::GetBuiltin => {
                    let builtin = self.get_builtin(ins[ip] as usize)?;
                    self.push_stack(builtin);
                    self.current_frame_ip_inc(1);
                }

                Opcode::Closure => {
                    //读取函数索引
                    let function_index = self.read_u16(&ins, ip);
                    //读取自由变量个数
                    let free_num = ins[ip + 2] as usize;
                    let func_object = &self.get_const_object(function_index);
                    if let Object::CompiledFunction(compiled_function) = func_object.as_ref() {
                        //往前free_num个都是free_variable
                        let frees = self.stack[self.sp - free_num..self.sp].to_vec();
                        let closure =
                            Object::Closure(Closure::new(compiled_function.clone(), frees));
                        self.push_stack(Rc::new(closure));
                    } else {
                        return Err(RuntimeError::NotFunction(Object::clone(func_object)));
                    };
                    self.current_frame_ip_inc(3);
                }
                Opcode::GetFree => {
                    let free_index = ins[ip] as usize;
                    self.push_stack(self.current_frame().get_free(free_index));
                    self.current_frame_ip_inc(1);
                }
                Opcode::CurrentClosure => {
                    let current_closure = self.current_frame().closure.clone();
                    self.push_stack(current_closure);
                }

                Opcode::Call => {
                    let arg_nums = ins[ip] as usize;
                    self.current_frame_ip_inc(1);
                    self.call_function(arg_nums)?;
                }

                Opcode::ReturnValue => {
                    let return_value = self.stack[self.sp - 1].clone();
                    // self.sp -= 1;
                    // let return_value = self.pop_stack(); //pop ret_val
                    let base_pointer = self.frames.pop().unwrap().base_pointer; // quit cur env
                    self.sp = base_pointer - 1;
                    self.push_stack(return_value); //push ret_val
                }
                Opcode::Return => {
                    let base_pointer = self.pop_frame().base_pointer; // quit cur env
                    self.sp = base_pointer - 1;
                    self.push_stack(self.null_cache.clone());
                }
                _ => return Err(RuntimeError::UnKnownOpCode(op_code)),
            }
            // _time_recorder._tick(op_code);
        }
        // _time_recorder._print_sorted_record();
        self.last_popped_stack_element()
    }
}

impl Vm {
    pub fn jump_if(&mut self, truthy: bool, ins: &Instructions, ip: usize) {
        if truthy {
            self.frames.last_mut().unwrap().ip += 2;
        } else {
            self.frames.last_mut().unwrap().ip = self.read_u16(ins, ip);
        }
    }
    /// # 执行非运算
    pub fn execute_not_expression(&mut self, value: &Object) -> VmResult<()> {
        if *value == TRUE {
            self.push_stack(self.bool_cache_false.clone());
        } else if *value == FALSE {
            self.push_stack(self.bool_cache_true.clone());
        } else if *value == NULL {
            self.push_stack(self.null_cache.clone());
        } else {
            return Err(RuntimeError::UnSupportedUnOperation(
                Opcode::Not,
                (value).clone(),
            ));
        }
        Ok(())
    }
    /// # 执行赋值操作
    pub fn execute_assign_operation_or_pop_and_set_global(
        &mut self,
        index: usize,
        is_local: bool,
    ) -> VmResult<()> {
        let opt = if is_local {
            self.get_local(index)
        } else {
            self.get_global(index)
        };
        match opt {
            None => {
                //声明
                if is_local {
                    self.pop_and_set_local(index);
                } else {
                    self.pop_and_set_global(index);
                }
            }
            Some(obj) => {
                //赋值
                match obj.as_ref() {
                    //数组赋值
                    Object::Array(items) => {
                        let val = self.pop_stack();
                        let index = self.pop_stack();
                        if let Object::Integer(i) = index.as_ref() {
                            items.borrow_mut()[*i as usize] = Object::clone(&val);
                        } else {
                            return Err(RuntimeError::UnSupportedIndexOperation(
                                Object::clone(&obj),
                                Object::clone(&index),
                            ));
                        }
                    }
                    // // Hash赋值
                    Object::Hash(pairs) => {
                        let val = self.pop_stack();
                        let index = self.pop_stack();
                        let key = HashKey::from_object(&index)?;
                        pairs.borrow_mut().insert(key, Object::clone(&val));
                    }
                    //普通赋值
                    _ => {
                        if is_local {
                            self.pop_and_set_local(index);
                        } else {
                            self.pop_and_set_global(index);
                        }
                    }
                }
            }
        }
        Ok(())
    }
    /// # 创建数组
    pub fn build_array(&mut self, arr_len: usize) {
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
            let key = HashKey::from_object(k)?;
            hash.insert(key, Object::clone(v));
            i += 2;
        }
        self.sp -= hash_len;
        self.push_stack(Rc::new(Object::Hash(RefCell::new(hash))));
        Ok(())
    }
    /// # 执行二元操作
    // #[inline]
    pub fn execute_add_operation(&mut self) -> VmResult<()> {
        let right = self.pop_stack();
        let left = self.pop_stack();
        let result = match (left.as_ref(), right.as_ref()) {
            (Object::Integer(left_val), Object::Integer(right_val)) => {
                let r = left_val + right_val;
                self.int_cache
                    .get(r as usize)
                    .cloned()
                    .unwrap_or_else(|| Rc::new(Object::Integer(r)))
            }
            (Object::String(left_val), Object::String(right_val)) => {
                Rc::new(Object::String(left_val.clone() + right_val))
            }
            (Object::Integer(left_val), Object::String(right_val)) => {
                Rc::new(Object::String(left_val.to_string() + right_val))
            }
            (Object::String(left_val), Object::Integer(right_val)) => {
                Rc::new(Object::String(left_val.clone() + &right_val.to_string()))
            }
            _ => {
                return Err(RuntimeError::UnSupportedBinOperation(
                    Opcode::Add,
                    Object::clone(&left),
                    Object::clone(&right),
                ));
            }
        };
        self.push_stack(result);
        Ok(())
    }
    pub fn execute_binary_operation(&mut self, op: &Opcode) -> VmResult<()> {
        let right = &*self.pop_stack();
        let left = &*self.pop_stack();
        let result = match (left, right) {
            (Object::Integer(left_val), Object::Integer(right_val)) => {
                let r = match op {
                    Opcode::Sub => left_val - right_val,
                    Opcode::Mul => left_val * right_val,
                    Opcode::Div => {
                        if right_val == &0 {
                            return Err(RuntimeError::ByZero(
                                Object::clone(left),
                                Object::clone(right),
                            ));
                        }
                        left_val / right_val
                    }
                    _ => return Err(RuntimeError::UnSupportedBinOperator(*op)),
                };
                self.int_cache
                    .get(r as usize)
                    .cloned()
                    .unwrap_or_else(|| Rc::new(Object::Integer(r)))
            }
            _ => {
                return Err(RuntimeError::UnSupportedBinOperation(
                    *op,
                    left.clone(),
                    right.clone(),
                ));
            }
        };
        self.push_stack(result);
        Ok(())
    }
    /// # 执行索引操作
    pub fn execute_index_operation(&self, obj: &Object, index: &Object) -> VmResult {
        if let Object::Array(items) = obj {
            if let Object::Integer(index) = index {
                let value = items.borrow().get(*index as usize).cloned().unwrap_or(NULL);
                return Ok(Rc::new(value));
            }
        } else if let Object::Hash(pairs) = obj {
            let key = HashKey::from_object(index)?;
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
        let right = self.pop_stack();
        let left = self.pop_stack();
        if let (Object::Integer(left), Object::Integer(right)) = (left.as_ref(), right.as_ref()) {
            let bool = match op {
                Opcode::GreaterThan => left > right,
                Opcode::GreaterEq => left >= right,
                Opcode::LessThan => left < right,
                Opcode::LessEq => left <= right,
                Opcode::Equal => left == right,
                Opcode::NotEqual => left != right,
                _ => return Err(RuntimeError::UnSupportedBinOperator(*op)),
            };
            // Ok(bool
            //     .then(|| self.bool_cache_true.clone())
            //     .unwrap_or_else(|| self.bool_cache_false.clone()))
            if bool {
                Ok(self.bool_cache_true.clone())
            } else {
                Ok(self.bool_cache_false.clone())
            }
            // Ok(self.get_bool_from_cache(bool))
        } else {
            match op {
                Opcode::Equal => Ok(self.get_bool_from_cache(left == right)),
                Opcode::NotEqual => Ok(self.get_bool_from_cache(left != right)),
                _ => Err(RuntimeError::UnSupportedBinOperation(
                    *op,
                    Object::clone(&left),
                    Object::clone(&right),
                )),
            }
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
        let callee = &self.stack[self.sp - 1]; //往回跳过参数个数位置, 当前位置是函数
        match callee.as_ref() {
            Object::Closure(Closure {
                compiled_function, ..
            }) => {
                // if arg_nums != compiled_function.num_parameters {
                //     return Err(RuntimeError::WrongArgumentCount(
                //         compiled_function.num_parameters,
                //         arg_nums,
                //     ));
                // }
                // let num_locals = closure.compiled_function.num_locals;
                let frame = Frame::new(callee.clone(), self.sp);
                // Equivalent to
                self.sp += compiled_function.num_locals;
                // self.sp = frame.base_pointer + num_locals;
                self.push_frame(frame); //进入函数内部（下一帧）
            }
            Object::Builtin(builtin_fun) => {
                //内置函数
                let mut v = vec![];
                for i in 0..arg_nums {
                    let rc = &self.stack[self.sp + i];
                    v.push(Object::clone(rc));
                }
                let r = builtin_fun(v)?;
                self.sp -= 1;
                self.push_stack(Rc::new(r));
            }
            _ => {
                return Err(RuntimeError::CustomErrMsg(
                    "calling non-function".to_string(),
                ));
            }
        }
        // self.pop_stack();
        Ok(())
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
    // #[inline]
    pub fn _read_u8(&self, insts: &[u8], start: usize) -> usize {
        insts[start] as usize
    }
    /// # 压入栈中
    pub fn push_stack(&mut self, object: Rc<Object>) {
        if self.sp == self.stack.len() {
            self.stack.push(object);
        } else {
            self.stack[self.sp] = object;
        }
        self.sp += 1;
    }
    /// # 弹出栈顶元素
    // #[inline]
    pub fn pop_stack(&mut self) -> Rc<Object> {
        if self.sp > 0 {
            self.sp -= 1;
            self.stack[self.sp].clone()
        } else {
            self.null_cache.clone()
        }
    }

    /// # 栈顶元素存入全局变量
    pub fn pop_and_set_global(&mut self, global_index: usize) {
        let global = self.pop_stack();
        if global_index >= self.globals.len() {
            self.globals.push(global);
        } else {
            self.globals[global_index] = global;
        }
    }
    /// # 弹出栈顶元素并设置到指定位置
    pub fn pop_and_set_local(&mut self, local_index: usize) {
        let popped = self.pop_stack();
        let frame = self.frames.last().unwrap();
        self.stack[frame.base_pointer + local_index] = popped;
    }
    /// # 取出全局变量
    pub fn get_global(&self, global_index: usize) -> Option<Rc<Object>> {
        self.globals.get(global_index).cloned()
    }
    pub fn get_local(&mut self, local_index: usize) -> Option<Rc<Object>> {
        let frame = self.frames.last()?;
        self.stack.get(frame.base_pointer + local_index).cloned()
    }
    /// # 取出局部变量并压入栈顶
    pub fn get_local_and_push(&mut self, local_index: usize) {
        let object = self.get_local(local_index).unwrap();
        self.push_stack(object)
    }
    /// # 取出全局变量并压入栈顶
    pub fn get_global_and_push(&mut self, global_index: usize) {
        let object = self.get_global(global_index).unwrap();
        self.push_stack(object)
    }
    pub fn get_builtin(&self, builtin_index: usize) -> VmResult {
        let builtin_fun = &BUILTINS[builtin_index];
        Ok(Rc::new(builtin_fun.builtin.clone()))
    }
    /// # 最后弹出栈顶的元素
    pub fn last_popped_stack_element(&self) -> VmResult {
        let object = &self.stack[self.sp];
        Ok(object.clone())
    }
    pub fn get_const_object(&self, index: usize) -> Rc<Object> {
        self.constants[index].clone()
    }
    pub fn current_frame_ip_inc(&mut self, n: usize) {
        self.frames.last_mut().unwrap().ip += n;
    }
    pub fn current_frame(&self) -> &Frame {
        self.frames.last().unwrap()
    }
    pub fn push_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
    }
    pub fn pop_frame(&mut self) -> Frame {
        self.frames.pop().unwrap()
    }
}
