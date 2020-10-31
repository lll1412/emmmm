use std::cell::RefCell;
use std::rc::Rc;

use crate::compiler::code::{read_usize, Opcode};
use crate::compiler::{ByteCode, Constants};
use crate::create_rc_ref_cell;
use crate::object::builtins::BUILTINS;
use crate::object::{Closure, CompiledFunction, Object, RuntimeError};
use crate::vm::frame::Frame;

mod frame;
mod r#impl;
mod test;

pub type VmResult<T = Rc<Object>> = std::result::Result<T, RuntimeError>;
pub type Globals = Rc<RefCell<Vec<Rc<Object>>>>;
pub type RCFrame = Frame;

const GLOBALS_SIZE: usize = 0xFFFF;
const STACK_SIZE: usize = 2048;
const MAX_FRAMES: usize = 1024;

pub const TRUE: Object = Object::Boolean(true);
pub const FALSE: Object = Object::Boolean(false);
pub const NULL: Object = Object::Null;

#[derive(Debug)]
pub struct Vm {
    // 常量池
    constants: Constants,
    // 字节码指令
    // instructions: Instructions,
    // 操作数栈
    stack: Vec<Rc<Object>>,
    // stack pointer
    sp: usize,
    // 全局变量
    globals: Globals,
    //内置函数
    builtins: Vec<Rc<Object>>,
    //栈帧
    frames: Vec<RCFrame>,
}

impl Vm {
    pub fn _new(byte_code: ByteCode) -> Self {
        let globals = create_rc_ref_cell(Vec::with_capacity(GLOBALS_SIZE));
        Vm::with_global_store(byte_code, globals)
    }

