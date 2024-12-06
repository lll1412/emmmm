#[cfg(test)]
mod tests {
    use crate::parser::ast::{
        BinaryOperator, BlockStatement, Expression, Statement, UnaryOperator,
    };
    use crate::parser::Parser;
    use crate::eval::evaluator;
    use crate::eval::evaluator::{Env, EvalResult};
    use crate::eval::Environment;
    use crate::object::{HashKey, Object, RuntimeError};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::fmt::{Debug, Display};
    use std::rc::Rc;
    #[test]
    fn for_statement() {
        let inputs = [(
            r#"
            let r = 0;
            for (let i = 0; i < 10; i = i + 1) {
                r = r + 1;
            }
            r
        "#,
            Object::Integer(10),
        )];
        check_input(&inputs);
    }
    #[test]
    fn test_hash() {
        let mut map = HashMap::new();
        map.insert(
            HashKey::from_object(&Object::String("one".to_string())).unwrap(),
            Object::Integer(1),
        );
        map.insert(
            HashKey::from_object(&Object::String("two".to_string())).unwrap(),
            Object::Integer(2),
        );
        map.insert(
            HashKey::from_object(&Object::String("three".to_string())).unwrap(),
            Object::Integer(3),
        );
        let inputs = &[
            (
                r#"{ "one": 1, "two": 2, "three": 3 }"#,
                Object::Hash(RefCell::new(map)),
            ),
            (
                r#"let map = { "one": 1, "two": 2, "three": 3 }; map["one"] = 10; map["one"]"#,
                Object::Integer(10),
            ),
        ];
        check_input(inputs);
    }
    #[test]
    fn test_array_index() {
        let inputs = &[
            ("[1, 2 * 2, 3][0]", Object::Integer(1)),
            ("[1, 2 * 2, 3][1]", Object::Integer(4)),
            ("[1, 2 * 2, 3][2]", Object::Integer(3)),
            ("[1, 2 * 2, fn(a) { a }][3]", Object::Null),
            ("let arr = [1,2,3]; arr[1]", Object::Integer(2)),
            ("let arr = [1,2,3]; arr[0] = 5; arr[0]", Object::Integer(5)),
        ];
        check_input(inputs);
    }

    #[test]
    fn test_array() {
        let inputs = [(
            "[1, 2*2, 3+3]",
            Object::Array(RefCell::new(vec![
                Object::Integer(1),
                Object::Integer(4),
                Object::Integer(6),
            ])),
        )];
        check_input(&inputs);
    }

