#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::compiler::{Compiler, Instructions};
    use crate::compiler::code::{
        self, _make, _make_closure, _make_const, _make_noop, _print_instructions, Constant, make,
        Opcode,
    };
    use crate::compiler::symbol_table::{Symbol, SymbolScope, SymbolTable};
    use crate::core::base::ast::Program;
    use crate::create_rc_ref_cell;

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
    fn recursive_function() {
        let count_down_const = Constant::CompiledFunction(
            vec![
                _make_noop(Opcode::CurrentClosure),
                _make(Opcode::GetLocal, 0),
                _make_const(0),
                _make_noop(Opcode::Sub),
                _make(Opcode::Call, 1),
                _make_noop(Opcode::ReturnValue),
            ]
                .concat(),
            1,
            1,
        );
        let inputs = vec![
            (
                r"
            let countDown = fn(x) {
                countDown(x - 1)
            }
            countDown(2)
            ",
                vec![
                    Constant::Integer(1),
                    count_down_const.clone(),
                    Constant::Integer(2),
                ],
                vec![
                    _make_closure(1, 0),
                    _make(Opcode::SetGlobal, 0),
                    _make(Opcode::GetGlobal, 0),
                    _make_const(2),
                    _make(Opcode::Call, 1),
                    _make_noop(Opcode::Pop),
                ],
            ),
            (
                r"
                let wrapper = fn() {
                    let countDown = fn(x) {
                        countDown(x - 1)
                    }
                    countDown(2)
                }
                wrapper()
                          ",
                vec![
                    Constant::Integer(1),
                    count_down_const,
                    Constant::Integer(2),
                    Constant::CompiledFunction(
                        vec![
                            _make_closure(1, 0),
                            _make(Opcode::SetLocal, 0),
                            _make(Opcode::GetLocal, 0),
                            _make_const(2),
                            _make(Opcode::Call, 1),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        1,
                        0,
                    ),
                ],
                vec![
                    _make_closure(3, 0),
                    _make(Opcode::SetGlobal, 0),
                    _make(Opcode::GetGlobal, 0),
                    _make(Opcode::Call, 0),
                    _make_noop(Opcode::Pop),
                ],
            ),
        ];

        run_compile_test(inputs);
    }
    #[test]
    fn closures() {
        let tests = vec![
            (
                r"
            fn(a) {
                fn(b) {
                    a + b
                }
            }
            ",
                vec![
                    Constant::CompiledFunction(
                        vec![
                            _make(Opcode::GetFree, 0),
                            _make(Opcode::GetLocal, 0),
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        1,
                        1,
                    ),
                    Constant::CompiledFunction(
                        vec![
                            _make(Opcode::GetLocal, 0),
                            _make_closure(0, 1),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        1,
                        1,
                    ),
                ],
                vec![_make_closure(1, 0), _make_noop(Opcode::Pop)],
            ),
            (
                r"
            fn(a) {
                fn(b) {
                    fn(c) {
                        a + b + c
                    }
                }
            }
            ",
                vec![
                    Constant::CompiledFunction(
                        vec![
                            _make(Opcode::GetFree, 0),
                            _make(Opcode::GetFree, 1),
                            _make_noop(Opcode::Add),
                            _make(Opcode::GetLocal, 0),
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        1,
                        1,
                    ),
                    Constant::CompiledFunction(
                        vec![
                            _make(Opcode::GetFree, 0),
                            _make(Opcode::GetLocal, 0),
                            _make_closure(0, 2),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        1,
                        1,
                    ),
                    Constant::CompiledFunction(
                        vec![
                            _make(Opcode::GetLocal, 0),
                            _make_closure(1, 1),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        1,
                        1,
                    ),
                ],
                vec![_make_closure(2, 0), _make_noop(Opcode::Pop)],
            ),
            (
                r"
            let global = 55
            fn() {
                let a = 66
                fn() {
                    let b = 77
                    fn() {
                        let c = 88
                        global + a + b + c
                    }
                }
            }
            ",
                vec![
                    Constant::Integer(55),
                    Constant::Integer(66),
                    Constant::Integer(77),
                    Constant::Integer(88),
                    Constant::CompiledFunction(
                        vec![
                            _make_const(3),
                            _make(Opcode::SetLocal, 0),  // declare c
                            _make(Opcode::GetGlobal, 0), // global
                            _make(Opcode::GetFree, 0),   // free a
                            _make_noop(Opcode::Add),
                            _make(Opcode::GetFree, 1), // free b
                            _make_noop(Opcode::Add),
                            _make(Opcode::GetLocal, 0), // local c
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        1,
                        0,
                    ),
                    Constant::CompiledFunction(
                        vec![
                            _make_const(2),
                            _make(Opcode::SetLocal, 0), // declare b
                            _make(Opcode::GetFree, 0),  // free a
                            _make(Opcode::GetLocal, 0),
                            _make_closure(4, 2), // closure
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        1,
                        0,
                    ),
                    Constant::CompiledFunction(
                        vec![
                            _make_const(1),
                            _make(Opcode::SetLocal, 0), // declare a
                            _make(Opcode::GetLocal, 0),
                            _make_closure(5, 1),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        1,
                        0,
                    ),
                ],
                vec![
                    _make_const(0),
                    _make(Opcode::SetGlobal, 0),
                    _make_closure(6, 0),
                    _make_noop(Opcode::Pop),
                ],
            ),
        ];
        run_compile_test(tests);
    }
    #[test]
    fn builtins() {
        let tests = vec![(
            r"
            len([]);
            push([], 1);
            ",
            vec![Constant::Integer(1)],
            vec![
                _make(Opcode::GetBuiltin, 0),
                _make(Opcode::Array, 0),
                _make(Opcode::Call, 1),
                _make_noop(Opcode::Pop),
                _make(Opcode::GetBuiltin, 4),
                _make(Opcode::Array, 0),
                _make_const(0),
                _make(Opcode::Call, 2),
                _make_noop(Opcode::Pop),
            ],
        )];
        run_compile_test(tests);
    }
    #[test]
    fn test_let_statement_scope() {
        let tests = vec![
            (
                r"
            let num = 55; 
            fn() { num }",
                vec![
                    Constant::Integer(55),
                    Constant::CompiledFunction(
                        vec![_make(Opcode::GetGlobal, 0), _make_noop(Opcode::ReturnValue)].concat(),
                        0,
                        0,
                    ),
                ],
                vec![
                    _make_const(0),
                    _make(Opcode::SetGlobal, 0),
                    _make_closure(1, 0),
                    _make_noop(Opcode::Pop),
                ],
            ),
            (
                r"
            fn() {
                let num = 55;
                num
            }
            ",
                vec![
                    Constant::Integer(55),
                    Constant::CompiledFunction(
                        vec![
                            _make_const(0),
                            _make(Opcode::SetLocal, 0),
                            _make(Opcode::GetLocal, 0),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        1,
                        0,
                    ),
                ],
                vec![_make_closure(1, 0), _make_noop(Opcode::Pop)],
            ),
            (
                r"
            fn() {
                let a = 1;
                let b = 2;
                return a + b
            }
            ",
                vec![
                    Constant::Integer(1),
                    Constant::Integer(2),
                    Constant::CompiledFunction(
                        vec![
                            _make_const(0),
                            _make(Opcode::SetLocal, 0),
                            _make_const(1),
                            _make(Opcode::SetLocal, 1),
                            _make(Opcode::GetLocal, 0),
                            _make(Opcode::GetLocal, 1),
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        2,
                        0,
                    ),
                ],
                vec![_make_closure(2, 0), _make_noop(Opcode::Pop)],
            ),
        ];
        run_compile_test(tests);
    }

    #[test]
    fn test_function_call_with_args() {
        let tests = vec![
            (
                r"
            let global_num = 10;
            let sum = fn(a, b) {
                let c = a + b
                c + global_num
            }
            let outer = fn() {
                sum(1, 2) + sum(3, 4) + global_num
            }
            outer() + global_num
                ",
                vec![
                    Constant::Integer(10),
                    Constant::CompiledFunction(
                        vec![
                            _make(Opcode::GetLocal, 0),
                            _make(Opcode::GetLocal, 1),
                            _make_noop(Opcode::Add),
                            _make(Opcode::SetLocal, 2),
                            _make(Opcode::GetLocal, 2),
                            _make(Opcode::GetGlobal, 0),
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        3,
                        2,
                    ),
                    Constant::Integer(1),
                    Constant::Integer(2),
                    Constant::Integer(3),
                    Constant::Integer(4),
                    Constant::CompiledFunction(
                        vec![
                            _make(Opcode::GetGlobal, 1),
                            _make_const(2),
                            _make_const(3),
                            _make(Opcode::Call, 2),
                            _make(Opcode::GetGlobal, 1),
                            _make_const(4),
                            _make_const(5),
                            _make(Opcode::Call, 2),
                            _make_noop(Opcode::Add),
                            _make(Opcode::GetGlobal, 0),
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        0,
                        0,
                    ),
                ],
                vec![
                    _make_const(0),
                    _make(Opcode::SetGlobal, 0),
                    _make_closure(1, 0),
                    _make(Opcode::SetGlobal, 1),
                    _make_closure(6, 0),
                    _make(Opcode::SetGlobal, 2),
                    _make(Opcode::GetGlobal, 2),
                    _make(Opcode::Call, 0),
                    _make(Opcode::GetGlobal, 0),
                    _make_noop(Opcode::Add),
                    _make_noop(Opcode::Pop),
                ],
            ),
            (
                r"
                let g = 123;
                let d = 234;
                let one_arg = fn(a) { a };
                one_arg(233);
                ",
                vec![
                    Constant::Integer(123),
                    Constant::Integer(234),
                    Constant::CompiledFunction(
                        vec![_make(Opcode::GetLocal, 0), _make_noop(Opcode::ReturnValue)].concat(),
                        1,
                        1,
                    ),
                    Constant::Integer(233),
                ],
                vec![
                    _make_const(0),
                    _make(Opcode::SetGlobal, 0),
                    _make_const(1),
                    _make(Opcode::SetGlobal, 1),
                    _make_closure(2, 0),
                    _make(Opcode::SetGlobal, 2),
                    _make(Opcode::GetGlobal, 2),
                    _make_const(3),
                    _make(Opcode::Call, 1),
                    _make_noop(Opcode::Pop),
                ],
            ),
            (
                r"
                let g = 111;
                let many_arg = fn(a, b, c) { a + b + c + g };
                many_arg(2, 3, 4);
                ",
                vec![
                    Constant::Integer(111),
                    Constant::CompiledFunction(
                        vec![
                            _make(Opcode::GetLocal, 0),
                            _make(Opcode::GetLocal, 1),
                            _make_noop(Opcode::Add),
                            _make(Opcode::GetLocal, 2),
                            _make_noop(Opcode::Add),
                            _make(Opcode::GetGlobal, 0),
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        3,
                        3,
                    ),
                    Constant::Integer(2),
                    Constant::Integer(3),
                    Constant::Integer(4),
                ],
                vec![
                    _make_const(0),
                    _make(Opcode::SetGlobal, 0),
                    _make_closure(1, 0),
                    _make(Opcode::SetGlobal, 1),
                    _make(Opcode::GetGlobal, 1),
                    _make_const(2),
                    _make_const(3),
                    _make_const(4),
                    _make(Opcode::Call, 3),
                    _make_noop(Opcode::Pop),
                ],
            ),
        ];
        run_compile_test(tests);
    }

    #[test]
    fn test_function_call() {
        let tests = vec![
            (
                r"
                fn() { 1 + 12 }();
                ",
                vec![
                    Constant::Integer(1),
                    Constant::Integer(12),
                    Constant::CompiledFunction(
                        vec![
                            _make_const(0),
                            _make_const(1),
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        0,
                        0,
                    ),
                ],
                vec![
                    _make_closure(2, 0),
                    _make(Opcode::Call, 0),
                    _make_noop(Opcode::Pop),
                ],
            ),
            (
                r"
                let one_arg = fn(a) {  };
                one_arg(233);
                ",
                vec![
                    Constant::CompiledFunction(vec![_make_noop(Opcode::Return)].concat(), 1, 1),
                    Constant::Integer(233),
                ],
                vec![
                    _make_closure(0, 0),
                    _make(Opcode::SetGlobal, 0),
                    _make(Opcode::GetGlobal, 0),
                    _make_const(1),
                    _make(Opcode::Call, 1),
                    _make_noop(Opcode::Pop),
                ],
            ),
            (
                r#"
                    let m = fn() { 1 + 12 }
                    m()
                "#,
                vec![
                    Constant::Integer(1),
                    Constant::Integer(12),
                    Constant::CompiledFunction(
                        vec![
                            _make_const(0),
                            _make_const(1),
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        0,
                        0,
                    ),
                ],
                vec![
                    _make_closure(2, 0),
                    make(Opcode::SetGlobal, vec![0]),
                    make(Opcode::GetGlobal, vec![0]),
                    _make(Opcode::Call, 0),
                    _make_noop(Opcode::Pop),
                ],
            ),
            (
                r#"
                    let one = fn() { 1 }
                    let two = fn() { 2 }
                    one() + two()
                "#,
                vec![
                    Constant::Integer(1),
                    Constant::CompiledFunction(
                        vec![_make_const(0), _make_noop(Opcode::ReturnValue)].concat(),
                        0,
                        0,
                    ),
                    Constant::Integer(2),
                    Constant::CompiledFunction(
                        vec![_make_const(2), _make_noop(Opcode::ReturnValue)].concat(),
                        0,
                        0,
                    ),
                ],
                vec![
                    // let one = fn1
                    _make_closure(1, 0),
                    make(Opcode::SetGlobal, vec![0]),
                    // let two = fn2
                    _make_closure(3, 0),
                    make(Opcode::SetGlobal, vec![1]),
                    // one()
                    make(Opcode::GetGlobal, vec![0]),
                    _make(Opcode::Call, 0),
                    // two()
                    make(Opcode::GetGlobal, vec![1]),
                    _make(Opcode::Call, 0),
                    // one() + two()
                    _make_noop(Opcode::Add),
                    _make_noop(Opcode::Pop),
                ],
            ),
        ];
        run_compile_test(tests);
    }

    #[test]
    fn test_functions() {
        let tests = vec![
            (
                r#"fn() { return 5 + 10 }"#,
                vec![
                    Constant::Integer(5),
                    Constant::Integer(10),
                    Constant::CompiledFunction(
                        vec![
                            _make_const(0),
                            _make_const(1),
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        0,
                        0,
                    ),
                ],
                vec![_make_closure(2, 0), _make_noop(Opcode::Pop)],
            ),
            (
                r#"fn() { 5 + 10 }"#,
                vec![
                    Constant::Integer(5),
                    Constant::Integer(10),
                    Constant::CompiledFunction(
                        vec![
                            _make_const(0),
                            _make_const(1),
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        0,
                        0,
                    ),
                ],
                vec![_make_closure(2, 0), _make_noop(Opcode::Pop)],
            ),
            (
                r#"fn() { 1; 2}"#,
                vec![
                    Constant::Integer(1),
                    Constant::Integer(2),
                    Constant::CompiledFunction(
                        vec![
                            _make_const(0),
                            _make_noop(Opcode::Pop),
                            _make_const(1),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        0,
                        0,
                    ),
                ],
                vec![_make_closure(2, 0), _make_noop(Opcode::Pop)],
            ),
        ];
        run_compile_test(tests);
    }

    #[test]
    fn test_assign() {
        let tests = vec![
            (
                "let a = 1; a = 2; a",
                vec![Constant::Integer(1), Constant::Integer(2)],
                vec![
                    make(Opcode::Constant, vec![0]),
                    make(Opcode::SetGlobal, vec![0]),
                    make(Opcode::Constant, vec![1]),
                    make(Opcode::Assign, vec![0]),
                    make(Opcode::GetGlobal, vec![0]),
                    make(Opcode::Pop, vec![]),
                ],
            ),
            (
                "let arr = [1,2,3]; arr[2] = 0;arr",
                vec![
                    Constant::Integer(1),
                    Constant::Integer(2),
                    Constant::Integer(3),
                    Constant::Integer(2),
                    Constant::Integer(0),
                    // Constant::Integer(2),
                ],
                vec![
                    make(Opcode::Constant, vec![0]),
                    make(Opcode::Constant, vec![1]),
                    make(Opcode::Constant, vec![2]),
                    make(Opcode::Array, vec![3]),     //声明赋值数组
                    make(Opcode::SetGlobal, vec![0]), //存arr
                    make(Opcode::Constant, vec![3]),  //index
                    make(Opcode::Constant, vec![4]),  //value
                    make(Opcode::Assign, vec![0]),    //arr[index] = value
                    make(Opcode::GetGlobal, vec![0]), //取arr
                    // make(Opcode::Constant, vec![5]),  //index
                    // make(Opcode::Index, vec![]),      //arr[index]
                    make(Opcode::Pop, vec![]),
                ],
            ),
            (
                r#"let map = {1+1:2+2,"hello":5*3, 10:"yo"}; map[2]="new_data"; map"#,
                vec![
                    Constant::Integer(1),
                    Constant::Integer(1),
                    Constant::Integer(2),
                    Constant::Integer(2),
                    Constant::String("hello".to_string()),
                    Constant::Integer(5),
                    Constant::Integer(3),
                    Constant::Integer(10),
                    Constant::String("yo".to_string()),
                    Constant::Integer(2),
                    Constant::String("new_data".to_string()),
                ],
                vec![
                    make(Opcode::Constant, vec![0]),
                    make(Opcode::Constant, vec![1]),
                    make(Opcode::Add, vec![]),
                    make(Opcode::Constant, vec![2]),
                    make(Opcode::Constant, vec![3]),
                    make(Opcode::Add, vec![]),
                    make(Opcode::Constant, vec![4]),
                    make(Opcode::Constant, vec![5]),
                    make(Opcode::Constant, vec![6]),
                    make(Opcode::Mul, vec![]),
                    make(Opcode::Constant, vec![7]),
                    make(Opcode::Constant, vec![8]),
                    make(Opcode::Hash, vec![3]),
                    make(Opcode::SetGlobal, vec![0]), //声明初始化Map
                    make(Opcode::Constant, vec![9]),  //index
                    make(Opcode::Constant, vec![10]), //value
                    make(Opcode::Assign, vec![0]),    //map[index] = value
                    make(Opcode::GetGlobal, vec![0]), //取map
                    make(Opcode::Pop, vec![]),
                ],
            ),
        ];
        run_compile_test(tests);
    }

    #[test]
    fn test_index() {
        let tests = vec![
            (
                "[1,2,3][1]",
                vec![
                    Constant::Integer(1),
                    Constant::Integer(2),
                    Constant::Integer(3),
                    Constant::Integer(1),
                ],
                vec![
                    make(Opcode::Constant, vec![0]),
                    make(Opcode::Constant, vec![1]),
                    make(Opcode::Constant, vec![2]),
                    make(Opcode::Array, vec![3]),
                    make(Opcode::Constant, vec![3]),
                    make(Opcode::Index, vec![]),
                    make(Opcode::Pop, vec![]),
                ],
            ),
            (
                r#"{"a":1,2:"b", 1+2: "3"+ "c"}[3]"#,
                vec![
                    Constant::String("a".to_string()),
                    Constant::Integer(1),
                    Constant::Integer(2),
                    Constant::String("b".to_string()),
                    Constant::Integer(1),
                    Constant::Integer(2),
                    Constant::String("3".to_string()),
                    Constant::String("c".to_string()),
                    Constant::Integer(3),
                ],
                vec![
                    make(Opcode::Constant, vec![0]),
                    make(Opcode::Constant, vec![1]),
                    make(Opcode::Constant, vec![2]),
                    make(Opcode::Constant, vec![3]),
                    make(Opcode::Constant, vec![4]),
                    make(Opcode::Constant, vec![5]),
                    make(Opcode::Add, vec![]),
                    make(Opcode::Constant, vec![6]),
                    make(Opcode::Constant, vec![7]),
                    make(Opcode::Add, vec![]),
                    make(Opcode::Hash, vec![3]),
                    make(Opcode::Constant, vec![8]),
                    make(Opcode::Index, vec![]),
                    make(Opcode::Pop, vec![]),
                ],
            ),
        ];
        run_compile_test(tests);
    }

    #[test]
    fn test_hash_literal() {
        let tests = vec![
            (
                "{}",
                vec![],
                vec![make(Opcode::Hash, vec![0]), make(Opcode::Pop, vec![])],
            ),
            (
                r#"{"a":1, "b":2, "c":3}"#,
                vec![
                    Constant::String("a".to_string()),
                    Constant::Integer(1),
                    Constant::String("b".to_string()),
                    Constant::Integer(2),
                    Constant::String("c".to_string()),
                    Constant::Integer(3),
                ],
                vec![
                    make(Opcode::Constant, vec![0]),
                    make(Opcode::Constant, vec![1]),
                    make(Opcode::Constant, vec![2]),
                    make(Opcode::Constant, vec![3]),
                    make(Opcode::Constant, vec![4]),
                    make(Opcode::Constant, vec![5]),
                    make(Opcode::Hash, vec![3]),
                    make(Opcode::Pop, vec![]),
                ],
            ),
            (
                r#"{1+1:2+2,"hello":5*3, 10:"yo"}"#,
                vec![
                    Constant::Integer(1),
                    Constant::Integer(1),
                    Constant::Integer(2),
                    Constant::Integer(2),
                    Constant::String("hello".to_string()),
                    Constant::Integer(5),
                    Constant::Integer(3),
                    Constant::Integer(10),
                    Constant::String("yo".to_string()),
                ],
                vec![
                    make(Opcode::Constant, vec![0]),
                    make(Opcode::Constant, vec![1]),
                    make(Opcode::Add, vec![]),
                    make(Opcode::Constant, vec![2]),
                    make(Opcode::Constant, vec![3]),
                    make(Opcode::Add, vec![]),
                    make(Opcode::Constant, vec![4]),
                    make(Opcode::Constant, vec![5]),
                    make(Opcode::Constant, vec![6]),
                    make(Opcode::Mul, vec![]),
                    make(Opcode::Constant, vec![7]),
                    make(Opcode::Constant, vec![8]),
                    make(Opcode::Hash, vec![3]),
                    make(Opcode::Pop, vec![]),
                ],
            ),
        ];
        run_compile_test(tests);
    }

    #[test]
    fn test_array_literals() {
        let tests = vec![
            (
                "[]",
                vec![],
                vec![make(Opcode::Array, vec![0]), make(Opcode::Pop, vec![])],
            ),
            (
                "[1, 2, 3]",
                vec![
                    Constant::Integer(1),
                    Constant::Integer(2),
                    Constant::Integer(3),
                ],
                vec![
                    make(Opcode::Constant, vec![0]),
                    make(Opcode::Constant, vec![1]),
                    make(Opcode::Constant, vec![2]),
                    make(Opcode::Array, vec![3]),
                    make(Opcode::Pop, vec![]),
                ],
            ),
            (
                "[1+2, 2*3, 3-1]",
                vec![
                    Constant::Integer(1),
                    Constant::Integer(2),
                    Constant::Integer(2),
                    Constant::Integer(3),
                    Constant::Integer(3),
                    Constant::Integer(1),
                ],
                vec![
                    make(Opcode::Constant, vec![0]),
                    make(Opcode::Constant, vec![1]),
                    make(Opcode::Add, vec![]),
                    make(Opcode::Constant, vec![2]),
                    make(Opcode::Constant, vec![3]),
                    make(Opcode::Mul, vec![]),
                    make(Opcode::Constant, vec![4]),
                    make(Opcode::Constant, vec![5]),
                    make(Opcode::Sub, vec![]),
                    make(Opcode::Array, vec![3]),
                    make(Opcode::Pop, vec![]),
                ],
            ),
        ];
        run_compile_test(tests);
    }

    #[test]
    fn test_string_expression() {
        let tests = vec![
            (
                r#""hello rust""#,
                vec![Constant::String("hello rust".to_string())],
                vec![make(Opcode::Constant, vec![0]), make(Opcode::Pop, vec![])],
            ),
            (
                r#""hello" + " world""#,
                vec![
                    Constant::String("hello".to_string()),
                    Constant::String(" world".to_string()),
                ],
                vec![
                    make(Opcode::Constant, vec![0]),
                    make(Opcode::Constant, vec![1]),
                    make(Opcode::Add, vec![]),
                    make(Opcode::Pop, vec![]),
                ],
            ),
        ];
        run_compile_test(tests);
    }

    #[test]
    fn test_define() {
        let expected = {
            let mut map = HashMap::new();
            map.insert(
                "a",
                Symbol {
                    name: "a".to_string(),
                    scope: SymbolScope::Global,
                    index: 0,
                },
            );
            map.insert(
                "b",
                Symbol {
                    name: "b".to_string(),
                    scope: SymbolScope::Global,
                    index: 1,
                },
            );
            map
        };
        let mut global = SymbolTable::new();
        let a = global.define("a");
        assert_eq!(a, expected["a"]);
        let b = global.define("b");
        assert_eq!(b, expected["b"]);
    }

    #[test]
    fn test_resolve() {
        let mut global = SymbolTable::new();
        let _a = global.define("a");
        let _b = global.define("b");
        let expected = {
            let mut map = HashMap::new();
            map.insert(
                "a",
                Symbol {
                    name: "a".to_string(),
                    scope: SymbolScope::Global,
                    index: 0,
                },
            );
            map.insert(
                "b",
                Symbol {
                    name: "b".to_string(),
                    scope: SymbolScope::Global,
                    index: 1,
                },
            );
            map
        };
        for (k, v) in expected {
            let x = global
                .resolve(k)
                .expect(&format!("name {} not resolvable", k));
            assert_eq!(x, v.clone());
        }
    }

    #[test]
    fn test_resolve_local() {
        let global = create_rc_ref_cell(SymbolTable::new());
        global.borrow_mut().define("a");
        global.borrow_mut().define("b");
        let first = SymbolTable::new_enclosed(global.clone());
        first.borrow_mut().define("c");
        first.borrow_mut().define("d");
        let second = SymbolTable::new_enclosed(global.clone());
        second.borrow_mut().define("e");
        second.borrow_mut().define("f");
        let expected = hash! {
                "a"=> Symbol {
                name: "a".to_string(),
                scope: SymbolScope::Global,
                index: 0
                },
                "b"=> Symbol {
                name: "b".to_string(),
                scope: SymbolScope::Global,
                index: 1
                },
                "c"=> Symbol {
                    name: "c".to_string(),
                    scope: SymbolScope::Local,
                    index: 0
                },
                "d"=> Symbol {
                    name: "d".to_string(),
                    scope: SymbolScope::Local,
                    index: 1
                },
        };
        for (k, v) in expected {
            let x = first
                .borrow_mut()
                .resolve(k)
                .expect(&format!("name {} not resolvable", k));
            assert_eq!(x, v.clone());
        }
        let expected = hash! {
                "a"=> Symbol {
                name: "a".to_string(),
                scope: SymbolScope::Global,
                index: 0
                },
                 "b"=> Symbol {
                name: "b".to_string(),
                scope: SymbolScope::Global,
                index: 1
                },
                "e"=> Symbol {
                    name: "e".to_string(),
                    scope: SymbolScope::Local,
                    index: 0
                },
                "f"=> Symbol {
                    name: "f".to_string(),
                    scope: SymbolScope::Local,
                    index: 1
                },
        };
        for (k, v) in expected {
            let x = second
                .borrow_mut()
                .resolve(k)
                .expect(&format!("name {} not resolvable", k));
            assert_eq!(x, v.clone());
        }
    }

    #[test]
    fn test_global_statement() {
        let tests = vec![(
            r"
            let a = 1;
            let b = 2;
            a
            ",
            vec![Constant::Integer(1), Constant::Integer(2)],
            vec![
                make(Opcode::Constant, vec![0]),
                make(Opcode::SetGlobal, vec![0]),
                make(Opcode::Constant, vec![1]),
                make(Opcode::SetGlobal, vec![1]),
                make(Opcode::GetGlobal, vec![0]),
                make(Opcode::Pop, vec![]),
            ],
        )];
        run_compile_test(tests);
    }

    #[test]
    fn test_condition_expression() {
        let tests = vec![
            (
                "if true { 10 } else { 2333 }; 678",
                vec![
                    Constant::Integer(10),
                    Constant::Integer(2333),
                    Constant::Integer(678),
                ],
                vec![
                    // 0000
                    make(Opcode::True, vec![]),
                    // 0001
                    make(Opcode::JumpIfNotTruthy, vec![10]),
                    // 0004
                    make(Opcode::Constant, vec![0]),
                    // 0007
                    make(Opcode::JumpAlways, vec![13]),
                    // 0010
                    make(Opcode::Constant, vec![1]),
                    // 0013
                    make(Opcode::Pop, vec![]),
                    // 0014
                    make(Opcode::Constant, vec![2]),
                    // 0017
                    make(Opcode::Pop, vec![]),
                ],
            ),
            (
                "if true { 10 }; 678",
                vec![Constant::Integer(10), Constant::Integer(678)],
                vec![
                    // 0000
                    make(Opcode::True, vec![]),
                    // 0001
                    make(Opcode::JumpIfNotTruthy, vec![10]),
                    // 0004
                    make(Opcode::Constant, vec![0]),
                    // 0007
                    make(Opcode::JumpAlways, vec![11]),
                    // 0010
                    make(Opcode::Null, vec![]),
                    // 0011
                    make(Opcode::Pop, vec![]),
                    // 0012
                    make(Opcode::Constant, vec![1]),
                    // 0015
                    make(Opcode::Pop, vec![]),
                ],
            ),
        ];
        run_compile_test(tests);
    }

    #[test]
    fn test_unary_expr() {
        let tests = vec![
            (
                "!true",
                vec![],
                vec![
                    code::make(Opcode::True, vec![]),
                    code::make(Opcode::Not, vec![]),
                    code::make(Opcode::Pop, vec![]),
                ],
            ),
            (
                "-1",
                vec![Constant::Integer(1)],
                vec![
                    code::make(Opcode::Constant, vec![0]),
                    code::make(Opcode::Neg, vec![]),
                    code::make(Opcode::Pop, vec![]),
                ],
            ),
        ];
        run_compile_test(tests);
    }

    #[test]
    fn test_boolean_expression() {
        let tests = vec![
            (
                "true",
                vec![],
                vec![
                    code::make(Opcode::True, vec![]),
                    code::make(Opcode::Pop, vec![]),
                ],
            ),
            (
                "false",
                vec![],
                vec![
                    code::make(Opcode::False, vec![]),
                    code::make(Opcode::Pop, vec![]),
                ],
            ),
            (
                "1>2",
                vec![Constant::Integer(1), Constant::Integer(2)],
                vec![
                    code::make(Opcode::Constant, vec![0]),
                    code::make(Opcode::Constant, vec![1]),
                    code::make(Opcode::GreaterThan, vec![]),
                    code::make(Opcode::Pop, vec![]),
                ],
            ),
            (
                "1<2",
                vec![Constant::Integer(1), Constant::Integer(2)],
                vec![
                    code::make(Opcode::Constant, vec![0]),
                    code::make(Opcode::Constant, vec![1]),
                    code::make(Opcode::LessThan, vec![]),
                    code::make(Opcode::Pop, vec![]),
                ],
            ),
            (
                "1==2",
                vec![Constant::Integer(1), Constant::Integer(2)],
                vec![
                    code::make(Opcode::Constant, vec![0]),
                    code::make(Opcode::Constant, vec![1]),
                    code::make(Opcode::Equal, vec![]),
                    code::make(Opcode::Pop, vec![]),
                ],
            ),
            (
                "1!=2",
                vec![Constant::Integer(1), Constant::Integer(2)],
                vec![
                    code::make(Opcode::Constant, vec![0]),
                    code::make(Opcode::Constant, vec![1]),
                    code::make(Opcode::NotEqual, vec![]),
                    code::make(Opcode::Pop, vec![]),
                ],
            ),
        ];
        run_compile_test(tests);
    }

    #[test]
    fn test_read_operands() {
        let tests = vec![(Opcode::Constant, vec![65535], 2)];
        for (op_code, operands, bytes_read) in tests {
            let def = op_code.definition();
            let insts = code::make(op_code, operands.clone());
            let (operand_reads, n) = code::read_operands(def, &insts[1..]);
            if n != bytes_read {
                panic!("n wrong. want: {}, got: {}", bytes_read, n);
            }
            for (i, want) in operands.iter().enumerate() {
                if *want != operand_reads[i] {
                    panic!("operand wrong. want: {}, got: {}", want, operand_reads[i]);
                }
            }
        }
    }

    #[test]
    fn test_instruction_string() {
        let tests = vec![
            (
                vec![
                    _make_const(1),
                    _make_const(2),
                    _make_const(65534),
                    _make_noop(Opcode::Add),
                    _make_noop(Opcode::Pop),
                    _make(Opcode::SetLocal, 0),
                    _make(Opcode::GetLocal, 0),
                ]
                .concat(),
                r"0000 OpConstant(0) 01
0003 OpConstant(0) 02
0006 OpConstant(0) FFFE
0009 OpAdd(5)
0010 OpPop(4)
0011 OpSetLocal(21) 00
0013 OpGetLocal(22) 00
",
            ),
            (
                vec![
                    _make_noop(Opcode::Add),
                    _make(Opcode::GetLocal, 0),
                    _make_const(0),
                    _make_closure(65535, 255),
                ]
                .concat(),
                r"0000 OpAdd(5)
0001 OpGetLocal(22) 00
0003 OpConstant(0) 00
0006 OpClosure(24) FFFF FF
",
            ),
        ];
        for (actual, expected) in tests {
            assert_eq!(code::_print_instructions(&actual), expected);
        }
    }

    #[test]
    fn test_integer_arithmetic() {
        let tests = vec![
            (
                "1+2;",
                vec![Constant::Integer(1), Constant::Integer(2)],
                vec![
                    code::make(Opcode::Constant, vec![0]),
                    code::make(Opcode::Constant, vec![1]),
                    code::make(Opcode::Add, vec![]),
                    code::make(Opcode::Pop, vec![]),
                ],
            ),
            (
                "1;\
                              2",
                vec![Constant::Integer(1), Constant::Integer(2)],
                vec![
                    code::make(Opcode::Constant, vec![0]),
                    code::make(Opcode::Pop, vec![]),
                    code::make(Opcode::Constant, vec![1]),
                    code::make(Opcode::Pop, vec![]),
                ],
            ),
        ];
        run_compile_test(tests);
    }

    #[test]
    fn test_make() {
        let tests = vec![
            (
                Opcode::Constant,
                vec![0xFFFE],
                vec![Opcode::Constant as u8, 0xFF, 0xFE],
            ),
            (
                Opcode::Closure,
                vec![65534, 255],
                vec![Opcode::Closure as u8, 0xFF, 0xFE, 0xFF],
            ),
            (Opcode::SetLocal, vec![0], vec![Opcode::SetLocal as u8, 0]),
            (Opcode::Add, vec![], vec![Opcode::Add as u8]),
        ];
        for (op, operands, expected) in tests.into_iter() {
            let instructions = code::make(op, operands);
            assert_eq!(instructions, expected)
        }
    }

    fn run_compile_test(tests: Vec<(&str, Vec<Constant>, Vec<Instructions>)>) {
        for (input, expected_constants, expected_instructions) in tests {
            let program = Program::_new(input);
            let mut compiler = Compiler::_new();
            match compiler.compile(program) {
                Ok(byte_code) => {
                    let constants = byte_code.constants.borrow().clone();
                    // dbg!(&compiler);
                    assert_eq!(
                        constants,
                        expected_constants,
                        "\nconstant:\n{}",
                        constants
                            .iter()
                            .map(|c| c.to_object())
                            .fold(String::new(), |a, b| a + "\n" + &b.to_string())
                    );
                    assert_eq!(
                        _print_instructions(&byte_code.instructions),
                        _print_instructions(&expected_instructions.concat()),
                        "\ninstructions:\n{}",
                        _print_instructions(&byte_code.instructions)
                    );
                }
                Err(err) => panic!("input: {}, \nerror:{:?}", input, err),
            }
        }
    }
}
