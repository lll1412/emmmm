use crate::compiler::code::{read_operands, Opcode};
use crate::object::Object;
use crate::vm::{Vm, VmError, VmResult, FALSE, GLOBALS_SIZE, STACK_SIZE, TRUE};
use std::ops::Deref;
use std::rc::Rc;

impl Vm {
    /// ## 最后弹出栈顶的元素
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
    /// ## 执行二元操作
    pub fn execute_binary_operation(&mut self, op: Opcode) -> VmResult {
        let right = self.pop_stack()?;
        let left = self.pop_stack()?;
        if let Object::Integer(right_val) = *right {
            if let Object::Integer(left_val) = *left {
                let r = match op {
                    Opcode::Add => left_val + right_val,
                    Opcode::Sub => left_val - right_val,
                    Opcode::Mul => left_val * right_val,
                    Opcode::Div => {
                        if right_val == 0 {
                            return Err(VmError::ByZero(
                                Object::clone(&left),
                                right.deref().clone(),
                            ));
                        }
                        left_val / right_val
                    }
                    _ => return Err(VmError::UnSupportedBinOperator(op)),
                };
                return Ok(Rc::new(Object::Integer(r)));
            }
        }
        Err(VmError::UnSupportedBinOperation(
            op,
            left.deref().clone(),
            right.deref().clone(),
        ))
    }
    /// ## 执行比较操作
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
                    left.deref().clone(),
                    right.deref().clone(),
                ));
            }
        };
        Ok(Rc::new(r))
    }
    /// ## 计算该指令操作数的长度，方便指令指针自增
    pub fn increment_num(&self, op: Opcode) -> usize {
        op.definition().operand_width.iter().sum()
    }
    /// ## 读取一个无符号整数，并返回字节长度
    pub fn read_usize(&self, op_code: Opcode, ip: usize) -> (usize, usize) {
        let (operands, n) = read_operands(op_code.definition(), &self.instructions[ip..]);
        (operands[0], n)
    }
    /// 压入栈中
    pub fn push_stack(&mut self, object: Rc<Object>) -> VmResult<()> {
        if self.sp == STACK_SIZE {
            Err(VmError::StackOverflow)
        } else {
            if self.sp == self.stack.len() {
                self.stack.push(object);
            } else {
                self.stack[self.sp] = object;
            }
            self.sp += 1;
            Ok(())
        }
    }
    /// 弹出栈顶元素
    pub fn pop_stack(&mut self) -> VmResult {
        if self.sp == 0 {
            Err(VmError::StackNoElement)
        } else {
            let o = &self.stack[self.sp - 1];
            self.sp -= 1;
            Ok(o.clone())
        }
    }

    /// 存入全局变量
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
    /// 取出全局变量
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
}
