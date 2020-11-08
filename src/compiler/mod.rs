use std::cell::RefCell;
use std::rc::Rc;

use crate::compiler::code::{Instructions, Opcode};
use crate::compiler::symbol_table::{Symbol, SymbolScope, SymbolTable};
use crate::create_rc_ref_cell;
use crate::object::builtins::BUILTINS;
use crate::object::{CompiledFunction, Object};
use crate::parser::base::ast::{
    BinaryOperator, BlockStatement, Expression, Program, Statement, UnaryOperator,
};
use std::prelude::v1::Option::Some;

pub mod code;
pub mod symbol_table;
mod test;

type CompileResult<T = ()> = std::result::Result<T, CompileError>;
pub type Constants = Vec<Rc<Object>>;
pub type RcSymbolTable = Rc<RefCell<SymbolTable>>;
const CONSTANT_CAPACITY: usize = 0xFFFF;

#[derive(Debug, Clone)]
pub struct Compiler {
    constants: Constants,
    symbol_table: RcSymbolTable,
    scopes: Vec<CompilationScope>,
    scope_index: usize,
}
#[derive(Debug, Clone)]
pub struct ByteCode {
    pub instructions: Instructions,
    pub constants: Constants,
}
#[derive(Debug, Clone)]
pub struct EmittedInstruction {
    pub op_code: Opcode,
    pub position: usize,
}

#[derive(Debug, Clone, Default)]
pub struct CompilationScope {
    instructions: Instructions,
    last_instruction: Option<EmittedInstruction>,
    previous_instruction: Option<EmittedInstruction>,
}

#[derive(Debug)]
pub enum CompileError {
    UnknownBinOperator(BinaryOperator),
    UnsupportedBinOperation(BinaryOperator, Expression, Expression),
    _UnsupportedIndexOperation(Expression, Expression),

    _UnknownUnOperator(UnaryOperator),
    UnknownExpression(Expression),

    UndefinedIdentifier(String),
    CustomErrMsg(String),
}

impl CompilationScope {
    fn new() -> Self {
        Default::default()
    }
    // 判断上条指令是否为
    fn last_instruction_is(&self, op: Opcode) -> bool {
        match &self.last_instruction {
            None => false,
            Some(last) => last.op_code == op,
        }
    }
    // 记录为为上条指令
    fn set_last_instruction(&mut self, inst: EmittedInstruction) {
        self.previous_instruction = self.last_instruction.clone();
        self.last_instruction = Some(inst)
    }
    // 移除上一条指令
    fn remove_last_instruction(&mut self) -> CompileResult {
        let last_instruction = &self.last_instruction;
        if let Some(last) = last_instruction {
            self.instructions = self.instructions[..last.position].to_vec();
            self.last_instruction = self.previous_instruction.clone();
            Ok(())
        } else {
            Err(CompileError::CustomErrMsg(
                "have no last instruction".to_string(),
            ))
        }
    }
    // 指令替换
    fn replace_instruction(&mut self, pos: usize, new_instruction: Instructions) {
        for (i, inst) in new_instruction.into_iter().enumerate() {
            self.instructions[pos + i] = inst;
        }
    }
    // 添加指令
    fn add_instruction(&mut self, instruction: &mut Instructions) -> usize {
        let pos_new_ins = self.instructions.len();
        self.instructions.append(instruction);
        pos_new_ins
    }
}

impl EmittedInstruction {
    fn _default() -> Self {
        Self {
            op_code: Opcode::Uninitialize,
            position: 0,
        }
    }
}

