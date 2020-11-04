use std::fmt::{Display, Formatter};

pub type Instructions = Vec<u8>;

macro_rules! op_build {
    ($name:ident, [$($var: ident($($v: expr),*)),+,]) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        #[repr(u8)]
        pub enum $name {
            $($var,)+
        }

        impl $name {
            pub fn from_byte(byte: u8) -> Option<Self> {
                match byte {
                    $(
                        op if Self::$var as u8 == op => Some(Self::$var),
                    )+
                    _ => None
                }
            }

            pub fn definition(&self) -> Definition {
                match self {
                    $(
                        Self::$var => Definition {
                                    name: Self::$var.to_string(),
                                    operand_width: vec![$(
                                        $v,
                                    )*],
                                },
                    )+
                }
            }
        }

        impl Display for $name{
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$var => write!(f, "Op{}", stringify!($var)),
                    )+
                }
            }
        }
    };
}

// (类名, [枚举(操作数位数), ...])
op_build!(
    Opcode,
    [
        // 常量
        Constant(2),
        Constant0(),// const_index = 0..4
        Constant1(),
        Constant2(),
        Constant3(),
        Constant4(),
        ConstantOne(1),//一字节
        // 数组
        Array(2),
        // Hash
        Hash(2),
        // 索引操作
        Index(),
        Pop(),
        //四则运算符
        Add(),
        Sub(),
        Mul(),
        Div(),
        //布尔字面常量
        True(),
        False(),
        //比较运算符
        Equal(),
        NotEqual(),
        GreaterThan(),
        LessThan(),
        //一元运算符
        Neg(),
        Not(),
        //跳转指令
        JumpIfNotTruthy(2),
        JumpIfLess(2),
        // JumpIfNotEq(2),
        JumpAlways(2),
        //全局变量绑定
        SetGlobal(2),
        GetGlobal(2),
        GetGlobal0(),
        GetGlobal1(),
        GetGlobal2(),
        GetGlobal3(),
        GetGlobal4(),
        //局部变量
        SetLocal(1),
        SetLocal0(),// set_local = 0..4
        SetLocal1(),
        SetLocal2(),
        SetLocal3(),
        SetLocal4(),

        GetLocal(1),
        GetLocal0(),// get_local = 0..4
        GetLocal1(),
        GetLocal2(),
        GetLocal3(),
        GetLocal4(),
        //内置函数
        GetBuiltin(1),
        Closure(2, 1),
        // GetThis(),
        // CurrentClosure(),
        GetFree(1),
        // 赋值操作
        Assign(2),
        // 函数调用(arg_len)
        Call(1),
        // 函数返回值
        ReturnValue(),
        Return(),
        //
        Null(),
        Uninitialize(),
    ]
);

#[derive(Debug)]
pub struct Definition {
    pub name: String,
    pub operand_width: Vec<usize>,
}
//
// #[derive(Debug, Clone, PartialEq)]
// pub enum Constant {
//     Integer(i64),
//     String(String),
//     /// insts, num_locals, num_parameters
//     CompiledFunction(Instructions, usize, usize),
// }
//
// impl Constant {
//     pub fn _to_object(&self) -> Object {
//         match self {
//             Constant::Integer(val) => Object::Integer(*val),
//             Constant::String(val) => Object::String(val.clone()),
//             Constant::CompiledFunction(insts, num_locals, num_parameters) => {
//                 Object::CompiledFunction(CompiledFunction {
//                     insts: Rc::new(insts.clone()),
//                     num_locals: *num_locals,
//                     num_parameters: *num_parameters,
//                 })
//             }
//         }
//     }
// }
// impl Display for Constant {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Constant::Integer(val) => write!(f, "{}", val),
//             Constant::String(val) => write!(f, "{}", val),
//             Constant::CompiledFunction(insts, _num_locals, _num_params) => {
//                 write!(f, "{}", _print_instructions(insts))
//             }
//         }
//     }
// }

/// # 生成 指令
/// ## 操作码, 操作数 => 二进制指令
/// ## (OpConstant, 0xABCD) => [0, 0xAB, 0xCD]
pub fn make(op_code: Opcode, operands: Vec<usize>) -> Instructions {
    let definition = op_code.definition();

    let instruction_len = definition.operand_width.iter().sum::<usize>();

    let mut instructions = Vec::with_capacity(instruction_len + 1);

    instructions.push(op_code as u8);

    for (i, operand) in operands.into_iter().enumerate() {
        let width = definition.operand_width[i];
        match width {
            2 => {
                //[0, 0, ..., xx, xx]
                let x = &operand.to_be_bytes();
                let len = x.len();
                instructions.push(x[len - 2]); //高位
                instructions.push(x[len - 1]); //低位
            }
            1 => {
                let x = *operand.to_be_bytes().last().unwrap();
                instructions.push(x);
            }
            _ => unimplemented!(),
        }
    }
    instructions
}
pub fn _make(op_code: Opcode, index: usize) -> Instructions {
    make(op_code, vec![index])
}
pub fn _make_closure(fun_index: usize, free_num: usize) -> Instructions {
    make(Opcode::Closure, vec![fun_index, free_num])
}
pub fn _make_const(index: usize) -> Instructions {
    make(Opcode::Constant, vec![index])
}

pub fn _make_noop(op_code: Opcode) -> Instructions {
    make(op_code, vec![])
}

/// # 打印指令
pub fn print_instructions(instructions: &Instructions) -> String {
    let mut pc = 0;
    let mut string = String::new();
    while pc < instructions.len() {
        let op = instructions[pc];
        let option = Opcode::from_byte(op);
        match option {
            None => return string,
            Some(op_code) => {
                let definition = op_code.definition();
                let operand_count = definition.operand_width.len(); //操作数 数量
                string.push_str(&format!(
                    "{pc:4} {operator}",
                    pc = pc,
                    operator = definition.name,
                    // op = op_code as u8
                ));
                for k in 0..operand_count {
                    let instruction_len = definition.operand_width[k]; //指令长度
                    let operand = read_usize(&instructions[pc + 1..], instruction_len);
                    string.push_str(&format!(" {operand:02}", operand = operand));
                    pc += instruction_len;
                }
                string.push('\n');
                pc += 1;
            }
        }
    }

    string
}

/// # 读取操作数
pub fn read_operands(def: &Definition, instructions: &[u8]) -> (Vec<usize>, usize) {
    let mut operands = vec![];
    let mut bytes_read = 0;
    for width in &def.operand_width {
        match width {
            2 => operands.push(read_usize(&instructions[bytes_read..], 2)),
            1 => {
                operands.push(read_usize(&instructions[bytes_read..], 1));
            }
            _ => unimplemented!(),
        }
        bytes_read += width;
    }
    (operands, bytes_read)
}
pub fn _read_operand(width: usize, insts: &[u8]) -> usize {
    match width {
        2 => read_usize(&insts, 2),
        1 => read_usize(&insts, 1),
        _ => unimplemented!(),
    }
}
/// # 从指令中读取数据并转换为usize
pub fn read_usize(instructions: &[u8], n: usize) -> usize {
    let mut bytes = [0; 8];
    for i in 0..n {
        bytes[8 - n + i] = instructions[i]
    }
    usize::from_be_bytes(bytes)
}
