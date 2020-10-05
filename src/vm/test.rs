#[cfg(test)]
mod tests {
    use crate::compiler::Compiler;
    use crate::core::base::ast::Program;
    use crate::object::Object;
    use crate::vm::Vm;
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
            let mut compiler = Compiler::new();
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