    #[test]
    fn test_builtin_function() {
        let inputs = [
            (r#"len("")"#, Object::Integer(0)),
            (r#"len("four")"#, Object::Integer(4)),
            (r#"len("hello world")"#, Object::Integer(11)),
        ];
        check_input(&inputs);
        let inputs = [
            (
                r#"len(1)"#,
                RuntimeError::BuiltinUnSupportedArg("len".to_string(), vec![Object::Integer(1)]),
            ),
            (
                r#"len("one", "two")"#,
                RuntimeError::BuiltinIncorrectArgNum(1, 2),
            ),
        ];
        check_error(&inputs);
    }

    #[test]
    fn test_closure() {
        let inputs = [(
            r"
            let new_adder = fn(x) {
                let garbage = 11;
                fn(y) { x+ y }
            }
            let add = new_adder(2)
            add(3)
            ",
            Object::Integer(5),
        )];
        check_input(&inputs);
    }

    #[test]
    fn test_function_application() {
        let inputs = [
            ("let identify = fn(x) {x}; identify(5)", Object::Integer(5)),
            (
                "let identify = fn(x) {return x;};identify(5);",
                Object::Integer(5),
            ),
            ("let double = fn(x) {x * 2};double(5)", Object::Integer(10)),
            (
                "let add = fn(a,b) { a + b; }; add(2,4);",
                Object::Integer(6),
            ),
            (
                "let add = fn(x, y) {x+y}; add(2, add(5, 5));",
                Object::Integer(12),
            ),
            (r"
            let add = fn(a, b) { a + b }
            let applyFunc = fn(a, b, func) { func(a, b) }
            applyFunc(2, 2, add)
            ",
                Object::Integer(4)),
        ];
        check_input(&inputs);
    }

    #[test]
    fn test_function_object() {
        let inputs = [(
            "fn(x) {x+2;}",
            Object::Function(None,
                vec!["x".to_string()],
                BlockStatement {
                    statements: vec![Statement::Expression(Expression::Binary(
                        BinaryOperator::Plus,
                        Box::new(Expression::Identifier("x".to_string())),
                        Box::new(Expression::IntLiteral(2)),
                    ))],
                },
                Rc::new(RefCell::new(Environment::new())),
            ),
        )];
        check_input(&inputs);
    }

    #[test]
    fn test_assign_expression() {
        let inputs = [("let a = 1; a = 2; a", Object::Integer(2))];
        check_input(&inputs);
    }

    #[test]
    fn test_let_statement() {
        let inputs = [
            ("let a = 5;a", Object::Integer(5)),
            ("let a = 5*5;a", Object::Integer(25)),
            ("let b = 5;let a = b;a", Object::Integer(5)),
            (
                "let a = 5;let b = a;let c = a + b + 5; c",
                Object::Integer(15),
            ),
        ];
        check_input(&inputs);
    }

    #[test]
    fn test_error_handling() {
        let inputs = [
            (
                "foobar",
                RuntimeError::IdentifierNotFound("foobar".to_string()),
            ),
            (
                "5+true",
                RuntimeError::TypeMismatch(
                    BinaryOperator::Plus,
                    Object::Integer(5),
                    Object::Boolean(true),
                ),
            ),
            (
                "5-true;5",
                RuntimeError::TypeMismatch(
                    BinaryOperator::Minus,
                    Object::Integer(5),
                    Object::Boolean(true),
                ),
            ),
            (
                "-true;",
                RuntimeError::UnknownUnaryOperator(UnaryOperator::Neg, Object::Boolean(true)),
            ),
            (
                "if(10>1){true+false}",
                RuntimeError::UnknownBinaryOperator(
                    BinaryOperator::Plus,
                    Object::Boolean(true),
                    Object::Boolean(false),
                ),
            ),
            (
                &read_from_file("test_return_if_error.my"),
                RuntimeError::UnknownBinaryOperator(
                    BinaryOperator::Plus,
                    Object::Boolean(true),
                    Object::Boolean(false),
                ),
            ),
        ];
        check_error(&inputs);
    }

    #[test]
    fn test_return_statement() {
        let inputs = [
            ("return 10;", Object::Integer(10)),
            ("return 10;9;", Object::Integer(10)),
            ("return 10;9;", Object::Integer(10)),
            ("return 2 * 5;9;", Object::Integer(10)),
            ("9;return 2 * 5;9;", Object::Integer(10)),
            (&read_from_file("test_return_if.my"), Object::Integer(10)),
        ];
        check_input(&inputs);
    }

    #[test]
    fn test_if_else_expression() {
        let inputs = [
            ("if(true) {10}", Object::Integer(10)),
            ("if(false) {10}", Object::Null),
            ("if(1) { 10 }", Object::Integer(10)),
            ("if(1<2) { 10 }", Object::Integer(10)),
            ("if(1>2) { 10 }", Object::Null),
            ("if(1<2) { 10 } else { 20 }", Object::Integer(10)),
            ("if(1>2) { 10 } else { 20 }", Object::Integer(20)),
        ];
        check_input(&inputs);
    }

    #[test]
    fn test_eval_bool_operator() {
        let tests = [
            ("!true", Object::Boolean(false)),
            ("!false", Object::Boolean(true)),
            ("!5", Object::Boolean(false)),
            ("!!5", Object::Boolean(true)),
            ("!!true", Object::Boolean(true)),
            ("!!false", Object::Boolean(false)),
            ("1<2", Object::Boolean(true)),
            ("1>2", Object::Boolean(false)),
            ("1<1", Object::Boolean(false)),
            ("1>1", Object::Boolean(false)),
            ("1==1", Object::Boolean(true)),
            ("1!=1", Object::Boolean(false)),
            ("1==2", Object::Boolean(false)),
            ("1!=2", Object::Boolean(true)),
            ("true == true", Object::Boolean(true)),
            ("false == false", Object::Boolean(true)),
            ("true == false", Object::Boolean(false)),
            ("true != false", Object::Boolean(true)),
            ("(1<2) == true", Object::Boolean(true)),
            ("(1<2)==false", Object::Boolean(false)),
            ("(1>2) ==true", Object::Boolean(false)),
            ("(1>2) == false", Object::Boolean(true)),
        ];
        check_input(&tests)
    }

    #[test]
    fn test_eval_integer_expression() {
        let inputs = [
            ("5", Object::Integer(5)),
            ("10", Object::Integer(10)),
            ("-5", Object::Integer(-5)),
            ("-10", Object::Integer(-10)),
            ("5+5+5+5-10", Object::Integer(10)),
            ("2*2*2*2*2", Object::Integer(32)),
            ("-50+100+-50", Object::Integer(0)),
            ("5*2+10", Object::Integer(20)),
            ("5+2*10", Object::Integer(25)),
            ("20+2*-10", Object::Integer(0)),
            ("50/2*2 +10", Object::Integer(60)),
            ("2*(5+10)", Object::Integer(30)),
            ("3*3*3 +10", Object::Integer(37)),
            ("3*(3*3) +10", Object::Integer(37)),
            ("(5+10*2+15/3)*2+-10", Object::Integer(50)),
        ];
        check_input(&inputs);
    }
    /*辅助函数*/
    fn check_input(inputs: &[(&str, Object)]) {
        for (i, (input, expected)) in inputs.iter().enumerate() {
            let env = Rc::new(RefCell::new(Environment::new()));
            let evaluated = eval(input, Rc::clone(&env));
            match evaluated {
                Ok(evaluated) => {
                    test_object(&evaluated, expected, i);
                }
                Err(err) => {
                    // eprintln!("eval error: {} ,input: {}", err, &input);
                    panic!("eval error: {} ,input: {}", err, &input);
                }
            }
        }
    }

    fn check_error(inputs: &[(&str, RuntimeError)]) {
        for (i, (input, expected)) in inputs.iter().enumerate() {
            let env = Rc::new(RefCell::new(Environment::new()));
            let evaluated = eval(input, Rc::clone(&env));
            match evaluated {
                Err(err) => {
                    test_object(&err, expected, i);
                }
                Ok(evaluated) => {
                    eprintln!("unexpected: input: {}, evaluated: {}", input, evaluated)
                }
            }
        }
    }

    fn eval(input: &str, env: Env) -> EvalResult<Object> {
        let mut parser = Parser::from(input);
        let program = parser.parse_program();
        println!("parsed program:\n{}", program);
        evaluator::eval(&program, Rc::clone(&env))
    }

    fn test_object<T: PartialEq + Debug + Display>(object: &T, expected: &T, i: usize) {
        assert_eq!(object, expected, "Error at index[{}]", i)
    }

    fn read_from_file(filename: &str) -> String {
        let path = format!(r"/Users/l/Code/RustroverProjects/emmmm/res/{}", filename);
        std::fs::read_to_string(path).unwrap()
    }
}
