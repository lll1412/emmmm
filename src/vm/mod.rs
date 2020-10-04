use crate::compiler::code::{self, read_operands, Instructions, Opcode};
use crate::compiler::ByteCode;
use crate::object::Object;

mod test;

pub type VmResult<T = Object> = std::result::Result<T, VmError>;

const STACK_SIZE: usize = 2048;
pub const TRUE: Object = Object::Boolean(true);
pub const FALSE: Object = Object::Boolean(false);
pub const NULL: Object = Object::Null;

#[derive(Debug)]
pub struct Vm {
    constants: Vec<Object>,
    instructions: Instructions,
    stack: Vec<Object>,
    sp: usize, // stack pointer
}

impl Vm {
    pub fn new(byte_code: ByteCode) -> Self {
        Self {
            constants: byte_code.constants,
            instructions: byte_code.instructions,
            stack: Vec::with_capacity(STACK_SIZE),
            sp: 0,
        }
    }

    pub fn run(&mut self) -> VmResult {
        let mut instruction_pointer = 0;
        while instruction_pointer < self.instructions.len() {
            let opcode = Opcode::from_byte(self.instructions[instruction_pointer]);
            instruction_pointer += 1; //从操作码后面一个位置开始
            if let Some(op_code) = opcode {
                match op_code {
                    Opcode::Constant => {
                        let (constants, n) = code::read_operands(
                            op_code.definition(),
                            &self.instructions,
                            instruction_pointer,
                        );
                        for const_index in constants {
                            let object = self.constants[const_index].clone();
                            self.push(object)?;
                        }
                        instruction_pointer += n;
                    }

                    Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div => {
                        let result = self.execute_binary_operation(op_code)?;
                        self.push(result)?;
                    }

                    Opcode::Pop => {
                        self.pop()?;
                    }

                    Opcode::True => {
                        self.push(TRUE)?;
                    }
                    Opcode::False => {
                        self.push(FALSE)?;
                    }

                    Opcode::Equal | Opcode::NotEqual | Opcode::GreaterThan | Opcode::LessThan => {
                        let x = self.execute_comparison_operation(op_code)?;
                        self.push(x)?;
                    }
                    Opcode::Neg => {
                        let value = self.pop()?;
                        let value = (-value)?;
                        self.push(value)?;
                    }
                    Opcode::Not => {
                        let value = self.pop()?;
                        if value == TRUE {
                            self.push(FALSE)?;
                        } else if value == FALSE {
                            self.push(TRUE)?;
                        } else if value == NULL {
                            self.push(NULL)?;
                        } else {
                            return Err(VmError::UnSupportedUnOperation(Opcode::Not, value));
                        }
                    }
                    Opcode::JumpAlways => {
                        let (constants, _n) = read_operands(
                            op_code.definition(),
                            &self.instructions,
                            instruction_pointer,
                        );
                        instruction_pointer = constants[0];
                    }
                    Opcode::JumpIfNotTruthy => {
                        let (constants, n) = read_operands(
                            op_code.definition(),
                            &self.instructions,
                            instruction_pointer,
                        );
                        let is_truthy = self.pop()?;
                        if is_truthy == FALSE || is_truthy == NULL {
                            instruction_pointer = constants[0];
                        } else {
                            instruction_pointer += n;
                        }
                    }
                    Opcode::Null => {
                        self.push(NULL)?;
                    }
                    _ => panic!(),
                }
            }
        }
        self.last_popped_stack_element()
    }

    pub fn push(&mut self, constant: Object) -> VmResult<()> {
        if self.sp == STACK_SIZE {
            Err(VmError::StackOverflow)
        } else {
            if self.sp == self.stack.len() {
                self.stack.push(constant);
            } else {
                self.stack[self.sp] = constant;
            }
            self.sp += 1;
            Ok(())
        }
    }

    pub fn pop(&mut self) -> VmResult {
        if self.sp == 0 {
            Err(VmError::StackNoElement)
        } else {
            let o = &self.stack[self.sp - 1];
            self.sp -= 1;
            Ok(o.clone())
        }
    }

    fn last_popped_stack_element(&self) -> VmResult {
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

    fn execute_binary_operation(&mut self, op: Opcode) -> VmResult {
        let right = self.pop()?;
        let left = self.pop()?;
        match op {
            Opcode::Add => left + right,
            Opcode::Sub => left - right,
            Opcode::Mul => left * right,
            Opcode::Div => left / right,
            _ => Err(VmError::UnSupportedBinOperation(op, left, right)),
        }
    }
    fn execute_comparison_operation(&mut self, op: Opcode) -> VmResult {
        let right = self.pop()?;
        let left = self.pop()?;
        if let Object::Integer(left) = left {
            if let Object::Integer(right) = right {
                let bool = match op {
                    Opcode::GreaterThan => left > right,
                    Opcode::LessThan => left < right,
                    Opcode::Equal => left == right,
                    Opcode::NotEqual => left != right,
                    _ => return Err(VmError::UnSupportedBinOperator(op)),
                };
                return Ok(if bool { TRUE } else { FALSE });
            }
        }
        match op {
            Opcode::Equal => Ok(if left == right { TRUE } else { FALSE }),
            Opcode::NotEqual => Ok(if left != right { TRUE } else { FALSE }),
            _ => Err(VmError::UnSupportedBinOperation(op, left, right)),
        }
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
}