impl Compiler {
    pub fn new() -> Self {
        let symbol_table = create_rc_ref_cell(SymbolTable::new());
        // let constants = create_rc_ref_cell(vec![]);
        let constants = Vec::with_capacity(CONSTANT_CAPACITY);
        Compiler::with_state(symbol_table, constants)
    }
    pub fn with_state(symbol_table: RcSymbolTable, constants: Constants) -> Self {
        for (i, builtin) in BUILTINS.iter().enumerate() {
            //预编译内置函数
            symbol_table.borrow_mut().define_builtin(i, builtin);
        }
        let main_scope = CompilationScope::new();
        Compiler {
            constants,
            symbol_table,
            scopes: vec![main_scope],
            scope_index: 1,
        }
    }
    /// 编译为字节码
    pub fn compile(&mut self, program: &Program) -> CompileResult<ByteCode> {
        for statement in &program.statements {
            self.compile_statement(statement)?;
        }
        Ok(self.bytecode())
    }
    pub fn bytecode(&mut self) -> ByteCode {
        ByteCode::new(
            self.cur_instruction().clone(),
            Constants::clone(&self.constants),
        )
    }
    fn enter_scope(&mut self) {
        //当前作用域
        let scope = CompilationScope {
            instructions: vec![],
            last_instruction: None,
            previous_instruction: None,
        };
        //入栈
        if self.scope_index >= self.scopes.len() {
            self.scopes.push(scope);
        } else {
            self.scopes[self.scope_index] = scope;
        }
        self.scope_index += 1;
        //进入下级符号表
        self.symbol_table = SymbolTable::new_enclosed(self.symbol_table.clone())
    }
    fn leave_scope(&mut self) -> Instructions {
        //退回上级符号表
        let st = SymbolTable::clone(&self.symbol_table.borrow());
        self.symbol_table = st.outer.unwrap();
        // let x = (&*self.symbol_table).borrow_mut().outer.clone().unwrap();
        // self.symbol_table = x;
        //出栈
        self.scope_index -= 1;
        self.scopes[self.scope_index].instructions.clone()
    }
    /// 编译语句
    fn compile_statement(&mut self, statement: &Statement) -> CompileResult {
        match statement {
            Statement::Let(name, expr) => {
                //先定义函数名，不然递归会找不着当前函数
                let symbol = self.symbol_table.borrow_mut().define(name);
                self.compile_expression(expr)?;
                self.store_symbol(symbol);
            }
            Statement::Return(ret) => match ret {
                None => {
                    self.emit(Opcode::Return, vec![]);
                }
                Some(expr) => {
                    self.compile_expression(expr)?;
                    self.emit(Opcode::ReturnValue, vec![]);
                }
            },
            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
                if let Expression::Binary(bin_op, _, _) = expr {
                    if bin_op == &BinaryOperator::Assign {
                        //赋值表达式末尾不添加Pop指令
                        return Ok(());
                    }
                }
                self.emit(Opcode::Pop, vec![]);
            }
            Statement::Comment(_comment) => {
                //todo ignore comment
            }
            Statement::For(init, cond, after, blocks) => {
                if let Some(init) = init.as_deref() {
                    self.compile_statement(init)?;
                }
                let tag = self.cur_instruction_len();

                let mut jump_if_pos = 0; //当前值随意，如果不跳出循环，永远用不到，如果能跳出循环，一定设置为了正确值
                if let Some(cond) = cond {
                    self.compile_expression(cond)?;
                    jump_if_pos = self.get_jump_if_pos()?;
                    self.compile_block_statement(blocks)?;
                    if let Some(after) = after {
                        self.compile_expression(after)?;
                    }
                }
                //始终跳转到tag处
                self.emit(Opcode::JumpAlways, vec![tag]);
                //if不成立则跳转到此处
                let after_blocks = self.cur_instruction_len();
                self.change_operand(jump_if_pos, after_blocks);
            }
            Statement::Function(name, args, blocks) => {
                let symbol = self.symbol_table.borrow_mut().define(name);
                self.compile_function_expression(Some(name.clone()), args, blocks)?;
                self.store_symbol(symbol);
            } // _ => unimplemented!(),
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
                self.add_constant_one_and_emit(Object::Integer(*value));
            }
            Expression::BoolLiteral(bool) => {
                if *bool {
                    self.emit(Opcode::True, vec![]);
                } else {
                    self.emit(Opcode::False, vec![]);
                }
            }
            Expression::StringLiteral(string) => {
                self.add_constant_one_and_emit(Object::String(string.to_string()));
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
                if op == &BinaryOperator::Assign {
                    //赋值
                    match left.as_ref() {
                        Expression::Identifier(name) => {
                            self.compile_expression(right)?;
                            self.compile_assign(name)?;
                        }
                        Expression::Index(left_expr, index_expr) => match left_expr.as_ref() {
                            Expression::Identifier(obj_name) => {
                                self.compile_expression(index_expr)?;
                                self.compile_expression(right)?;
                                self.compile_assign(obj_name)?;
                            }
                            _ => {
                                return Err(CompileError::UnsupportedBinOperation(
                                    op.clone(),
                                    *left.clone(),
                                    *right.clone(),
                                ));
                            }
                        },
                        _ => {
                            return Err(CompileError::UnsupportedBinOperation(
                                op.clone(),
                                *left.clone(),
                                *right.clone(),
                            ));
                        }
                    }
                } else {
                    self.compile_expression(left)?;
                    self.compile_expression(right)?;
                    self.compile_binary_expression(op)?;
                }
            }
            Expression::If(cond, cons, alt) => {
                self.compile_expression(cond)?;
                let jump_if_pos = self.get_jump_if_pos()?;
                //条件不成立跳转的位置
                // let jump_if_not_truthy_pos = self.emit(Opcode::JumpIfNotTruthy, vec![9999]);
                self.compile_block_statement(cons)?;
                // if语句是一个表达式有pop,
                // 编译语句块之后会多一个pop,
                // 语句块最后的值需要保存到栈上,作为if的返回值,不能在内部pop掉
                if self.last_instruction_is(Opcode::Pop) {
                    // 移除pop指令
                    self.remove_last_instruction()?;
                }
                //如果if语句块正常执行到这，就不能继续后面的else块，应该跳转到整个语句末尾
                let jump_always_pos = self.emit(Opcode::JumpAlways, vec![9999]);
                //将条件不成立跳转位置设定为当前位置
                let after_cons_pos = self.cur_instruction_len();
                self.change_operand(jump_if_pos, after_cons_pos);
                // 如果有else语句块
                if let Some(alt) = alt {
                    self.compile_block_statement(alt)?;
                    if self.last_instruction_is(Opcode::Pop) {
                        self.remove_last_instruction()?;
                    }
                } else {
                    self.emit(Opcode::Null, vec![]);
                }
                //条件语句末尾
                let final_pos = self.cur_instruction_len();
                self.change_operand(jump_always_pos, final_pos);
            }
            Expression::FunctionLiteral(args, blocks) => {
                self.compile_function_expression(None, args, blocks)?;
            }
            Expression::Call(fun, args) => {
                self.compile_expression(fun)?;
                for arg in args {
                    self.compile_expression(arg)?;
                }
                self.emit(Opcode::Call, vec![args.len()]);
            }
            _ => return Err(CompileError::UnknownExpression(expression.clone())),
        }
        Ok(())
    }
    fn compile_function_expression(
        &mut self,
        fun_name: Option<String>,
        args: &Vec<String>,
        blocks: &BlockStatement,
    ) -> CompileResult {
        self.enter_scope();
        //当前函数
        self.symbol_table.borrow_mut().define_self(fun_name.clone());
        //参数列表
        for arg in args {
            self.symbol_table.borrow_mut().define(arg);
        }
        //编译语句块
        self.compile_block_statement(blocks)?;
        //如果最后一条指令是pop，说明有返回值，改为return_value
        if self.last_instruction_is(Opcode::Pop) {
            self.remove_last_instruction()?;
            self.emit(Opcode::ReturnValue, vec![]);
        }
        //如果没有返回值
        if !self.last_instruction_is(Opcode::ReturnValue) {
            self.emit(Opcode::Return, vec![]);
        }

        let frees = &self
            .symbol_table
            .borrow()
            .free_symbols
            .iter()
            .map(|s| s.name.clone())
            .collect::<Vec<String>>();
        //自由变量个数
        let free_count = frees.len();
        //局部变量个数
        let num_locals = self.symbol_table_len();
        //编译后的函数常量
        let compiled_fn = self.leave_scope();
        //自由变量
        for name in frees {
            self.load_symbol(name)?; // emit free
        }
        let constant = Object::CompiledFunction(CompiledFunction::with_name(
            fun_name,
            Rc::new(compiled_fn),
            num_locals,
            args.len(),
        ));
        let const_index = self.add_constant(constant);
        //函数常量索引
        self.emit(Opcode::Closure, vec![const_index, free_count]);
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
    fn compile_assign(&mut self, name: &str) -> CompileResult<()> {
        let option = self.symbol_table.borrow_mut().resolve(name);
        let symbol = match option {
            None => return Err(CompileError::UndefinedIdentifier(name.to_string())),
            Some(symbol) => symbol,
        };
        self.store_symbol(symbol);
        Ok(())
    }
    fn get_jump_if_pos(&mut self) -> CompileResult<usize> {
        let jump_if_pos;
        if self.last_instruction_is(Opcode::LessThan) {
            //如果是小于比较运算
            self.remove_last_instruction()?; //移除小于指令
            jump_if_pos = self.emit(Opcode::JumpIfNotLess, vec![9999]); //替换指令
        } else {
            //不变
            jump_if_pos = self.emit(Opcode::JumpIfNotTruthy, vec![9999]);
        }
        Ok(jump_if_pos)
    }
    /// 常量池添加常量，返回常量索引
    fn add_constant(&mut self, constant: Object) -> usize {
        self.constants.push(Rc::new(constant));
        self.constants.len() - 1
    }
    //在一字节范围的常量
    fn add_constant_one_and_emit(&mut self, constant: Object) {
        self.constants.push(Rc::new(constant));
        let i = self.constants.len();
        if i <= 0xF {
            let mut o = vec![];
            let op = match i {
                1 => Opcode::Constant0,
                2 => Opcode::Constant1,
                3 => Opcode::Constant2,
                4 => Opcode::Constant3,
                5 => Opcode::Constant4,
                i => {
                    o = vec![i - 1];
                    Opcode::ConstantOne
                }
            };
            self.emit(op, o);
        } else {
            self.emit(Opcode::Constant, vec![i - 1]);
        }
        // self.constants.len() - 1
    }
    /// 指令表添加指令，返回指令开始位置
    fn add_instruction(&mut self, instruction: &mut Instructions) -> usize {
        self.scopes[self.scope_index - 1].add_instruction(instruction)
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
        let last = EmittedInstruction {
            op_code: op,
            position: pos,
        };
        self.scopes[self.scope_index - 1].set_last_instruction(last)
    }
    /// 移除上一条指令
    fn remove_last_instruction(&mut self) -> CompileResult {
        self.scopes[self.scope_index - 1].remove_last_instruction()
    }
    /// 指令替换
    fn replace_instruction(&mut self, pos: usize, new_instruction: Instructions) {
        self.scopes[self.scope_index - 1].replace_instruction(pos, new_instruction);
    }
    fn last_instruction_is(&mut self, op: Opcode) -> bool {
        if self.cur_instruction_len() == 0 {
            return false;
        }
        self.scopes[self.scope_index - 1].last_instruction_is(op)
    }
    fn cur_instruction(&mut self) -> &Instructions {
        &self.scopes[self.scope_index - 1].instructions
    }
    fn cur_instruction_len(&mut self) -> usize {
        self.cur_instruction().len()
    }

    /// 改变操作数
    fn change_operand(&mut self, op_pos: usize, operand: usize) {
        let b = self.cur_instruction()[op_pos];
        let op = Opcode::from_byte(b).expect(&format!("没有此操作码:{:4x}", b));
        let new_instruction = code::make(op, vec![operand]);
        self.replace_instruction(op_pos, new_instruction);
    }
    /// 读取符号表元素（生成一条获取该数据的指令）
    fn load_symbol(&mut self, name: &str) -> CompileResult {
        let option = self.symbol_table.borrow_mut().resolve(name);
        let symbol = match option {
            None => return Err(CompileError::UndefinedIdentifier(name.to_string())),
            Some(symbol) => symbol,
        };
        let op = match symbol.scope {
            SymbolScope::Global => {
                let i = symbol.index;
                let mut o = vec![];
                let op = match i {
                    0 => Opcode::GetGlobal0,
                    1 => Opcode::GetGlobal1,
                    2 => Opcode::GetGlobal2,
                    3 => Opcode::GetGlobal3,
                    4 => Opcode::GetGlobal4,
                    _ => {
                        o = vec![i];
                        Opcode::GetGlobal
                    }
                };
                self.emit(op, o);
                return Ok(());
            }
            SymbolScope::Local => {
                let i = symbol.index;
                let mut o = vec![];
                let op = match i {
                    0 => Opcode::GetLocal0,
                    1 => Opcode::GetLocal1,
                    2 => Opcode::GetLocal2,
                    3 => Opcode::GetLocal3,
                    4 => Opcode::GetLocal4,
                    _ => {
                        o = vec![i];
                        Opcode::GetLocal
                    }
                };
                self.emit(op, o);
                return Ok(());
            }
            SymbolScope::Builtin => Opcode::GetBuiltin,
            SymbolScope::Free => Opcode::GetFree,
            SymbolScope::Function => {
                self.emit(Opcode::CurrentClosure, vec![]);
                return Ok(());
            }
        };
        self.emit(op, vec![symbol.index]);
        Ok(())
    }
    //
    fn store_symbol(&mut self, symbol: Rc<Symbol>) {
        match symbol.scope {
            SymbolScope::Global => {
                let i = symbol.index;
                let op = match i {
                    0 => Opcode::SetGlobal0,
                    1 => Opcode::SetGlobal1,
                    2 => Opcode::SetGlobal2,
                    3 => Opcode::SetGlobal3,
                    4 => Opcode::SetGlobal4,
                    _ => {
                        self.emit(Opcode::SetLocal, vec![i]);
                        return;
                    }
                };
                self.emit(op, vec![]);
            }
            SymbolScope::Local => {
                let i = symbol.index;
                let op = match i {
                    0 => Opcode::SetLocal0,
                    1 => Opcode::SetLocal1,
                    2 => Opcode::SetLocal2,
                    3 => Opcode::SetLocal3,
                    4 => Opcode::SetLocal4,
                    _ => {
                        self.emit(Opcode::SetLocal, vec![i]);
                        return;
                    }
                };
                self.emit(op, vec![]);
            }
            _ => unimplemented!(),
        };
    }

    fn symbol_table_len(&self) -> usize {
        self.symbol_table.borrow().num_definitions
    }
}

impl ByteCode {
    pub fn new(instructions: Instructions, constants: Constants) -> Self {
        Self {
            instructions,
            constants,
        }
    }
}
