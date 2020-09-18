#[cfg(test)]
mod tests {
    use crate::core::{ast::*, lexer::*, parser::Parser};
    use BinaryOperator::*;
    use Expression::*;

    #[test]
    fn test_call_expression() {
        let tests = [
            (
                "add(1, 2)".to_string(),
                Expression::Call(
                    Box::new(Expression::Identifier("add".to_string())),
                    vec![Expression::IntLiteral(1), Expression::IntLiteral(2)],
                ),
            ),
            (
                "add(1 + 2, 2 * 3)".to_string(),
                Expression::Call(
                    Box::new(Expression::Identifier("add".to_string())),
                    vec![
                        Expression::Binary(
                            BinaryOperator::Plus,
                            Box::new(Expression::IntLiteral(1)),
                            Box::new(Expression::IntLiteral(2)),
                        ),
                        Expression::Binary(
                            BinaryOperator::Mul,
                            Box::new(Expression::IntLiteral(2)),
                            Box::new(Expression::IntLiteral(3)),
                        ),
                    ],
                ),
            ),
        ];
        test_parse(&tests);
    }

    fn test_parse(data: &[(String, Expression)]) {
        for (input, expected) in data {
            let mut parser = Parser::from(input.to_string());
            let program = parser.parse_program();
            println!("{}", program);
            check_parser_error(parser);
            assert_eq!(program.to_string(), expected.to_string());
        }
    }

    #[test]
    fn test_function_literal_parsing() {
        let tests = [
            (
                "fun(x, y) { x=1; x + y; }".to_string(),
                Expression::Fun(
                    vec!["x".to_string(), "y".to_string()],
                    BlockStatement {
                        statements: vec![
                            Statement::Expression(Expression::Binary(
                                BinaryOperator::Assign,
                                Box::new(Expression::Identifier("x".to_string())),
                                Box::new(Expression::IntLiteral(1)),
                            )),
                            Statement::Expression(Expression::Binary(
                                BinaryOperator::Plus,
                                Box::new(Expression::Identifier("x".to_string())),
                                Box::new(Expression::Identifier("y".to_string())),
                            )),
                        ],
                    },
                ),
            ),
            (
                "fun() { return 1;}".to_string(),
                Expression::Fun(
                    vec![],
                    BlockStatement {
                        statements: vec![Statement::Return(Some(Expression::IntLiteral(1)))],
                    },
                ),
            ),
        ];
        test_parse(&tests);
    }

    #[test]
    fn test_if_expression() {
        let (input, expected) = (
            r"if (x < y) { let x = 1;return x; } else { y }",
            Expression::If(
                Box::new(Expression::Binary(
                    BinaryOperator::Lt,
                    Box::new(Expression::Identifier("x".to_string())),
                    Box::new(Expression::Identifier("y".to_string())),
                )),
                BlockStatement {
                    statements: vec![
                        Statement::Let("x".to_string(), Expression::IntLiteral(1)),
                        Statement::Return(Some(Expression::Identifier("x".to_string()))),
                    ],
                },
                Some(BlockStatement {
                    statements: vec![Statement::Expression(Expression::Identifier(
                        "y".to_string(),
                    ))],
                }),
            ),
        );
        let mut parser = Parser::from(input.to_string());
        let program = parser.parse_program();
        println!("{}", program.to_string());
        check_parser_error(parser);
        assert_eq!(expected.to_string(), program.to_string());
    }

    #[test]
    fn test_operator_precedence_parsing() {
        let tests = vec![
            ("1 + (2 + 3) + 4", "((1 + (2 + 3)) + 4)"),
            ("(5+5)*2", "((5 + 5) * 2)"),
            ("2/(5+5)", "(2 / (5 + 5))"),
            ("-(5+5)", ("(-(5 + 5))")),
            ("!(true == true)", "(!(true == true))"),
            ("true", "true"),
            ("false", "false"),
            ("3>5 == false", "((3 > 5) == false)"),
            ("3<5 == true", "((3 < 5) == true)"),
            ("-a * b", "((-a) * b)"),
            ("! -a", "(!(-a))"),
            ("a+b+c", "((a + b) + c)"),
            ("a+b-c", "((a + b) - c)"),
            ("a*b*c", "((a * b) * c)"),
            ("a*b/c", "((a * b) / c)"),
            ("a+b/c", "(a + (b / c))"),
            ("a+b*c+d/e-f", "(((a + (b * c)) + (d / e)) - f)"),
            ("3+4;-5*5", "(3 + 4)((-5) * 5)"),
            ("5>4 == 3<4", "((5 > 4) == (3 < 4))"),
            ("5<4 != 3>4", "((5 < 4) != (3 > 4))"),
            ("3+4*5 == 3*1+4*5", "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))"),
            (
                "add(a + b + c * d / f + g)",
                "add((((a + b) + ((c * d) / f)) + g))",
            ),
        ];
        for (input, expect) in tests {
            let mut parser = Parser::from(String::from(input));
            let program = parser.parse_program();
            println!("{}", program);
            check_parser_error(parser);
            assert_eq!(expect, program.to_string());
        }
    }

