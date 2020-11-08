#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::rc::Rc;

    use crate::compiler::code::{
        self, _make, _make_closure, _make_const, _make_noop, make, print_instructions, Opcode,
    };
    use crate::compiler::symbol_table::{Symbol, SymbolScope, SymbolTable};
    use crate::compiler::{Compiler, Instructions};
    use crate::create_rc_ref_cell;
    use crate::object::{CompiledFunction, Object};
    use crate::parser::base::ast::Program;

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
    fn for_statement() {
        let inputs = vec![(
            r"
            for(let i = 0; i < 10; i = i + 1) {

            }
            ",
            vec![Object::Integer(0), Object::Integer(10), Object::Integer(1)],
            vec![
                //init
                _make_const(0),// 0
                _make_noop(Opcode::SetGlobal0),// 1
                //cond
                _make_noop(Opcode::GetGlobal0),// 2
                _make_const(1),// 3
                _make(Opcode::JumpIfNotLess, 11+3),// 4
                //loop blocks
                //after
                _make_noop(Opcode::GetGlobal0),// 7
                _make_const(2),//8
                _make_noop(Opcode::Add),//9
                _make_noop(Opcode::SetGlobal0),//10
                //always jump to start
                _make(Opcode::JumpAlways, 2),//11
            ],
        )];
        run_compile_test(inputs);
    }
    #[test]
    fn recursive_function() {
        let count_down_const = make_fun_object_with_name(
            "countDown",
            vec![
                _make_noop(Opcode::CurrentClosure),
                _make_noop(Opcode::GetLocal0),
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
                    Object::Integer(1),
                    count_down_const.clone(),
                    Object::Integer(2),
                ],
                vec![
                    _make_closure(1, 0),
                    _make_noop(Opcode::SetGlobal0),
                    _make_noop(Opcode::GetGlobal0),
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
                    // 1
                    countDown(1)
                }
                wrapper()
                          ",
                vec![
                    Object::Integer(1),
                    count_down_const,
                    Object::Integer(1),
                    make_fun_object_with_name(
                        "wrapper",
                        vec![
                            _make_closure(1, 0),
                            _make_noop(Opcode::SetLocal0),
                            _make_noop(Opcode::GetLocal0),
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
                    _make_noop(Opcode::SetGlobal0),
                    _make_noop(Opcode::GetGlobal0),
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
                    make_fun_object(
                        vec![
                            _make(Opcode::GetFree, 0),
                            _make_noop(Opcode::GetLocal0),
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        1,
                        1,
                    ),
                    make_fun_object(
                        vec![
                            _make_noop(Opcode::GetLocal0),
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
                    make_fun_object(
                        vec![
                            _make(Opcode::GetFree, 0),
                            _make(Opcode::GetFree, 1),
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::GetLocal0),
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        1,
                        1,
                    ),
                    make_fun_object(
                        vec![
                            _make(Opcode::GetFree, 0),
                            _make_noop(Opcode::GetLocal0),
                            _make_closure(0, 2),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        1,
                        1,
                    ),
                    make_fun_object(
                        vec![
                            _make_noop(Opcode::GetLocal0),
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
                    Object::Integer(55),
                    Object::Integer(66),
                    Object::Integer(77),
                    Object::Integer(88),
                    make_fun_object(
                        vec![
                            _make_const(3),
                            _make_noop(Opcode::SetLocal0),  // declare c
                            _make_noop(Opcode::GetGlobal0), // global
                            _make(Opcode::GetFree, 0),      // free a
                            _make_noop(Opcode::Add),
                            _make(Opcode::GetFree, 1), // free b
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::GetLocal0), // local c
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        1,
                        0,
                    ),
                    make_fun_object(
                        vec![
                            _make_const(2),
                            _make_noop(Opcode::SetLocal0), // declare b
                            _make(Opcode::GetFree, 0),     // free a
                            _make_noop(Opcode::GetLocal0),
                            _make_closure(4, 2), // closure
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        1,
                        0,
                    ),
                    make_fun_object(
                        vec![
                            _make_const(1),
                            _make_noop(Opcode::SetLocal0), // declare a
                            _make_noop(Opcode::GetLocal0),
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
                    _make_noop(Opcode::SetGlobal0),
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
            vec![Object::Integer(1)],
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
                    Object::Integer(55),
                    make_fun_object(
                        vec![
                            _make_noop(Opcode::GetGlobal0),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        0,
                        0,
                    ),
                ],
                vec![
                    _make_const(0),
                    _make_noop(Opcode::SetGlobal0),
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
                    Object::Integer(55),
                    make_fun_object(
                        vec![
                            _make_const(0),
                            _make_noop(Opcode::SetLocal0),
                            _make_noop(Opcode::GetLocal0),
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
                    Object::Integer(1),
                    Object::Integer(2),
                    make_fun_object(
                        vec![
                            _make_const(0),
                            _make_noop(Opcode::SetLocal0),
                            _make_const(1),
                            _make_noop(Opcode::SetLocal1),
                            _make_noop(Opcode::GetLocal0),
                            _make_noop(Opcode::GetLocal1),
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
                    Object::Integer(10),
                    make_fun_object_with_name(
                        "sum",
                        vec![
                            _make_noop(Opcode::GetLocal0),
                            _make_noop(Opcode::GetLocal1),
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::SetLocal2),
                            _make_noop(Opcode::GetLocal2),
                            _make_noop(Opcode::GetGlobal0),
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        3,
                        2,
                    ),
                    Object::Integer(1),
                    Object::Integer(2),
                    Object::Integer(3),
                    Object::Integer(4),
                    make_fun_object_with_name(
                        "outer",
                        vec![
                            _make_noop(Opcode::GetGlobal1),
                            _make_const(2),
                            _make_const(3),
                            _make(Opcode::Call, 2),
                            _make_noop(Opcode::GetGlobal1),
                            _make_const(4),
                            _make_const(5),
                            _make(Opcode::Call, 2),
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::GetGlobal0),
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
                    _make_noop(Opcode::SetGlobal0),
                    _make_closure(1, 0),
                    _make_noop(Opcode::SetGlobal1),
                    _make_closure(6, 0),
                    _make_noop(Opcode::SetGlobal2),
                    _make_noop(Opcode::GetGlobal2),
                    _make(Opcode::Call, 0),
                    _make_noop(Opcode::GetGlobal0),
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
                    Object::Integer(123),
                    Object::Integer(234),
                    make_fun_object_with_name(
                        "one_arg",
                        vec![
                            _make_noop(Opcode::GetLocal0),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        1,
                        1,
                    ),
                    Object::Integer(233),
                ],
                vec![
                    _make_const(0),
                    _make_noop(Opcode::SetGlobal0),
                    _make_const(1),
                    _make_noop(Opcode::SetGlobal1),
                    _make_closure(2, 0),
                    _make_noop(Opcode::SetGlobal2),
                    _make_noop(Opcode::GetGlobal2),
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
                    Object::Integer(111),
                    make_fun_object_with_name(
                        "many_arg",
                        vec![
                            _make_noop(Opcode::GetLocal0),
                            _make_noop(Opcode::GetLocal1),
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::GetLocal2),
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::GetGlobal0),
                            _make_noop(Opcode::Add),
                            _make_noop(Opcode::ReturnValue),
                        ]
                        .concat(),
                        3,
                        3,
                    ),
                    Object::Integer(2),
                    Object::Integer(3),
                    Object::Integer(4),
                ],
                vec![
                    _make_const(0),
                    _make_noop(Opcode::SetGlobal0),
                    _make_closure(1, 0),
                    _make_noop(Opcode::SetGlobal1),
                    _make_noop(Opcode::GetGlobal1),
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
                    Object::Integer(1),
                    Object::Integer(12),
                    make_fun_object(
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
                    make_fun_object_with_name(
                        "one_arg",
                        vec![_make_noop(Opcode::Return)].concat(),
                        1,
                        1,
                    ),
                    Object::Integer(233),
                ],
                vec![
                    _make_closure(0, 0),
                    _make_noop(Opcode::SetGlobal0),
                    _make_noop(Opcode::GetGlobal0),
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
                    Object::Integer(1),
                    Object::Integer(12),
                    make_fun_object_with_name(
                        "m",
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
                    _make_noop(Opcode::SetGlobal0),
                    _make_noop(Opcode::GetGlobal0),
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
                    Object::Integer(1),
                    make_fun_object_with_name(
                        "one",
                        vec![_make_const(0), _make_noop(Opcode::ReturnValue)].concat(),
                        0,
                        0,
                    ),
                    Object::Integer(2),
                    make_fun_object_with_name(
                        "two",
                        vec![_make_const(2), _make_noop(Opcode::ReturnValue)].concat(),
                        0,
                        0,
                    ),
                ],
                vec![
                    // let one = fn1
                    _make_closure(1, 0),
                    _make_noop(Opcode::SetGlobal0),
                    // let two = fn2
                    _make_closure(3, 0),
                    _make_noop(Opcode::SetGlobal1),
                    // one()
                    _make_noop(Opcode::GetGlobal0),
                    _make(Opcode::Call, 0),
                    // two()
                    _make_noop(Opcode::GetGlobal1),
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
                    Object::Integer(5),
                    Object::Integer(10),
                    make_fun_object(
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
                    Object::Integer(5),
                    Object::Integer(10),
                    make_fun_object(
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
                    Object::Integer(1),
                    Object::Integer(2),
                    make_fun_object(
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
                vec![Object::Integer(1), Object::Integer(2)],
                vec![
                    _make_const(0),
                    _make_noop(Opcode::SetGlobal0),
                    _make_const(1),
                    _make_noop(Opcode::SetGlobal0),
                    _make_noop(Opcode::GetGlobal0),
                    _make_noop(Opcode::Pop),
                ],
            ),
            (
                "let arr = [1,2,3]; arr[2] = 0;arr",
                vec![
                    Object::Integer(1),
                    Object::Integer(2),
                    Object::Integer(3),
                    Object::Integer(2),
                    Object::Integer(0),
                    // Object::Integer(2),
                ],
                vec![
                    _make_const(0),
                    _make_const(1),
                    _make_const(2),
                    _make(Opcode::Array, 3),     //声明赋值数组
                    _make_noop(Opcode::SetGlobal0), //存arr
                    _make_const(3),                   //index
                    _make_const(4),                   //value
                    _make_noop(Opcode::SetGlobal0),         //arr[index] = value
                    _make_noop(Opcode::GetGlobal0),   //取arr
                    // make(Opcode::Constant, vec![5]),  //index
                    // make(Opcode::Index, vec![]),      //arr[index]
                    _make_noop(Opcode::Pop),
                ],
            ),
            (
                r#"let map = {1+1:2+2,"hello":5*3, 10:"yo"}; map[2]="new_data"; map"#,
                vec![
                    Object::Integer(1),
                    Object::Integer(1),
                    Object::Integer(2),
                    Object::Integer(2),
                    Object::String("hello".to_string()),
                    Object::Integer(5),
                    Object::Integer(3),
                    Object::Integer(10),
                    Object::String("yo".to_string()),
                    Object::Integer(2),
                    Object::String("new_data".to_string()),
                ],
                vec![
                    _make_const(0),
                    _make_const(1),
                    _make_noop(Opcode::Add),
                    _make_const(2),
                    _make_const(3),
                    _make_noop(Opcode::Add),
                    _make_const(4),
                    _make_const(5),
                    _make_const(6),
                    _make_noop(Opcode::Mul),
                    _make_const(7),
                    _make_const(8),
                    _make(Opcode::Hash, 3),
                    _make_noop(Opcode::SetGlobal0),    //声明初始化Map
                    _make_const(9),                 //index
                    _make_const(10),                //value
                    _make_noop(Opcode::SetGlobal0),       //map[index] = value
                    _make_noop(Opcode::GetGlobal0), //取map
                    _make_noop(Opcode::Pop),
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
                    Object::Integer(1),
                    Object::Integer(2),
                    Object::Integer(3),
                    Object::Integer(1),
                ],
                vec![
                    _make_const(0),
                    _make_const(1),
                    _make_const(2),
                    _make(Opcode::Array, 3),
                    _make_const(3),
                    _make_noop(Opcode::Index),
                    _make_noop(Opcode::Pop),
                ],
            ),
            (
                r#"{"a":1,2:"b", 1+2: "3"+ "c"}[3]"#,
                vec![
                    Object::String("a".to_string()),
                    Object::Integer(1),
                    Object::Integer(2),
                    Object::String("b".to_string()),
                    Object::Integer(1),
                    Object::Integer(2),
                    Object::String("3".to_string()),
                    Object::String("c".to_string()),
                    Object::Integer(3),
                ],
                vec![
                    _make_const(0),
                    _make_const(1),
                    _make_const(2),
                    _make_const(3),
                    _make_const(4),
                    _make_const(5),
                    _make_noop(Opcode::Add),
                    _make_const(6),
                    _make_const(7),
                    _make_noop(Opcode::Add),
                    _make(Opcode::Hash, 3),
                    _make_const(8),
                    _make_noop(Opcode::Index),
                    _make_noop(Opcode::Pop),
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
                    Object::String("a".to_string()),
                    Object::Integer(1),
                    Object::String("b".to_string()),
                    Object::Integer(2),
                    Object::String("c".to_string()),
                    Object::Integer(3),
                ],
                vec![
                    _make_const(0),
                    _make_const(1),
                    _make_const(2),
                    _make_const(3),
                    _make_const(4),
                    _make_const(5),
                    _make(Opcode::Hash, 3),
                    _make_noop(Opcode::Pop),
                ],
            ),
            (
                r#"{1+1:2+2,"hello":5*3, 10:"yo"}"#,
                vec![
                    Object::Integer(1),
                    Object::Integer(1),
                    Object::Integer(2),
                    Object::Integer(2),
                    Object::String("hello".to_string()),
                    Object::Integer(5),
                    Object::Integer(3),
                    Object::Integer(10),
                    Object::String("yo".to_string()),
                ],
                vec![
                    _make_const(0),
                    _make_const(1),
                    _make_noop(Opcode::Add),
                    _make_const(2),
                    _make_const(3),
                    _make_noop(Opcode::Add),
                    _make_const(4),
                    _make_const(5),
                    _make_const(6),
                    _make_noop(Opcode::Mul),
                    _make_const(7),
                    _make_const(8),
                    _make(Opcode::Hash, 3),
                    _make_noop(Opcode::Pop),
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
                vec![Object::Integer(1), Object::Integer(2), Object::Integer(3)],
                vec![
                    _make_const(0),
                    _make_const(1),
                    _make_const(2),
                    _make(Opcode::Array, 3),
                    _make_noop(Opcode::Pop),
                ],
            ),
            (
                "[1+2, 2*3, 3-1]",
                vec![
                    Object::Integer(1),
                    Object::Integer(2),
                    Object::Integer(2),
                    Object::Integer(3),
                    Object::Integer(3),
                    Object::Integer(1),
                ],
                vec![
                    _make_const(0),
                    _make_const(1),
                    _make_noop(Opcode::Add),
                    _make_const(2),
                    _make_const(3),
                    _make_noop(Opcode::Mul),
                    _make_const(4),
                    _make_const(5),
                    _make_noop(Opcode::Sub),
                    _make(Opcode::Array, 3),
                    _make_noop(Opcode::Pop),
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
                vec![Object::String("hello rust".to_string())],
                vec![_make_const(0), _make_noop(Opcode::Pop)],
            ),
            (
                r#""hello" + " world""#,
                vec![
                    Object::String("hello".to_string()),
                    Object::String(" world".to_string()),
                ],
                vec![
                    _make_const(0),
                    _make_const(1),
                    _make_noop(Opcode::Add),
                    _make_noop(Opcode::Pop),
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
        assert_eq!(*a, expected["a"]);
        let b = global.define("b");
        assert_eq!(*b, expected["b"]);
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
            assert_eq!(*x, v.clone());
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
            assert_eq!(*x, v.clone());
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
            assert_eq!(*x, v.clone());
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
            vec![Object::Integer(1), Object::Integer(2)],
            vec![
                _make_const(0),
                _make_noop(Opcode::SetGlobal0),
                _make_const(1),
                _make_noop(Opcode::SetGlobal1),
                _make_noop(Opcode::GetGlobal0),
                _make_noop(Opcode::Pop),
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
                    Object::Integer(10),
                    Object::Integer(2333),
                    Object::Integer(678),
                ],
                vec![
                    // 0000
                    _make_noop(Opcode::True),
                    // 0001
                    _make(Opcode::JumpIfNotTruthy, 8),
                    // 0004
                    _make_const(0),
                    // 0005
                    _make(Opcode::JumpAlways, 9),
                    // 0008
                    _make_const(1),
                    // 0009
                    _make_noop(Opcode::Pop),
                    // 0010
                    _make_const(2),
                    // 0011
                    _make_noop(Opcode::Pop),
                ],
            ),
            (
                "if true { 10 }; 678",
                vec![Object::Integer(10), Object::Integer(678)],
                vec![
                    // 0000
                    _make_noop(Opcode::True),
                    // 0001
                    _make(Opcode::JumpIfNotTruthy, 8),
                    // 0004
                    _make_const(0),
                    // 0005
                    _make(Opcode::JumpAlways, 9),
                    // 0008
                    _make_noop(Opcode::Null),
                    // 0009
                    _make_noop(Opcode::Pop),
                    // 0010
                    _make_const(1),
                    // 0011
                    _make_noop(Opcode::Pop),
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
                    code::_make_noop(Opcode::True),
                    code::_make_noop(Opcode::Not),
                    code::_make_noop(Opcode::Pop),
                ],
            ),
            (
                "-1",
                vec![Object::Integer(1)],
                vec![
                    code::_make_const(0),
                    code::_make_noop(Opcode::Neg),
                    code::_make_noop(Opcode::Pop),
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
                    code::_make_noop(Opcode::True),
                    code::_make_noop(Opcode::Pop),
                ],
            ),
            (
                "false",
                vec![],
                vec![
                    code::_make_noop(Opcode::False),
                    code::_make_noop(Opcode::Pop),
                ],
            ),
            (
                "1>2",
                vec![Object::Integer(1), Object::Integer(2)],
                vec![
                    code::_make_const(0),
                    code::_make_const(1),
                    code::_make_noop(Opcode::GreaterThan),
                    code::_make_noop(Opcode::Pop),
                ],
            ),
            (
                "1<2",
                vec![Object::Integer(1), Object::Integer(2)],
                vec![
                    code::_make_const(0),
                    code::_make_const(1),
                    code::_make_noop(Opcode::LessThan),
                    code::_make_noop(Opcode::Pop),
                ],
            ),
            (
                "1==2",
                vec![Object::Integer(1), Object::Integer(2)],
                vec![
                    code::_make_const(0),
                    code::_make_const(1),
                    code::_make_noop(Opcode::Equal),
                    code::_make_noop(Opcode::Pop),
                ],
            ),
            (
                "1!=2",
                vec![Object::Integer(1), Object::Integer(2)],
                vec![
                    code::_make_const(0),
                    code::_make_const(1),
                    code::_make_noop(Opcode::NotEqual),
                    code::_make_noop(Opcode::Pop),
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
            let (operand_reads, n) = code::read_operands(&def, &insts[1..]);
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
                    _make_noop(Opcode::SetLocal0),
                    _make_noop(Opcode::GetLocal0),
                ]
                .concat(),
                r"   0 OpConstant1
   1 OpConstant2
   2 OpConstant 65534
   5 OpAdd
   6 OpPop
   7 OpSetLocal0
   8 OpGetLocal0
",
            ),
            (
                vec![
                    _make_noop(Opcode::Add),
                    _make_noop(Opcode::GetLocal0),
                    _make_const(0),
                    _make_closure(65535, 255),
                ]
                .concat(),
                r"   0 OpAdd
   1 OpGetLocal0
   2 OpConstant0
   3 OpClosure 65535 255
",
            ),
        ];
        for (actual, expected) in tests {
            assert_eq!(code::print_instructions(&actual), expected);
        }
    }

    #[test]
    fn test_integer_arithmetic() {
        let tests = vec![
            (
                "1+2;",
                vec![Object::Integer(1), Object::Integer(2)],
                vec![
                    code::_make_const(0),
                    code::_make_const(1),
                    code::make(Opcode::Add, vec![]),
                    code::make(Opcode::Pop, vec![]),
                ],
            ),
            (
                "1;\
                              2",
                vec![Object::Integer(1), Object::Integer(2)],
                vec![
                    code::_make_const(0),
                    code::make(Opcode::Pop, vec![]),
                    code::_make_const(1),
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

    fn run_compile_test(tests: Vec<(&str, Vec<Object>, Vec<Instructions>)>) {
        for (input, expected_constants, expected_instructions) in tests {
            let program = Program::_new(input);
            let mut compiler = Compiler::new();
            match compiler.compile(&program) {
                Ok(byte_code) => {
                    let constants = byte_code.constants.clone();
                    // dbg!(&compiler);
                    assert_eq!(
                        constants
                            .iter()
                            .map(|o| Object::clone(&o))
                            .collect::<Vec<Object>>(),
                        expected_constants,
                        "\nconstant:\n{}",
                        constants
                            .iter()
                            .fold(String::new(), |a, b| a + "\n" + &b.to_string())
                    );
                    assert_eq!(
                        print_instructions(&byte_code.instructions),
                        print_instructions(&expected_instructions.concat()),
                        "\ninstructions:\n{}",
                        print_instructions(&byte_code.instructions)
                    );
                }
                Err(err) => panic!("input: {}, \nerror:{:?}", input, err),
            }
        }
    }
    fn make_fun_object(insts: Vec<u8>, num_locals: usize, num_parameters: usize) -> Object {
        Object::CompiledFunction(CompiledFunction::new(
            Rc::new(insts),
            num_locals,
            num_parameters,
        ))
    }
    fn make_fun_object_with_name(
        name: &str,
        insts: Vec<u8>,
        num_locals: usize,
        num_parameters: usize,
    ) -> Object {
        Object::CompiledFunction(CompiledFunction::with_name(
            Some(name.to_string()),
            Rc::new(insts),
            num_locals,
            num_parameters,
        ))
    }
}
