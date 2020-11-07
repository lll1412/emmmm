use crate::parser::{
    {BinaryParseFn, Parser, ParserError, ParseResult, Precedence, UnaryParseFn},
    base::ast::{BinaryOperator, Expression, Program, Statement, UnaryOperator},
    base::token::Token,
    lexer::Lexer,
};
use crate::parser::base::ast::BlockStatement;

impl Parser {
    // 从Lexer构建Parser
    pub fn new(lexer: Lexer) -> Parser {
        let mut parser = Parser {
            lexer,
            token: Token::Eof,
            peek_token: Token::Eof,
            errors: vec![],
        };
        parser.next_token();
        parser.next_token();
        parser
    }
    /// 从字符串构建Parser
    pub fn from(input: &str) -> Self {
        let lexer = Lexer::new(input);
        Parser::new(lexer)
    }
    /// 解析程序
    pub fn parse_program(&mut self) -> Program {
        let mut statements = vec![];
        while self.has_next() {
            match self.parse_statement() {
                Ok(statement) => statements.push(statement),
                Err(err) => self.errors.push(err),
            }
            self.next_token()
        }
        Program { statements }
    }
    /// 读取下一个Token
    pub fn next_token(&mut self) {
        self.token = self.peek_token.clone();
        self.peek_token = self.lexer.parse_token();
    }
    /// 解析语句
    fn parse_statement(&mut self) -> ParseResult<Statement> {
        match &self.token {
            Token::Let => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            Token::Comment(comment) => Ok(Statement::Comment(comment.to_string())),
            Token::For => self.parse_for_statement(),
            Token::Function => self.parse_function_statement(),
            _ => self.parse_expression_statement(),
        }
    }
    /// 解析let语句
    ///
    /// let identifier = expression;
    fn parse_let_statement(&mut self) -> ParseResult<Statement> {
        // cur_token is let
        //ident
        let name = match &self.peek_token {
            Token::Ident(ident) => ident.clone(),
            token => return Err(ParserError::ExpectedIdentifier(token.clone())),
        };
        //eat let
        self.next_token();
        // expect '=' then eat ident
        self.expect_peek(Token::Assign, ParserError::ExpectedAssign)?;
        //expr
        self.next_token(); //eat =
        let expression = self.parse_expression(Precedence::Lowest)?;
        let result;
        //如果是let name = fn() {} ,转换为 fn name(){}
        if let Expression::FunctionLiteral(params, block) = expression {
            result = Statement::Function(name.clone(), params, block)
        } else {
            result = Statement::Let(name, expression)
        }
        if self.peek_token == Token::Semicolon {
            self.next_token(); //eat ;
        }
        Ok(result)
    }
    /// 解析return语句
    ///
    /// 1. return;
    /// 2. return expr;
    fn parse_return_statement(&mut self) -> ParseResult<Statement> {
        self.next_token(); //eat return
        let mut option = None;
        if self.token != Token::Semicolon {
            let expression = self.parse_expression(Precedence::Lowest)?;
            option = Some(expression);
        }
        if self.peek_token == Token::Semicolon {
            self.next_token(); //eat ;
        }
        Ok(Statement::Return(option))
    }
    /// 解析for语句
    /// for (init; cond; after) { block_statement }
    fn parse_for_statement(&mut self) -> ParseResult<Statement> {
        self.next_token();
        let mut init = None;
        if self.peek_token != Token::Semicolon {
            self.next_token();
            init = Some(Box::new(self.parse_let_statement()?));
        }
        self.next_token();
        let mut cond = None;
        if self.peek_token != Token::Semicolon {
            cond = Some(self.parse_expression(Precedence::Lowest)?);
        }
        self.next_token();
        let mut after = None;
        if self.peek_token != Token::Rparen {
            self.next_token();
            after = Some(self.parse_expression(Precedence::Lowest)?);
        }
        self.next_token();
        if self.peek_token != Token::Rbrace {
            self.next_token();
        }
        let blocks = self.parse_block_statement()?;
        let for_statement = Statement::For(init, cond, after, blocks);
        Ok(for_statement)
    }
    /// 解析函数语句 fn ident(args..) { blocks }
    fn parse_function_statement(&mut self) -> ParseResult<Statement> {
        //cur token: fn
        if let Token::Ident(name) = self.peek_token.clone() {
            self.next_token(); // eat fun
            self.next_token(); // eat ident
            let params = self.parse_function_parameters()?;
            self.expect_peek_is(Token::Lbrace)?; // eat )
            let blocks = self.parse_block_statement()?;
            Ok(Statement::Function(name, params, blocks))
        } else {
            self.parse_expression_statement()
        }
    }

