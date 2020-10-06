#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::compiler::code::{self, make, print_instructions, Constant, Opcode};
    use crate::compiler::symbol_table::{Symbol, SymbolTable, GLOBAL_SCOPE};
    use crate::compiler::{Compiler, Instructions};
    use crate::core::base::ast::Program;

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
                    scope: GLOBAL_SCOPE,
                    index: 0,
                },
            );
            map.insert(
                "b",
                Symbol {
                    name: "b".to_string(),
                    scope: GLOBAL_SCOPE,
                    index: 1,
                },
            );
            map
        };
        let mut global = SymbolTable::default();
        let a = global.define("a");
        assert_eq!(a, expected["a"]);
        let b = global.define("b");
        assert_eq!(b, expected["b"]);
    }

    #[test]
    fn test_resolve() {
        let mut global = SymbolTable::default();
        let _a = global.define("a");
        let _b = global.define("b");
        let expected = {
            let mut map = HashMap::new();
            map.insert(
                "a",
                Symbol {
                    name: "a".to_string(),
                    scope: GLOBAL_SCOPE,
                    index: 0,
                },
            );
            map.insert(
                "b",
                Symbol {
                    name: "b".to_string(),
                    scope: GLOBAL_SCOPE,
                    index: 1,
                },
            );
            map
        };
        for (k, v) in &expected {
            let x = global
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
        let tests = vec![(
            vec![
                code::make(Opcode::Constant, vec![1]),
                code::make(Opcode::Constant, vec![2]),
                code::make(Opcode::Constant, vec![65534]),
                code::make(Opcode::Add, vec![]),
                code::make(Opcode::Pop, vec![]),
            ]
            .concat(),
            r"0000 OpConstant 1
0003 OpConstant 2
0006 OpConstant 65534
0009 OpAdd
0010 OpPop
",
        )];
        for (actual, expected) in tests {
            assert_eq!(code::print_instructions(actual), expected);
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

    fn run_compile_test(tests: Vec<(&str, Vec<Constant>, Vec<Instructions>)>) {
        for (input, expected_constants, expected_instructions) in tests {
            let program = Program::new(input);
            let mut compiler = Compiler::_new();
            match compiler.compile(program) {
                Ok(byte_code) => {
                    let constants = byte_code.constants.borrow().clone();
                    // .borrow()
                    // .iter()
                    // .map(|c| c.to_object())
                    // .collect::<Vec<Object>>();
                    assert_eq!(constants, expected_constants, "bytecode: {:?}", byte_code);
                    assert_eq!(
                        print_instructions(byte_code.instructions.clone()),
                        print_instructions(expected_instructions.concat()),
                    );
                    println!("{}", print_instructions(byte_code.instructions));
                }
                Err(err) => panic!("{:?}", err),
            }
        }
    }

    #[test]
    fn test_make() {
        let tests = vec![
            (
                Opcode::Constant,
                vec![0xFFFE],
                vec![Opcode::Constant as u8, 0xFF, 0xFE],
            ),
            (Opcode::Add, vec![], vec![Opcode::Add as u8]),
        ];
        for (op, operands, expected) in tests.into_iter() {
            let instructions = code::make(op, operands);
            assert_eq!(instructions, expected)
        }
    }
}
