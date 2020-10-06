use std::cell::RefCell;
use std::rc::Rc;

use crate::compiler::code::{Constant, Instructions, Opcode};
use crate::compiler::symbol_table::SymbolTable;
use crate::core::base::ast::{
    BinaryOperator, BlockStatement, Expression, Program, Statement, UnaryOperator,
};

pub mod code;
pub mod symbol_table;
mod test;

type CompileResult<T = ()> = std::result::Result<T, CompileError>;
pub type Constants = Rc<RefCell<Vec<Constant>>>;
pub type RcSymbolTable = Rc<RefCell<SymbolTable>>;

#[derive(Debug)]
pub struct Compiler {
    instructions: Instructions,
    constants: Constants,
    symbol_table: RcSymbolTable,
    last_instruction: EmittedInstruction,
    previous_instruction: EmittedInstruction,
}

#[derive(Debug, Clone)]
pub struct EmittedInstruction {
    pub op_code: Opcode,
    pub position: usize,
}

impl EmittedInstruction {
    fn default() -> Self {
        Self {
            op_code: Opcode::Uninitialize,
            position: 0,
        }
    }
}

impl Compiler {
    pub fn _new() -> Self {
        Self::with_state(
            Rc::new(RefCell::new(SymbolTable::default())),
            Rc::new(RefCell::new(vec![])),
        )
    }
    pub fn with_state(symbol_table: RcSymbolTable, constants: Constants) -> Self {
        Self {
            instructions: vec![],
            constants,
            symbol_table,
            last_instruction: EmittedInstruction::default(),
            previous_instruction: EmittedInstruction::default(),
        }
    }
    /// 编译为字节码
    pub fn compile(&mut self, program: Program) -> CompileResult<ByteCode> {
        for statement in &program.statements {
            self.compile_statement(statement)?;
        }
        Ok(ByteCode::new(
            self.instructions.clone(),
            self.constants.clone(),
        ))
    }

