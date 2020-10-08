use std::cell::RefCell;
use std::rc::Rc;

use crate::compiler::code::{Instructions, Opcode};
use crate::compiler::{ByteCode, Constants};
use crate::object::Object;

mod r#impl;
mod test;

pub type VmResult<T = Rc<Object>> = std::result::Result<T, VmError>;
pub type Globals = Rc<RefCell<Vec<Rc<Object>>>>;

const GLOBALS_SIZE: usize = 0xFFFF;
const STACK_SIZE: usize = 2048;

pub const TRUE: Object = Object::Boolean(true);
pub const FALSE: Object = Object::Boolean(false);
pub const NULL: Object = Object::Null;

#[derive(Debug)]
pub struct Vm {
    // 常量池
    constants: Constants,
    // 字节码指令
    instructions: Instructions,
    // 操作数栈
    stack: Vec<Rc<Object>>,
    // stack pointer
    sp: usize,
    // 全局变量
    globals: Globals,
}

impl Vm {
    pub fn _new(byte_code: ByteCode) -> Self {
        Self {
            constants: byte_code.constants,
            instructions: byte_code.instructions,
            stack: Vec::with_capacity(STACK_SIZE),
            sp: 0,
            globals: Rc::new(RefCell::new(Vec::with_capacity(GLOBALS_SIZE))),
        }
    }

    pub fn with_global_store(byte_code: ByteCode, globals: Globals) -> Self {
        Self {
            constants: byte_code.constants.clone(),
            instructions: byte_code.instructions,
            stack: Vec::with_capacity(STACK_SIZE),
            sp: 0,
            globals,
        }
    }
    pub fn run(&mut self) -> VmResult {
        // ip means instruction_pointer
        let mut ip = 0;
        while ip < self.instructions.len() {
            let opcode = Opcode::from_byte(self.instructions[ip]);
            ip += 1; //从操作码后面一个位置开始
            if let Some(op_code) = opcode {
                match op_code {
                    Opcode::Constant => {
                        let (const_index, n) = self.read_usize(op_code, ip);
                        let constant = self.constants.borrow()[const_index].to_object();
                        self.push_stack(Rc::new(constant))?;
                        ip += n;
                    }

                    Opcode::Array => {
                        let (arr_len, n) = self.read_usize(op_code, ip);
                        self.build_array(arr_len)?;
                        ip += n;
                    }

                    Opcode::Hash => {
                        let (hash_len, n) = self.read_usize(op_code, ip);
                        self.build_hash(hash_len)?;
                        ip += n;
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
                            return Err(VmError::UnSupportedUnOperation(op_code, (*value).clone()));
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
                            return Err(VmError::UnSupportedUnOperation(
                                Opcode::Not,
                                (*value).clone(),
                            ));
                        }
                    }

                    Opcode::JumpAlways => {
                        let (jump_index, _) = self.read_usize(op_code, ip);
                        ip = jump_index;
                    }
                    Opcode::JumpIfNotTruthy => {
                        let is_truthy = self.pop_stack()?;
                        if *is_truthy == FALSE || *is_truthy == NULL {
                            let (jump_index, _) = self.read_usize(op_code, ip);
                            ip = jump_index;
                        } else {
                            ip += self.increment_num(op_code);
                        }
                    }

                    Opcode::Null => {
                        self.push_stack(Rc::new(NULL))?;
                    }

                    Opcode::SetGlobal => {
                        let (global_index, n) = self.read_usize(op_code, ip);
                        let popped = self.pop_stack()?;
                        self.set_global(global_index, popped)?;
                        ip += n;
                    }
                    Opcode::GetGlobal => {
                        let (global_index, n) = self.read_usize(op_code, ip);
                        let object = self.get_global(global_index)?;
                        self.push_stack(object)?;
                        ip += n;
                    }
                    Opcode::Assign => {
                        let (global_index, n) = self.read_usize(op_code, ip);
                        self.execute_assign_operation(global_index)?;
                        ip += n;
                    }
                    _ => return Err(VmError::UnKnownOpCode(op_code)),
                }
            }
        }
        self.last_popped_stack_element()
    }
}

#[derive(Debug)]
pub enum VmError {
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
}
