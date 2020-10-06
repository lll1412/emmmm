#[cfg(test)]
mod tests {
    use crate::compiler::Compiler;
    use crate::core::base::ast::Program;
    use crate::object::{HashKey, Object};
    use crate::vm::Vm;
    use std::cell::RefCell;
    use std::collections::HashMap;
    macro_rules! hash {
        {} => {
            HashMap::new()
        }
        ;
        {$($k:expr => $v:expr),+,} => {
            {
            let mut map = HashMap::new();
            $(
                map.insert($k, $v);
            )+
            map
            }
        };
    }
    #[test]
    fn test_index() {
        let tests = vec![
            ("[1,2,3][1]", Object::Integer(2)),
            ("[1,2+12,3][10]", Object::Null),
            ("{}[10]", Object::Null),
            (
                r#"{"a":1,2:"b", 1+2: "3"+ "c"}[3]"#,
                Object::String("3c".to_string()),
            ),
        ];
        run_vm_test(tests);
    }

    #[test]
    fn test_hash_literal() {
        let tests = vec![
            ("{}", Object::Hash(RefCell::new(hash! {}))),
            (
                r#"{"a":1, "b":2, "c":3}"#,
                Object::Hash(RefCell::new(hash! {
                HashKey::String("a".to_string()) => Object::Integer(1),
                HashKey::String("b".to_string()) => Object::Integer(2),
                HashKey::String("c".to_string()) => Object::Integer(3),
                })),
            ),
            (
                r#"{1+1:2+2,"hello":5*3, 10:"yo"}"#,
                Object::Hash(RefCell::new(hash! {
                HashKey::Integer(2) => Object::Integer(4),
                HashKey::String("hello".to_string()) => Object::Integer(15),
                HashKey::Integer(10) => Object::String("yo".to_string()),
                })),
            ),
        ];
        run_vm_test(tests);
    }

    #[test]
    fn test_array_literal() {
        let tests = vec![
            ("[]", Object::Array(RefCell::new(vec![]))),
            (
                "[1,2,3]",
                Object::Array(RefCell::new(vec![
                    Object::Integer(1),
                    Object::Integer(2),
                    Object::Integer(3),
                ])),
            ),
            (
                "[1+2, 2*3, 3-1]",
                Object::Array(RefCell::new(vec![
                    Object::Integer(3),
                    Object::Integer(6),
                    Object::Integer(2),
                ])),
            ),
        ];
        run_vm_test(tests);
    }

    #[test]
    fn test_string_literal() {
        let tests = vec![
            (
                r#""hello world""#,
                Object::String("hello world".to_string()),
            ),
            (
                r#"let s = "hello world"; s"#,
                Object::String("hello world".to_string()),
            ),
            (
                r#""hello" + " world""#,
                Object::String("hello world".to_string()),
            ),
        ];
        run_vm_test(tests);
    }

    #[test]
    fn test_let_statement() {
        let tests = vec![
            ("let a= 1;a", Object::Integer(1)),
            ("let a= 1;let b = 2;a+b", Object::Integer(3)),
            ("let a= 1;let b = a + a;a+b", Object::Integer(3)),
        ];
        run_vm_test(tests);
    }

    #[test]
    fn test_bang_expression() {
        let tests = vec![("! if false { 1 };", Object::Null)];
        run_vm_test(tests);
    }

    #[test]
    fn test_condition_expression() {
        let tests = vec![
            ("if false { 10 }", Object::Null),
            ("if 1>2 { 10 }", Object::Null),
            ("if true { 10 }", Object::Integer(10)),
            ("if true { 10 } else { 20 }", Object::Integer(10)),
            ("if 1 { 10 }", Object::Integer(10)),
            ("if 1<2 { 10 }", Object::Integer(10)),
            ("if 1>2 { 10 } else { 20 }", Object::Integer(20)),
        ];
        run_vm_test(tests);
    }

    #[test]
    fn test_boolean_expression() {
        let tests = vec![
            ("!!true", Object::Boolean(true)),
            ("!!false", Object::Boolean(false)),
            ("1<2", Object::Boolean(true)),
            ("1>2", Object::Boolean(false)),
            ("1>1", Object::Boolean(false)),
            ("1>1", Object::Boolean(false)),
            ("1==1", Object::Boolean(true)),
            ("1!=1", Object::Boolean(false)),
            ("1==2", Object::Boolean(false)),
            ("1!=2", Object::Boolean(true)),
            ("true==true", Object::Boolean(true)),
            ("false==false", Object::Boolean(true)),
            ("true==false", Object::Boolean(false)),
            ("true!=false", Object::Boolean(true)),
            ("(1 < 2) == true", Object::Boolean(true)),
            ("(1 < 2) == false", Object::Boolean(false)),
            ("(1 > 2) == true", Object::Boolean(false)),
            ("(1 > 2) == false", Object::Boolean(true)),
            ("!true", Object::Boolean(false)),
            ("!false", Object::Boolean(true)),
            ("!(1>2)", Object::Boolean(true)),
            ("-1", Object::Integer(-1)),
            ("-(2+3)", Object::Integer(-5)),
            ("-(-1)", Object::Integer(1)),
        ];
        run_vm_test(tests);
    }

    #[test]
    fn test_integer_arithmetic() {
        let tests = vec![
            ("1", Object::Integer(1)),
            ("2", Object::Integer(2)),
            ("1+2", Object::Integer(3)),
            ("1-2", Object::Integer(-1)),
            ("2*3+2", Object::Integer(8)),
            ("15/2 +3", Object::Integer(10)),
            ("15/(2 +3)", Object::Integer(3)),
        ];
        run_vm_test(tests)
    }

    fn run_vm_test(tests: Vec<(&str, Object)>) {
        for (input, expected) in tests {
            let program = Program::new(input);
            let mut compiler = Compiler::_new();
            match compiler.compile(program) {
                Ok(byte_code) => {
                    let mut vm = Vm::new(byte_code);
                    match vm.run() {
                        Ok(object) => {
                            test_expected_object(input, &object, &expected);
                        }
                        Err(e) => panic!("Input: {}\nVm Error: {:?}", input, e),
                    };
                }
                Err(e) => panic!("Input: {}\nCompiler Error: {:?}", input, e),
            }
        }
    }

    fn test_expected_object(input: &str, top: &Object, expected: &Object) {
        assert_eq!(top, expected, "input: {}", input);
    }
}