    /// 编译语句
    fn compile_statement(&mut self, statement: &Statement) -> CompileResult {
        match statement {
            Statement::Let(name, expr) => {
                self.compile_expression(expr)?;
                self.store_symbol(name);
            }
            Statement::Return(_) => {}
            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
                self.emit(Opcode::Pop, vec![]);
            }
        }
        Ok(())
    }
    fn compile_block_statement(&mut self, block_statement: &BlockStatement) -> CompileResult {
        for statement in &block_statement.statements {
            self.compile_statement(statement)?;
        }
        Ok(())
    }
    /// 编译表达式
    fn compile_expression(&mut self, expression: &Expression) -> CompileResult {
        match expression {
            Expression::IntLiteral(value) => {
                let i = self.add_constant(Constant::Integer(*value));
                self.emit(Opcode::Constant, vec![i]);
            }
            Expression::BoolLiteral(bool) => {
                if *bool {
                    self.emit(Opcode::True, vec![]);
                } else {
                    self.emit(Opcode::False, vec![]);
                }
            }
            Expression::StringLiteral(string) => {
                let i = self.add_constant(Constant::String(string.to_string()));
                self.emit(Opcode::Constant, vec![i]);
            }
            Expression::ArrayLiteral(items) => {
                for item in items {
                    self.compile_expression(item)?;
                }
                self.emit(Opcode::Array, vec![items.len()]);
            }
            Expression::HashLiteral(pairs) => {
                for (k, v) in pairs {
                    self.compile_expression(k)?;
                    self.compile_expression(v)?;
                }
                self.emit(Opcode::Hash, vec![pairs.len()]);
            }
            Expression::Index(left_expr, index_expr) => {
                self.compile_expression(left_expr)?;
                self.compile_expression(index_expr)?;
                self.emit(Opcode::Index, vec![]);
            }
            Expression::Identifier(name) => {
                self.load_symbol(name)?;
            }
            Expression::Unary(op, expr) => {
                self.compile_expression(expr)?;
                self.compile_unary_expression(op)?;
            }
            Expression::Binary(op, left, right) => {
                self.compile_expression(left)?;
                self.compile_expression(right)?;
                self.compile_binary_expression(op)?;
            }
            Expression::If(cond, cons, alt) => {
                self.compile_expression(cond)?;
                //条件不成立跳转的位置
                let jump_if_not_truthy_pos = self.emit(Opcode::JumpIfNotTruthy, vec![9999]);
                self.compile_block_statement(cons)?;
                // if语句是一个表达式有pop,
                // 编译语句块之后会多一个pop,
                // 语句块最后的值需要保存到栈上,作为if的返回值,不能在内部pop掉
                if self.last_instruction_pop() {
                    // 移除pop指令
                    self.remove_last_instruction();
                }
                //如果if语句块正常执行到这，就不能继续后面的else块，应该跳转到整个语句末尾
                let jump_always_pos = self.emit(Opcode::JumpAlways, vec![9999]);
                //将条件不成立跳转位置设定为当前位置
                let after_cons_pos = self.instructions.len();
                self.change_operand(jump_if_not_truthy_pos, after_cons_pos);
                // 如果有else语句块
                if let Some(alt) = alt {
                    self.compile_block_statement(alt)?;
                    if self.last_instruction_pop() {
                        self.remove_last_instruction();
                    }
                } else {
                    self.emit(Opcode::Null, vec![]);
                }
                //条件语句末尾
                let final_pos = self.instructions.len();
                self.change_operand(jump_always_pos, final_pos);
            }
            _ => return Err(CompileError::UnknownExpression(expression.clone())),
        }
        Ok(())
    }
    /// 编译一元表达式
    fn compile_unary_expression(&mut self, op: &UnaryOperator) -> CompileResult {
        match op {
            UnaryOperator::Not => {
                self.emit(Opcode::Not, vec![]);
            }
            UnaryOperator::Neg => {
                self.emit(Opcode::Neg, vec![]);
            } // _ => return Err(CompileError::UnknownUnOperator(op.clone())),
        }
        Ok(())
    }
    /// 编译二元表达式
    fn compile_binary_expression(&mut self, op: &BinaryOperator) -> CompileResult {
        match op {
            BinaryOperator::Plus => {
                self.emit(Opcode::Add, vec![]);
            }
            BinaryOperator::Minus => {
                self.emit(Opcode::Sub, vec![]);
            }
            BinaryOperator::Mul => {
                self.emit(Opcode::Mul, vec![]);
            }
            BinaryOperator::Div => {
                self.emit(Opcode::Div, vec![]);
            }
            BinaryOperator::Gt => {
                self.emit(Opcode::GreaterThan, vec![]);
            }
            BinaryOperator::Lt => {
                self.emit(Opcode::LessThan, vec![]);
            }
            BinaryOperator::Eq => {
                self.emit(Opcode::Equal, vec![]);
            }
            BinaryOperator::NotEq => {
                self.emit(Opcode::NotEqual, vec![]);
            }

            _ => return Err(CompileError::UnknownBinOperator(op.clone())),
        }
        Ok(())
    }
    /// 常量池添加常量，返回常量索引
    fn add_constant(&mut self, constant: Constant) -> usize {
        self.constants.borrow_mut().push(constant);
        self.constants.borrow().len() - 1
    }
    /// 指令表添加指令，返回指令开始位置
    fn add_instruction(&mut self, instruction: &mut Instructions) -> usize {
        let pos_new_ins = self.instructions.len();
        self.instructions.append(instruction);
        pos_new_ins
    }
    /// 生成指令
    fn emit(&mut self, op: Opcode, operands: Vec<usize>) -> usize {
        let mut ins = code::make(op, operands);
        let pos = self.add_instruction(&mut ins);
        self.set_last_instruction(op, pos);
        pos
    }
    /// 保存上一条指令
    fn set_last_instruction(&mut self, op: Opcode, pos: usize) {
        let previous = self.last_instruction.clone();
        let last = EmittedInstruction {
            op_code: op,
            position: pos,
        };
        self.last_instruction = last;
        self.previous_instruction = previous;
    }
    /// 上一条是否为pop指令
    fn last_instruction_pop(&self) -> bool {
        self.last_instruction.op_code == Opcode::Pop
    }
    /// 移除上一条指令
    fn remove_last_instruction(&mut self) {
        self.instructions = self.instructions[..self.last_instruction.position].to_vec();
        self.last_instruction = self.previous_instruction.clone();
    }
    fn _last_instruction_jump(&self) -> bool {
        self.last_instruction.op_code == Opcode::JumpAlways
    }
    /// 指令替换
    fn replace_instruction(&mut self, pos: usize, new_instruction: Instructions) {
        for (i, inst) in new_instruction.into_iter().enumerate() {
            self.instructions[pos + i] = inst;
        }
    }
    /// 改变操作数
    fn change_operand(&mut self, op_pos: usize, operand: usize) {
        let op = Opcode::from_byte(self.instructions[op_pos]).unwrap();
        let new_instruction = code::make(op, vec![operand]);
        self.replace_instruction(op_pos, new_instruction);
    }
    /// 读取符号表元素（生成一条获取该数据的指令）
    fn load_symbol(&mut self, name: &str) -> CompileResult {
        let option = self.symbol_table.borrow().resolve(name);
        match option {
            None => return Err(CompileError::UndefinedVariable(name.to_string())),
            Some(symbol) => {
                self.emit(Opcode::GetGlobal, vec![symbol.index]);
            }
        }
        Ok(())
    }
    ///
    fn store_symbol(&mut self, name: &str) {
        let symbol = self.symbol_table.borrow_mut().define(name);
        self.emit(Opcode::SetGlobal, vec![symbol.index]);
    }
}

#[derive(Debug)]
pub struct ByteCode {
    pub instructions: Instructions,
    pub constants: Constants,
}

impl ByteCode {
    pub fn new(instructions: Instructions, constants: Constants) -> Self {
        Self {
            instructions,
            constants,
        }
    }
}

#[derive(Debug)]
pub enum CompileError {
    UnknownBinOperator(BinaryOperator),
    _UnknownUnOperator(UnaryOperator),
    UnknownExpression(Expression),

    UndefinedVariable(String),
}
