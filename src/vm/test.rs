#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::compiler::Compiler;
    use crate::parser::base::ast::Program;
    use crate::object::{HashKey, Object, RuntimeError};
    use crate::vm::Vm;
    use std::time::Instant;

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
    fn recursive_fibonacci() {
        let inputs = vec![(
            r"
            let fibonacci = fn(n) {
                if n < 2 {
                    n
                } else {
                    fibonacci(n - 1) + fibonacci(n - 2)
                }
            }
            fibonacci(26)
            ",
            Object::Integer(121393),
        )];
        run_vm_test(inputs);
    }
    #[test]
    fn recursive_function() {
        let inputs = vec![
            (
                r"
            let countDown = fn(x) {
                if x == 0 {
                    0
                } else {
                    countDown(x - 1)
                }
            }
            let wrapper = fn() {
                countDown(1)
            }
            wrapper()
        ",
                Object::Integer(0),
            ),
            (
                r"
                let wrapper = fn() {
                    let countDown = fn(x) {
                        if x == 0 {
                            0
                        } else {
                            countDown(x - 1)
                        }
                    }
                    countDown(1)
                }
                wrapper()
            ",
                Object::Integer(0),
            ),
        ];

        run_vm_test(inputs);
    }
    #[test]
    fn closures() {
        let tests = vec![
            (
                r"
               let newAdder = fn(a) {
                 let adder = fn(b) { a + b};
                 return adder;
               };
               let addTwo = newAdder(2);
               addTwo(3);
        ",
                Object::Integer(5),
            ),
            (
                r"
            let newClosure = fn(a) {
                fn() { a }
            }
            let closure = newClosure(99)
            closure()
            ",
                Object::Integer(99),
            ),
            (
                r"
                let newAdderOuter = fn(a, b) {
                    let c = a + b
                    fn(d) {
                        let e = d + c
                        fn(f) {
                            e + f
                        }
                    }
                }
                let newAddInner = newAdderOuter(1, 2)
                let adder = newAddInner(3)
                adder(8)
            ",
                Object::Integer(14),
            ),
            (
                r"
                let a = 1
                let newAdderOuter = fn(b) {
                    fn(c) {
                        fn(d) {
                            a + b + c + d
                        }
                    }
                }
                let newAdderInner = newAdderOuter(2)
                let adder = newAdderInner(3)
                adder(8)
                ",
                Object::Integer(14),
            ),
        ];
        run_vm_test(tests);
    }
    #[test]
    fn builtin_function() {
        let tests = vec![
            ("len([1,2,3])", Object::Integer(3)),
            (r#"len("hello")"#, Object::Integer(5)),
        ];
        run_vm_test(tests);
    }
    #[test]
    fn call_with_wrong_arguments() {
        let tests = vec![
            ("fn() {1;}(1)", RuntimeError::WrongArgumentCount(0, 1)),
            ("fn(a) {a;}()", RuntimeError::WrongArgumentCount(1, 0)),
            ("fn(a, b) {a;}(2)", RuntimeError::WrongArgumentCount(2, 1)),
        ];
        run_vm_test_error(tests);
    }

    #[test]
    fn test_call_with_args_and_bindings() {
        let tests = vec![(
            r"
            let global_num = 10;
            let sum = fn(a, b) {
                let c = a + b
                c + global_num
            }
            sum(12,18)
            ",
            Object::Integer(40),
        )];
        run_vm_test(tests);
    }

    #[test]
    fn test_call_with_binding() {
        let tests = vec![
            (
                r"
            let sum = fn(a, b) {
                let c = a + b;
                c;
            };
            sum(1, 2);
            ",
                Object::Integer(3),
            ),
            (
                r"
            let sum = fn(a, b) {
            let c = a + b;
            c;
            };
            sum(1, 2) + sum(3, 4);
            ",
                Object::Integer(10),
            ),
            (
                r"
            let sum = fn(a, b) {
            let c = a + b;
            c;
            };
            let outer = fn() {
                sum(1, 2) + sum(3, 4);
            };
            outer();
            ",
                Object::Integer(10),
            ),
            (
                r"
            let one = fn(a) { a }
            one(233)
            ",
                Object::Integer(233),
            ),
            (
                r"
            let add = fn(a, b) { a + b }
            add(2, 3)
            ",
                Object::Integer(5),
            ),
            (
                r"
            let one = fn() { let a = 1; a }
            one()
            ",
                Object::Integer(1),
            ),
            (
                r"
            let oneAndTwo = fn() {
                let one = 1;
                let two = 2;
                return one + two;
            }
            oneAndTwo()
            ",
                Object::Integer(3),
            ),
            (
                r"
            let oneAndTwo = fn() { let one = 1; let two = 2; one + two }
            let threeAndFour = fn() { let three = 3; let four = 4; three + four }
            oneAndTwo() + threeAndFour()
            ",
                Object::Integer(10),
            ),
            (
                r"
            let firstFoobar = fn() { let foobar = 50; foobar; };
            let secondFoobar = fn() { let foobar = 100; foobar; };
            firstFoobar() + secondFoobar();
            ",
                Object::Integer(150),
            ),
            (
                r"
            let globalSeed = 50;
            let minusOne = fn() {
                let num = 1;
                globalSeed - num;
            }
            let minusTwo = fn() {
                let num = 2;
                globalSeed - num;
            }
            minusOne() + minusTwo();
            ",
                Object::Integer(97),
            ),
        ];
        run_vm_test(tests);
    }

    #[test]
    fn test_call() {
        let tests = vec![
            (
                r"
            let one_plus_two = fn() { 1 + 5 };
            one_plus_two();
            ",
                Object::Integer(6),
            ),
            (
                r"
            let null = fn() {  }
            null()
            ",
                Object::Null,
            ),
            (
                r"
            let one = fn() { 1 }
            let two = fn() { return 2; 3 }
            one() + two()
            ",
                Object::Integer(3),
            ),
        ];
        run_vm_test(tests);
    }

    #[test]
    fn test_assign() {
        let tests = vec![
            ("let a = 1; a = 2; a", Object::Integer(2)),
            ("let arr = [1,2,3]; arr[2] = 0;arr[2]", Object::Integer(0)),
            (
                r#"let map = {1+1:2+2,"hello":5*3, 10:"yo"}; map[2]="new_data"; map[2]"#,
                Object::String("new_data".to_string()),
            ),
        ];
        run_vm_test(tests);
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
            let program = Program::_new(input);
            let mut compiler = Compiler::new();
            match compiler.compile(&program) {
                Ok(byte_code) => {
                    let mut vm = Vm::new(byte_code.clone());
                    let start = Instant::now();
                    match vm.run() {
                        Ok(object) => {
                            let duration = start.elapsed();
                            println!(
                                "{} s {} ms, result: {}",
                                duration.as_secs(),
                                duration.as_millis(),
                                object
                            );
                            test_expected_object(input, &object, &expected);
                        }
                        Err(e) => {
                            let cs = &byte_code.constants;
                            for x in cs.iter() {
                                println!("-----");
                                println!("{}", x);
                            }
                            panic!("Input: {}\nRuntime Error: {:?}", input, e)
                        },
                    };
                }
                Err(e) => panic!("Input: {}\nCompile Error: {:?}", input, e),
            }
        }
    }

    fn run_vm_test_error(tests: Vec<(&str, RuntimeError)>) {
        for (input, expected) in tests {
            let program = Program::_new(input);
            let mut compiler = Compiler::new();
            match compiler.compile(&program) {
                Ok(byte_code) => {
                    let mut vm = Vm::new(byte_code);
                    if let Err(err) = vm.run() {
                        assert_eq!(err, expected, "error input:\n{}", input);
                    }
                }
                Err(e) => panic!("Input: {}\nCompiler Error: {:?}", input, e),
            }
        }
    }

    fn test_expected_object(input: &str, top: &Object, expected: &Object) {
        assert_eq!(top, expected, "input: {}", input);
    }
}
