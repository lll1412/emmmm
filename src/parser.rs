use crate::ast::{Expression, Program, Statement};
use crate::lexer::Lexer;
use crate::token::Token;

#[derive(Debug)]
enum ParserError {
    UnknownError(Token),
    /// expected, actual
    Expect(Token, Token),
    ExpectAssign(Token),
    ExpectLet(Token),
    ExpectedIdentifier(Token),
    ExpectedInteger(Token),
    ParseInt(String),
}

type Result<T> = std::result::Result<T, ParserError>;

pub struct Parser {
    lexer: Lexer,
    token: Token,
    peek_token: Token,
    errors: Vec<ParserError>,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        let mut parser = Self {
            lexer,
            token: Token::Eof,
            peek_token: Token::Eof,
            errors: vec![],
        };
        parser.next_token();
        parser.next_token();
        parser
    }

    pub fn from(input: String) -> Self {
        let lexer = Lexer::new(input);
        Parser::new(lexer)
    }

    pub fn parse_program(&mut self) -> Program {
        let mut statements = vec![];
        while self.token != Token::Eof {
            match self.parse_statement() {
                Ok(statement) => statements.push(statement),
                Err(err) => {
                    self.errors.push(err);
                    while self.token != Token::Semicolon {
                        self.next_token();
                    }
                }
            }
            self.next_token();
        }
        Program { statements }
    }

    fn next_token(&mut self) {
        self.token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match &self.token {
            Token::Let => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            token => Err(ParserError::UnknownError(token.clone())),
        }
    }
    /// let identifier = expression
    fn parse_let_statement(&mut self) -> Result<Statement> {
        // cur_token is let
        //ident
        let name;
        match &self.peek_token {
            Token::Ident(ident) => {
                name = ident.clone();
                self.next_token();
            }
            token => return Err(ParserError::ExpectedIdentifier(token.clone())),
        }
        //=
        self.expect_peek(Token::Assign, ParserError::ExpectAssign)?;
        //expr
        self.next_token();
        let expression = self.parse_expression()?;
        let statement = Statement::Let(name, expression);
        Ok(statement)
    }
    /// return;
    /// return expr;
    fn parse_return_statement(&mut self) -> Result<Statement> {
        self.next_token();
        if self.token == Token::Semicolon {
            Ok(Statement::Return(None))
        } else {
            let expression = self.parse_expression()?;
            Ok(Statement::Return(Some(expression)))
        }
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        match &self.token {
            Token::Int(int) => {
                let result = self.parse_integer_literal(int.to_string());
                self.next_token();
                result
            }
            token => Err(ParserError::ExpectedInteger(token.clone())),
        }
    }

    fn parse_integer_literal(&self, int: String) -> Result<Expression> {
        match int.parse() {
            Ok(val) => Ok(Expression::IntegerLiteral(val)),
            Err(_) => Err(ParserError::ParseInt(int.to_string())),
        }
    }

    fn expect_peek(&mut self, expected: Token, error: fn(Token) -> ParserError) -> Result<()> {
        if self.peek_token == expected {
            self.next_token();
            Ok(())
        } else {
            Err(error(self.peek_token.clone()))
        }
    }

    fn errors(&self) -> &[ParserError] {
        &self.errors
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn test_let_statements() {
        let input = r"
    let x = 1;
    let y = 10;
    let z = 838383 ;
        ";
        let lexer = Lexer::new(String::from(input));
        let parser = Parser::new(lexer);
        // println!("sattements: {:#?}", statements);
        test_let_statement(parser)
    }

    #[test]
    fn test_return_statement() {
        let input = r"
        return ;
        return 123;
        ";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();
        println!("statements: {:#?}", program.statements);
        assert_eq!(program.statements.len(), 2);

        check_parser_error(parser);
    }

    fn test_let_statement(mut parser: Parser) {
        parser.parse_program();
        check_parser_error(parser);
    }

    fn check_parser_error(parser: Parser) {
        let errors = parser.errors();
        let len = errors.len();
        if len == 0 {
            return;
        }
        let msg = format!("parser has {} errors", len);
        dbg!(msg);
        for err in errors {
            dbg!(err);
        }
        panic!()
    }
}
