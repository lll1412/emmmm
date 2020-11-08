use std::cell::RefCell;
use crate::create_rc_ref_cell;
use std::rc::Rc;

use crate::{
    compiler::{ByteCode, code::Opcode, Constants},
    object::{Closure, CompiledFunction, Object, RuntimeError},
    vm::frame::Frame,
};

mod frame;
mod r#impl;
mod test;

pub type VmResult<T = Rc<Object>> = std::result::Result<T, RuntimeError>;
pub type Globals = Rc<RefCell<Vec<Rc<Object>>>>;
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
        let globals = create_rc_ref_cell(Vec::with_capacity(GLOBALS_SIZE));
        Vm::with_global_store(byte_code, globals)
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
            let op_code = Opcode::from_byte(ins[frame.ip]).unwrap();
            //从操作码后面一个位置开始
            frame.ip += 1;
            let ip = frame.ip;
            match op_code {
                Opcode::Constant => {
                    let const_index = self.read_u16(&ins, ip);
                    let constant = Rc::clone(&self.constants[const_index]);
                    self.push_stack(constant);
                    self.current_frame_ip_inc(2);
                }
                Opcode::ConstantOne => {
                    let const_index = ins[ip] as usize;
                    let constant = self.constants[const_index].clone();
                    self.push_stack(constant);
                    self.frames.last_mut().unwrap().ip += 1;
                }
                Opcode::Constant0 => {
                    let constant = self.constants[0].clone();
                    self.push_stack(constant);
                }
                Opcode::Constant1 => {
                    let constant = self.constants[1].clone();
                    self.push_stack(constant);
                }
                Opcode::Constant2 => {
                    let constant = self.constants[2].clone();
                    self.push_stack(constant);
                }
                Opcode::Constant3 => {
                    let constant = self.constants[3].clone();
                    self.push_stack(constant);
                }
                Opcode::Constant4 => {
                    let constant = self.constants[4].clone();
                    self.push_stack(constant);
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

                Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div => {
                    let result = self.execute_binary_operation(&op_code)?;
                    self.push_stack(result);
                }

                Opcode::Pop => {
                    self.sp -= 1;
                }

                Opcode::True => {
                    self.push_stack(self.bool_cache_true.clone());
                }
                Opcode::False => {
                    self.push_stack(self.bool_cache_false.clone());
                }

                Opcode::Equal | Opcode::NotEqual | Opcode::GreaterThan | Opcode::LessThan => {
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
                            ))
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
                    self.get_global_and_push(0);
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
                    self.get_local_and_push(0);
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
                        let mut frees = vec![];
                        //往前free_num个都是free_variable
                        for i in 0..free_num {
                            let free_object = &self.stack[self.sp - free_num + i];
                            frees.push(free_object.clone()); //存引用
                        }
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