    pub fn with_global_store(byte_code: ByteCode, globals: Globals) -> Self {
        let mut stack = Vec::with_capacity(STACK_SIZE);
        let null = Rc::new(NULL);
        for _ in 0..STACK_SIZE {
            stack.push(null.clone())
        }
        let main_fn = CompiledFunction::new(byte_code.instructions, 0, 0);
        let main_closure = Closure::new(main_fn, vec![]);
        let main_frame = Frame::new(main_closure, 0);
        let mut frames = Vec::with_capacity(MAX_FRAMES);
        frames.push(main_frame);
        let mut builtins = vec![];
        for x in BUILTINS {
            builtins.push(Rc::new((&x.builtin).clone()));
        }
        Self {
            constants: byte_code.constants,
            stack,
            sp: 0,
            globals,
            builtins,
            frames,
        }
    }
    pub fn run(&mut self) -> VmResult {
        // ip means instruction_pointer
        while self.current_frame().ip < self.current_frame_instructions_len() {
            let ins = self.current_frame().instructions();
            let opcode = Opcode::from_byte(ins[self.current_frame().ip]);
            let ip = self.current_frame_ip_inc(1); //从操作码后面一个位置开始
            if let Some(op_code) = opcode {
                match op_code {
                    Opcode::Constant => {
                        let (const_index, n) = self.read_usize(op_code, ip);
                        let constant = self.get_const_object(const_index);
                        self.push_stack(Rc::new(constant))?;
                        self.current_frame_ip_inc(n);
                    }

                    Opcode::Array => {
                        let (arr_len, n) = self.read_usize(op_code, ip);
                        self.build_array(arr_len)?;
                        self.current_frame_ip_inc(n);
                    }

                    Opcode::Hash => {
                        let (hash_len, n) = self.read_usize(op_code, ip);
                        self.build_hash(hash_len)?;
                        self.current_frame_ip_inc(n);
                    }

                    Opcode::Index => {
                        let index = self.pop_stack()?;
                        let obj = self.pop_stack()?;
                        let result = self.execute_index_operation(&obj, &index)?;
                        self.push_stack(result)?;
                    }

                    Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div => {
                        let result = self.execute_binary_operation(op_code)?;
                        self.push_stack(result)?;
                    }

                    Opcode::Pop => {
                        self.pop_stack()?;
                    }

                    Opcode::True => {
                        self.push_stack(Rc::new(TRUE))?;
                    }
                    Opcode::False => {
                        self.push_stack(Rc::new(FALSE))?;
                    }

                    Opcode::Equal | Opcode::NotEqual | Opcode::GreaterThan | Opcode::LessThan => {
                        let x = self.execute_comparison_operation(op_code)?;
                        self.push_stack(x)?;
                    }

                    Opcode::Neg => {
                        let value = self.pop_stack()?;
                        if let Object::Integer(val) = *value {
                            self.push_stack(Rc::new(Object::Integer(-val)))?;
                        } else {
                            return Err(RuntimeError::UnSupportedUnOperation(
                                op_code,
                                (*value).clone(),
                            ));
                        }
                    }
                    Opcode::Not => {
                        let value = self.pop_stack()?;
                        if *value == TRUE {
                            self.push_stack(Rc::new(FALSE))?;
                        } else if *value == FALSE {
                            self.push_stack(Rc::new(TRUE))?;
                        } else if *value == NULL {
                            self.push_stack(Rc::new(NULL))?;
                        } else {
                            return Err(RuntimeError::UnSupportedUnOperation(
                                Opcode::Not,
                                (*value).clone(),
                            ));
                        }
                    }

                    Opcode::JumpAlways => {
                        let (jump_index, _) = self.read_usize(op_code, ip);
                        self.set_current_frame_ip(jump_index);
                    }
                    Opcode::JumpIfNotTruthy => {
                        let is_truthy = self.pop_stack()?;
                        if *is_truthy == FALSE || *is_truthy == NULL {
                            let (jump_index, _) = self.read_usize(op_code, ip);
                            self.set_current_frame_ip(jump_index);
                        } else {
                            self.current_frame_ip_inc(self.increment_num(op_code));
                        }
                    }

                    Opcode::Null => {
                        self.push_stack(Rc::new(NULL))?;
                    }

                    Opcode::SetGlobal => {
                        let (global_index, n) = self.read_usize(op_code, ip);
                        let popped = self.pop_stack()?;
                        self.set_global(global_index, popped)?;
                        self.current_frame_ip_inc(n);
                    }
                    Opcode::GetGlobal => {
                        let (global_index, n) = self.read_usize(op_code, ip);
                        let object = self.get_global(global_index)?;
                        self.push_stack(object)?;
                        self.current_frame_ip_inc(n);
                    }

                    Opcode::SetLocal => {
                        let (local_index, n) = self.read_usize(op_code, ip);
                        self.current_frame_ip_inc(n);
                        let base_pointer = self.current_frame_bp();
                        let popped = self.pop_stack()?;
                        self.stack[base_pointer + local_index] = popped;
                    }
                    Opcode::GetLocal => {
                        let (local_index, n) = self.read_usize(op_code, ip);
                        self.current_frame_ip_inc(n);
                        let base_pointer = self.current_frame_bp();
                        let object = self.stack[base_pointer + local_index].clone();
                        self.push_stack(object)?;
                    }

                    Opcode::GetBuiltin => {
                        let (built_index, n) = self.read_usize(op_code, ip);
                        let builtin = self.get_builtin(built_index)?;
                        self.push_stack(builtin)?;
                        self.current_frame_ip_inc(n);
                    }

                    Opcode::Closure => {
                        //读取函数索引
                        let function_index = read_usize(&ins[ip..], 2);
                        //读取自由变量个数
                        let free_num = read_usize(&ins[ip + 2..], 1);
                        let func_object = self.get_const_object(function_index);
                        if let Object::CompiledFunction(insts, num_locals, num_parameters) =
                            func_object
                        {
                            let compiled_function =
                                CompiledFunction::new(insts, num_locals, num_parameters);
                            let mut frees = vec![];
                            //往前free_num个都是free_variable
                            for i in 0..free_num {
                                let free_object = &self.stack[self.sp - free_num + i];
                                frees.push(free_object.clone()); //存引用
                            }
                            let closure = Object::Closure(compiled_function, frees);
                            self.push_stack(Rc::new(closure))?;
                        } else {
                            return Err(RuntimeError::NotFunction(func_object));
                        };
                        self.current_frame_ip_inc(3);
                    }
                    Opcode::GetFree => {
                        let (free_index, n) = self.read_usize(op_code, ip);
                        self.push_stack(
                            self.current_frame().closure.free_variables[free_index].clone(),
                        )?;
                        self.current_frame_ip_inc(n);
                    }
                    Opcode::CurrentClosure => {
                        let Closure {
                            compiled_function,
                            free_variables,
                        } = &self.current_frame().closure;
                        let current_closure = Object::Closure(compiled_function.clone(), free_variables.clone());
                        self.push_stack(Rc::new(current_closure))?;
                    }

                    Opcode::Assign => {
                        let (global_index, n) = self.read_usize(op_code, ip);
                        self.execute_assign_operation(global_index)?;
                        self.current_frame_ip_inc(n);
                    }

                    Opcode::Call => {
                        let (arg_nums, n) = self.read_usize(op_code, ip);
                        self.current_frame_ip_inc(n); //当前帧指令+1
                        self.call_function(arg_nums)?;
                    }
                    Opcode::ReturnValue => {
                        let return_value = self.pop_stack()?; //pop ret_val
                        let base_pointer = self.pop_frame().base_pointer; // quit cur env
                        self.sp = base_pointer - 1;
                        self.push_stack(return_value.clone())?; //push ret_val
                    }
                    Opcode::Return => {
                        let base_pointer = self.pop_frame().base_pointer; // quit cur env
                        self.sp = base_pointer - 1;
                        self.push_stack(Rc::new(NULL))?;
                    }
                    _ => return Err(RuntimeError::UnKnownOpCode(op_code)),
                }
            }
        }
        self.last_popped_stack_element()
    }
}
