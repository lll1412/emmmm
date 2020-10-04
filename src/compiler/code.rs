use std::fmt::{Display, Formatter};

pub type Instructions = Vec<u8>;

// (类名, [枚举(操作数位数), ...])
macro_rules! op_build {
    ($name:ident, [$($var: ident($v: expr)),+,]) => {
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
                                    operand_width: $v,
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

op_build!(
    Opcode,
    [
        // 常量
        Constant(vec![2]),
        Pop(vec![]),
        //四则运算符
        Add(vec![]),
        Sub(vec![]),
        Mul(vec![]),
        Div(vec![]),
        //布尔字面常量
        True(vec![]),
        False(vec![]),
        //比较运算符
        Equal(vec![]),
        NotEqual(vec![]),
        GreaterThan(vec![]),
        LessThan(vec![]),
        //一元运算符
        Neg(vec![]),
        Not(vec![]),
        //跳转指令
        JumpIfNotTruthy(vec![2]),
        JumpAlways(vec![2]),
        //
        Null(vec![]),
        Uninitialize(vec![]),
    ]
);

#[derive(Debug)]
pub struct Definition {
    pub name: String,
    pub operand_width: Vec<usize>,
}

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
            _ => {}
        }
    }
    instructions
}

/// # 打印指令
pub fn print_instructions(instructions: Instructions) -> String {
    let mut i = 0;
    let mut string = String::new();
    while i < instructions.len() {
        let op = instructions[i];
        let option = Opcode::from_byte(op);
        match option {
            None => return string,
            Some(op_code) => {
                let definition = op_code.definition();
                let operand_count = definition.operand_width.len(); //操作数 数量
                string.push_str(&format!(
                    "{pc:04} {operator}",
                    pc = i,
                    operator = definition.name,
                ));
                for k in 0..operand_count {
                    let instruction_len = definition.operand_width[k]; //指令长度
                    let operand = read_usize(&instructions, i + 1, instruction_len);
                    string.push_str(&format!(" {operand}", operand = operand));
                    i += instruction_len;
                }
                string.push('\n');
                i += 1;
            }
        }
    }

    string
}

/// # 读取操作数
pub fn read_operands(
    def: Definition,
    instructions: &Instructions,
    offset: usize,
) -> (Vec<usize>, usize) {
    let mut operands = Vec::with_capacity(def.operand_width.len());
    let mut bytes_read = 0;
    for width in def.operand_width {
        match width {
            2 => operands.push(read_usize(instructions, bytes_read + offset, 2)),
            1 => {
                operands.push(read_usize(instructions, bytes_read + offset, 1));
            }
            _ => {}
        }
        bytes_read += width;
    }
    (operands, bytes_read)
}

/// # 从指令中读取数据并转换为usize
pub fn read_usize(instructions: &Instructions, start: usize, n: usize) -> usize {
    let mut bytes = [0; 8];
    for i in 0..n {
        bytes[8 - n + i] = instructions[start + i]
    }
    usize::from_be_bytes(bytes)
}
