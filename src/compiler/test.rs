#[cfg(test)]
mod tests {
    use crate::compiler::code::{self, make, print_instructions, Opcode};
    use crate::compiler::{Compiler, Instructions};
    use crate::core::base::ast::Program;
    use crate::object::Object;

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
                vec![Object::Integer(10), Object::Integer(678)],
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
                vec![Object::Integer(1)],
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
                vec![Object::Integer(1), Object::Integer(2)],
                vec![
                    code::make(Opcode::Constant, vec![0]),
                    code::make(Opcode::Constant, vec![1]),
                    code::make(Opcode::GreaterThan, vec![]),
                    code::make(Opcode::Pop, vec![]),
                ],
            ),
            (
                "1<2",
                vec![Object::Integer(1), Object::Integer(2)],
                vec![
                    code::make(Opcode::Constant, vec![0]),
                    code::make(Opcode::Constant, vec![1]),
                    code::make(Opcode::LessThan, vec![]),
                    code::make(Opcode::Pop, vec![]),
                ],
            ),
            (
                "1==2",
                vec![Object::Integer(1), Object::Integer(2)],
                vec![
                    code::make(Opcode::Constant, vec![0]),
                    code::make(Opcode::Constant, vec![1]),
                    code::make(Opcode::Equal, vec![]),
                    code::make(Opcode::Pop, vec![]),
                ],
            ),
            (
                "1!=2",
                vec![Object::Integer(1), Object::Integer(2)],
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
            let (operand_reads, n) = code::read_operands(def, &insts, 1);
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
                vec![Object::Integer(1), Object::Integer(2)],
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
                vec![Object::Integer(1), Object::Integer(2)],
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

    fn run_compile_test(tests: Vec<(&str, Vec<Object>, Vec<Instructions>)>) {
        for (input, expected_constants, expected_instructions) in tests {
            let program = Program::new(input);
            let mut compiler = Compiler::new();
            match compiler.compile(program) {
                Ok(byte_code) => {
                    assert_eq!(
                        byte_code.constants, expected_constants,
                        "bytecode: {:?}",
                        byte_code
                    );
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
