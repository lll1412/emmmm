#[cfg(test)]
mod tests {
    use crate::parser::lexer::token::Token;
    use crate::parser::lexer::Lexer;
    use Token::Plus;
    #[test]
    fn print_token() {
        let input = r#"
        let add = fn(x, y) {
                    a + b;
                  };
        add(2,4);
            "#;
        let mut lexer = Lexer::new(input);
        let mut token = lexer.parse_token();
        while token != Token::Eof {
            print!("{:?} ", token);
            token = lexer.parse_token();
        }
    }
    #[test]
    fn test_next_token() {
        let input = r#"
    let s = "hello \n \"world\"";
    let five = 5.1;
    let ten = 10;
    let add = fn(x, y) {
        return x + y;
    };
    let result = add(five, ten);
    !-/*5;
    5 < 10 > 5;
    if 5 < 10 {
        return true;
    } else {
        return false;
    }
    [1,2]
"#;
        // let input = input.split("").collect();

        let tests = [
            Token::Let,
            Token::Ident("s".to_string()),
            Token::Assign,
            Token::String("hello \n \"world\"".to_string()),
            Token::Semicolon,
            Token::Let,
            Token::Ident("five".to_string()),
            Token::Assign,
            Token::Float("5.1".to_string()),
            Token::Semicolon,
            Token::Let,
            Token::Ident("ten".to_string()),
            Token::Assign,
            Token::Int(String::from("10")),
            Token::Semicolon,
            Token::Let,
            Token::Ident("add".to_string()),
            Token::Assign,
            Token::Function,
            Token::Lparen,
            Token::Ident("x".parse().unwrap()),
            Token::Comma,
            Token::Ident("y".to_string()),
            Token::Rparen,
            Token::Lbrace,
            Token::Return,
            Token::Ident("x".to_string()),
            Plus,
            Token::Ident("y".to_string()),
            Token::Semicolon,
            Token::Rbrace,
            Token::Semicolon,
            Token::Let,
            Token::Ident("result".to_string()),
            Token::Assign,
            Token::Ident("add".to_string()),
            Token::Lparen,
            Token::Ident("five".to_string()),
            Token::Comma,
            Token::Ident("ten".to_string()),
            Token::Rparen,
            Token::Semicolon,
            Token::Bang,
            Token::Minus,
            Token::Slash,
            Token::Asterisk,
            Token::Int("5".to_string()),
            Token::Semicolon,
            Token::Int("5".to_string()),
            Token::Lt,
            Token::Int("10".to_string()),
            Token::Gt,
            Token::Int("5".to_string()),
            Token::Semicolon,
            Token::If,
            Token::Int("5".to_string()),
            Token::Lt,
            Token::Int("10".to_string()),
            Token::Lbrace,
            Token::Return,
            Token::True,
            Token::Semicolon,
            Token::Rbrace,
            Token::Else,
            Token::Lbrace,
            Token::Return,
            Token::False,
            Token::Semicolon,
            Token::Rbrace,
            Token::Lbracket,
            Token::Int("1".to_string()),
            Token::Comma,
            Token::Int("2".to_string()),
            Token::Rbracket,
            Token::Eof,
        ];

        let mut lexer = Lexer::new(input);
        for (i, expected_token) in tests.iter().enumerate() {
            let token = lexer.parse_token();
            // println!("{:?}", token);
            assert_eq!(&token, expected_token, "tests[{}] - token", i);
        }
    }

    #[test]
    fn test_token() {
        let input = r"
        5 + 5;
            5 + 5;
            5*5;
        ";
        let mut lexer = Lexer::new(input);
        let tests = [
            Token::Int("5".to_string()),
            Token::Plus,
            Token::Int("5".to_string()),
            Token::Semicolon,
            Token::Int("5".to_string()),
            Token::Plus,
            Token::Int("5".to_string()),
            Token::Semicolon,
            Token::Int("5".to_string()),
            Token::Asterisk,
            Token::Int("5".to_string()),
            Token::Semicolon,
        ];
        for tk in tests.iter() {
            assert_eq!(tk, &lexer.parse_token())
        }
    }
}