    /// 解析表达式语句
    ///
    /// expr;
    fn parse_expression_statement(&mut self) -> ParseResult<Statement> {
        let expression = self.parse_expression(Precedence::Lowest);
        if self.peek_token == Token::Semicolon {
            self.next_token();
        }
        expression.map(Statement::Expression)
    }
    /// 解析语句块
    ///
    /// {
    ///   statement;
    ///   statement;
    /// }
    fn parse_block_statement(&mut self) -> ParseResult<BlockStatement> {
        self.next_token(); // eat {
        let mut statements = vec![];
        while self.token != Token::Rbrace && self.token != Token::Eof {
            let statement = self.parse_statement()?;
            statements.push(statement);
            self.next_token();
        }
        Ok(BlockStatement { statements })
    }
    /// 解析表达式
    fn parse_expression(&mut self, precedence: Precedence) -> ParseResult {
        let unary = self
            .unary_parse_fn()
            .ok_or_else(|| ParserError::ExpectedUnaryOp(self.token.clone()))?;
        let mut left_expr = unary(self)?;
        while self.peek_token != Token::Semicolon
            && precedence < self.binary_token(&self.peek_token).0
        {
            if let Some(binary) = self.binary_parse_fn() {
                self.next_token();
                left_expr = binary(self, left_expr)?;
            } else {
                break;
            }
        }
        Ok(left_expr)
    }
    ///解析分组表达式
    fn parse_grouped_expression(&mut self) -> ParseResult {
        self.next_token(); // eat (
        let expr = self.parse_expression(Precedence::Lowest)?;
        self.expect_peek(Token::Rparen, |tk| ParserError::Expected(Token::Rparen, tk))?;
        Ok(expr)
    }
    /// ## 解析数组字面量
    fn parse_array_literal(&mut self) -> ParseResult {
        let args = self.parse_comma_arguments(Token::Rbracket)?;
        Ok(Expression::ArrayLiteral(args))
    }
    /// ## 解析映射表字面量
    fn parse_hash_literal(&mut self) -> ParseResult {
        let mut v = vec![];
        self.next_token(); // eat {
        while self.token != Token::Rbrace {
            if self.token != Token::Comma {
                let key = self.parse_expression(Precedence::Lowest)?;
                self.expect_peek_is(Token::Colon)?; // cu token is :
                self.next_token(); // eat :
                let val = self.parse_expression(Precedence::Lowest)?;
                v.push((key, val));
            }
            self.next_token();
        }
        Ok(Expression::HashLiteral(v))
    }
    /// 解析函数声明参数列表
    fn parse_function_parameters(&mut self) -> ParseResult<Vec<String>> {
        self.next_token(); // eat (
        let mut params = vec![];
        while self.token != Token::Rparen {
            if self.token != Token::Comma {
                // if not eq )
                params.push(self.parse_identifier_string()?);
            }
            self.next_token(); //eat param or colon
        }
        Ok(params)
    }
    /// 解析函数调用表达式
    fn parse_call_expression(&mut self, function: Expression) -> ParseResult {
        let arguments = self.parse_comma_arguments(Token::Rparen)?;
        Ok(Expression::Call(Box::new(function), arguments))
    }
    fn parse_index_expression(&mut self, left: Expression) -> ParseResult {
        self.next_token();
        let index = self.parse_expression(Precedence::Lowest)?;
        self.next_token();
        Ok(Expression::Index(Box::new(left), Box::new(index)))
    }
    /// 解析函数调用参数列表
    fn parse_comma_arguments(&mut self, end_token: Token) -> ParseResult<Vec<Expression>> {
        self.next_token(); // eat start_token
        let mut arguments = vec![];
        while self.token != end_token {
            if self.token != Token::Comma {
                let expr = self.parse_expression(Precedence::Lowest)?;
                arguments.push(expr);
            }
            self.next_token(); // eat expr or ,
        }
        Ok(arguments)
    }
    /// 解析函数表达式
    fn parse_function_expression(&mut self) -> ParseResult {
        //cur token: fn
        self.expect_peek_is(Token::Lparen)?; // eat fun
        let params = self.parse_function_parameters()?;
        self.expect_peek_is(Token::Lbrace)?; // eat )
        let blocks = self.parse_block_statement()?;
        Ok(Expression::FunctionLiteral(params, blocks))
    }
    ///解析if表达式
    fn parse_if_expression(&mut self) -> ParseResult {
        let has_bracket = self.peek_token == Token::Lparen;
        if has_bracket {
            self.next_token();
        }
        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest)?;
        if has_bracket {
            self.expect_peek_is(Token::Rparen)?;
        }
        self.expect_peek_is(Token::Lbrace)?;
        let consequence = self.parse_block_statement()?;
        self.next_token(); // eat }

