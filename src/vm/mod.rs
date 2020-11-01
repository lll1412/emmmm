use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::compiler::{ByteCode, Constants};
use crate::compiler::code::{Opcode, read_usize};
use crate::create_rc_ref_cell;
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
    // 操作数栈
    stack: Vec<Rc<Object>>,
    // stack pointer
    sp: usize,
    // 全局变量
    globals: Globals,
    //栈帧
    frames: Vec<RCFrame>,
    call_count: usize,
    // add_times: u128,
    // call_times: u128,
    times_spend_map: HashMap<u8, u128>,
    tick_0: u128,
    tick_1: u128,
    tick_2: u128,
}

impl Vm {
    pub fn new(byte_code: ByteCode) -> Self {
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
        Self {
            constants: byte_code.constants,
            stack,
            sp: 0,
            globals,
            frames,
            // frame_index: 1,
            call_count: 0,
            // add_times: 0,
            // call_times: 0,
            times_spend_map: HashMap::new(),
            tick_0: 0,
            tick_1: 0,
            tick_2: 0,
        }
    }
    pub fn run(&mut self) -> VmResult {
        // ip means instruction_pointer
        while self.current_frame().ip < self.current_frame().instructions().len() {
            let start = Instant::now();
            let ins = self.current_frame().instructions();
            let op_code = Opcode::from_byte(ins[self.frames.last().unwrap().ip]).unwrap();
            // self.tick_0 += start.elapsed().as_nanos();
            //从操作码后面一个位置开始
            self.frames.last_mut().unwrap().ip += 1;
            let ip = self.current_frame().ip;
            match op_code {
                Opcode::Constant => {
                    let const_index = self.read_u16(&ins, ip);
                    let constant = self.constants.borrow()[const_index].clone();
                    self.push_stack(constant)?;
                    self.frames.last_mut().unwrap().ip += 2;
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
                    self.execute_not_expression(&value)?;
                }

                Opcode::JumpAlways => {
                    self.frames.last_mut().unwrap().ip = self.read_u16(&ins, ip);
                }
                Opcode::JumpIfNotTruthy => {
                    let is_truthy = self.pop_stack()?;
                    if *is_truthy == TRUE {
                        self.frames.last_mut().unwrap().ip += 2;
                    } else {
                        self.frames.last_mut().unwrap().ip = self.read_u16(&ins, ip);
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
                    let global_index = self.read_u16(&ins, ip);
                    let object = self.get_global(global_index)?;
                    self.push_stack(object)?;
                    self.current_frame_ip_inc(2);
                }

                Opcode::SetLocal => {
                    let (local_index, n) = self.read_usize(op_code, ip);
                    self.current_frame_ip_inc(n);
                    let base_pointer = self.current_frame().base_pointer;
                    let popped = self.pop_stack()?;
                    self.stack[base_pointer + local_index] = popped;
                }
                Opcode::GetLocal => {
                    // let start_0 = Instant::now();
                    let mut frame = self.frames.last_mut().unwrap();
                    // self.tick_0 += start_0.elapsed().as_nanos();

                    // let start_1 = Instant::now();
                    let object = self.stack[frame.base_pointer + ins[ip] as usize].clone();
                    frame.ip += 1;
                    // self.tick_1 += start_1.elapsed().as_nanos();

                    // let start_2 = Instant::now();
                    self.push_stack(object)?;
                    // self.tick_2 += start_2.elapsed().as_nanos();
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
                    let func_object = &self.get_const_object(function_index);
                    if let Object::CompiledFunction(compiled_function) = func_object.as_ref() {
                        let mut frees = vec![];
                        //往前free_num个都是free_variable
                        for i in 0..free_num {
                            let free_object = &self.stack[self.sp - free_num + i];
                            frees.push(free_object.clone()); //存引用
                        }
                        let closure = Object::Closure(Closure::new(compiled_function.clone(), frees));
                        self.push_stack(Rc::new(closure))?;
                    } else {
                        return Err(RuntimeError::NotFunction(Object::clone(func_object)));
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

                Opcode::Assign => {
                    let (global_index, n) = self.read_usize(op_code, ip);
                    self.execute_assign_operation(global_index)?;
                    self.current_frame_ip_inc(n);
                }

                Opcode::Call => {
                    // let start = Instant::now();
                    let arg_nums = ins[ip] as usize;
                    // let (arg_nums, n) = self.read_usize(op_code, ip);
                    self.current_frame_ip_inc(1);
                    self.call_function(arg_nums)?;
                    // self.call_times += start.elapsed().as_nanos();
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
            let k = op_code as u8;
            let mut nv = start.elapsed().as_nanos();
            if self.times_spend_map.contains_key(&k) {
                let ov = self.times_spend_map.get(&k).unwrap();
                nv += ov;
            }
            self.times_spend_map.insert(k, nv);
        }
        println!("call count: {}", self.call_count);
        // println!("frame count: {}", self.frames.len());
        // println!(
        //     "add_times: {:?}",
        //     Duration::from_nanos(self.add_times as u64)
        // );
        // println!(
        //     "call_times: {:?}",
        //     Duration::from_nanos(self.call_times as u64)
        // );
        // self.times_spend_map.iter().map(|k,v|)
        let t = 1_000_000;
        println!(
            "sum: {}, t0: {}, t1: {}, t2: {}",
            (self.tick_0 + self.tick_1 + self.tick_2) / t,
            self.tick_0 / t,
            self.tick_1 / t,
            self.tick_2 / t
        );
        let mut map = BTreeMap::new();
        for (k, v) in self.times_spend_map.iter() {
            let op = Opcode::from_byte(*k);
            map.insert(Duration::from_nanos(*v as u64), op.unwrap());
            // println!("{:?} ==> {:?}", op, Duration::from_nanos(*v as u64));
        }
        println!("{:#?}", map.iter());
        // println!("times spend map: \n {:#?}", self.times_spend_map);
        self.last_popped_stack_element()
    }
}