    /// 测试二元表达式
    #[test]
    fn test_parsing_infix_expression() {
        let infix_tests = vec![
            ("true == true;", BoolLiteral(true), Eq, BoolLiteral(true)),
            (
                "true != false;",
                BoolLiteral(true),
                NotEq,
                BoolLiteral(false),
            ),
            (
                "false == false;",
                BoolLiteral(false),
                Eq,
                BoolLiteral(false),
            ),
            ("5 + 5;", IntLiteral(5), Plus, IntLiteral(5)),
            ("5 - 5;", IntLiteral(5), Minus, IntLiteral(5)),
            ("5 * 5;", IntLiteral(5), Mul, IntLiteral(5)),
            ("5 / 5;", IntLiteral(5), Div, IntLiteral(5)),
            ("5 > 5;", IntLiteral(5), Gt, IntLiteral(5)),
            ("5 < 5;", IntLiteral(5), Lt, IntLiteral(5)),
            ("5 == 5;", IntLiteral(5), Eq, IntLiteral(5)),
            ("5 != 5;", IntLiteral(5), NotEq, IntLiteral(5)),
        ];
        for (input, left_val, operator, right_val) in infix_tests {
            let mut parser = Parser::from(input.to_string());
            let program = parser.parse_program();
            check_parser_error(parser);
            assert_eq!(
                program.statements[0],
                Statement::Expression(Expression::Binary(
                    operator,
                    Box::new(left_val),
                    Box::new(right_val),
                ))
            );
        }
    }

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
        println!("statements:\n{}", program);
        assert_eq!(program.statements.len(), 2);

        check_parser_error(parser);
    }

    #[test]
    fn test_identifier_expression() {
        let input = String::from("1;");
        let mut parser = Parser::from(input);
        let program = parser.parse_program();
        check_parser_error(parser);
        assert_eq!(
            1,
            program.statements.len(),
            "program has not enough statements. got [{}]",
            program.statements.len()
        );
        if let Statement::Expression(expr) = &program.statements[0] {
            if let IntLiteral(val) = expr {
                if *val != 1i64 {
                    panic!("ident value not {}, got={}", 1, val)
                };
            } else {
                panic!("expr is not identifier. got ={}", expr);
            }
        } else {
            panic!(
                "program statements[0] is not ExpressionStatement. got = {}",
                &program.statements[0]
            )
        }
    }

    #[test]
    fn test_parsing_prefix_expression() {
        let prefix_tests = vec![
            ("!5;", UnaryOperator::Not, IntLiteral(5)),
            ("-15;", UnaryOperator::Neg, IntLiteral(15)),
        ];
        for (input, expected_operator, expected_expr) in prefix_tests {
            let mut parser = Parser::from(input.to_string());
            let program = parser.parse_program();
            check_parser_error(parser);
            assert_eq!(
                program.statements,
                vec![Statement::Expression(Expression::Unary(
                    expected_operator,
                    Box::new(expected_expr),
                ))]
            );
        }
    }

    /// 辅助函数 测试let
    fn test_let_statement(mut parser: Parser) {
        let program = parser.parse_program();
        check_parser_error(parser);
    }

    /// 辅助函数 检查错误
    fn check_parser_error(parser: Parser) {
        let errors = parser.errors();
        let len = errors.len();
        if len == 0 {
            return;
        }
        // let msg = format!("parser has {} errors", len);
        println!("Parser has {} errors.", len);
        for error in errors {
            println!("{:?}", error);
        }
        panic!()
    }
}