        // parse else block
        let mut alternative = None;
        if self.token == Token::Else {
            self.next_token(); //eat else
            let else_block = self.parse_block_statement()?;
            alternative = Some(else_block);
        }
        Ok(Expression::If(
            Box::new(condition),
            consequence,
            alternative,
        ))
    }
    /*一元表达式相关*/
    /// 解析一元表达式
    fn parse_unary_expression(&mut self) -> ParseResult {
        let operator = self.unary_token(&self.token)?;
        self.next_token();
        let expr = self.parse_expression(Precedence::Prefix)?;
        Ok(Expression::Unary(operator, Box::new(expr)))
    }
    /// 一元表达式函数
    fn unary_parse_fn(&self) -> Option<UnaryParseFn> {
        match self.token {
            Token::Ident(_) => Some(Parser::parse_identifier),

            Token::Int(_) => Some(Parser::parse_integer_literal),
            Token::Float(_) => Some(Parser::parse_float_literal),
            Token::True => Some(Parser::parse_boolean),
            Token::False => Some(Parser::parse_boolean),
            Token::String(_) => Some(Parser::parse_string_literal),

            Token::Bang => Some(Parser::parse_unary_expression),
            Token::Minus => Some(Parser::parse_unary_expression),

            Token::Lparen => Some(Parser::parse_grouped_expression),
            Token::Lbracket => Some(Parser::parse_array_literal),
            Token::Lbrace => Some(Parser::parse_hash_literal),

            Token::If => Some(Parser::parse_if_expression),
            Token::Function => Some(Parser::parse_function_expression),

            _ => None,
        }
    }
    /// 一元表达式操作符
    fn unary_token(&self, token: &Token) -> ParseResult<UnaryOperator> {
        match token {
            Token::Bang => Ok(UnaryOperator::Not),
            Token::Minus => Ok(UnaryOperator::Neg),
            other => Err(ParserError::ExpectedUnaryOp(other.clone())),
        }
    }

    /*二元表达式相关*/
    /// 解析二元表达式
    fn parse_binary_expression(&mut self, left: Expression) -> ParseResult {
        let (precedence, operator) = self.binary_token(&self.token);
        let operator = operator.ok_or_else(|| ParserError::ExpectedBinaryOp(self.token.clone()))?;
        self.next_token(); //eat op
        let right = self.parse_expression(precedence)?;
        let expression = Expression::Binary(operator, Box::new(left), Box::new(right));
        Ok(expression)
    }
    ///二元表达式函数
    fn binary_parse_fn(&self) -> Option<BinaryParseFn> {
        match self.peek_token {
            Token::Assign
            | Token::Plus
            | Token::Minus
            | Token::Slash
            | Token::Asterisk
            | Token::Eq
            | Token::NotEq
            | Token::Lt
            | Token::Gt => Some(Parser::parse_binary_expression),
            Token::Lparen => Some(Parser::parse_call_expression),
            Token::Lbracket => Some(Parser::parse_index_expression),
            _ => None,
        }
    }
    /// 二元表达式操作符
    fn binary_token(&self, token: &Token) -> (Precedence, Option<BinaryOperator>) {
        match token {
            Token::Assign => (Precedence::Assign, Some(BinaryOperator::Assign)),
            Token::Eq => (Precedence::Equals, Some(BinaryOperator::Eq)),
            Token::NotEq => (Precedence::Equals, Some(BinaryOperator::NotEq)),
            Token::Lt => (Precedence::LessGreater, Some(BinaryOperator::Lt)),
            Token::Le => (Precedence::LessGreater, Some(BinaryOperator::Le)),
            Token::Gt => (Precedence::LessGreater, Some(BinaryOperator::Gt)),
            Token::Ge => (Precedence::LessGreater, Some(BinaryOperator::Ge)),
            Token::Plus => (Precedence::Sum, Some(BinaryOperator::Plus)),
            Token::Minus => (Precedence::Sum, Some(BinaryOperator::Minus)),
            Token::Slash => (Precedence::Product, Some(BinaryOperator::Div)),
            Token::Asterisk => (Precedence::Product, Some(BinaryOperator::Mul)),
            Token::Lparen => (Precedence::Call, None),
            Token::Lbracket => (Precedence::Index, None),
            _ => (Precedence::Lowest, None),
        }
    }

    /*基本解析*/
    /// 解析标识符
    fn parse_identifier(&mut self) -> ParseResult {
        self.parse_identifier_string().map(Expression::Identifier)
    }
    /// 解析标识符字符串
    fn parse_identifier_string(&mut self) -> ParseResult<String> {
        if let Token::Ident(id) = &self.token {
            Ok(id.to_string())
        } else {
            Err(ParserError::ExpectedIdentifier(self.token.clone()))
        }
    }
    /*数据类型解析*/
    /// 解析整型字面量
    fn parse_integer_literal(&mut self) -> ParseResult {
        if let Token::Int(int) = &self.token {
            match int.parse() {
                Ok(val) => Ok(Expression::IntLiteral(val)),
                Err(_) => Err(ParserError::ParseInt(int.to_string())),
            }
        } else {
            Err(ParserError::ExpectedInteger(self.token.clone()))
        }
    }
    /// 解析浮点数字面量
    fn parse_float_literal(&mut self) -> ParseResult {
        if let Token::Float(float) = &self.token {
            match float.parse() {
                Ok(val) => Ok(Expression::FloatLiteral(val)),
                Err(_) => Err(ParserError::ParseInt(float.to_string())),
            }
        } else {
            Err(ParserError::ExpectedFloat(self.token.clone()))
        }
    }
    /// 解析字符串字面量
    fn parse_string_literal(&mut self) -> ParseResult {
        if let Token::String(s) = &self.token {
            Ok(Expression::StringLiteral(s.to_string()))
        } else {
            Err(ParserError::ExpectedString(self.token.clone()))
        }
    }
    ///解析布尔值
    fn parse_boolean(&mut self) -> ParseResult {
        match &self.token {
            Token::True => Ok(Expression::BoolLiteral(true)),
            Token::False => Ok(Expression::BoolLiteral(false)),
            tk => Err(ParserError::ExpectedBoolean(tk.clone())),
        }
    }
    /*其他*/
    /// 断言Token是否为期待值
    fn expect_peek(&mut self, expected: Token, error: fn(Token) -> ParserError) -> ParseResult<()> {
        if self.peek_token == expected {
            self.next_token();
            Ok(())
        } else {
            Err(error(self.peek_token.clone()))
        }
    }
    fn expect_peek_is(&mut self, expected: Token) -> ParseResult<()> {
        if self.peek_token == expected {
            self.next_token();
            Ok(())
        } else {
            Err(ParserError::Expected(expected, self.peek_token.clone()))
        }
    }
    /// 判断是否还有Token
    fn has_next(&self) -> bool {
        self.token != Token::Eof
    }
    /// 返回错误信息
    pub fn errors(&self) -> &[ParserError] {
        &self.errors
    }
}
