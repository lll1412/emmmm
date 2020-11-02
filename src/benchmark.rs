use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use crate::compiler::Compiler;
use crate::core::parser::Parser;
use crate::Engine;
use crate::eval::evaluator;
use crate::object::{environment, Object};
use crate::vm::Vm;

pub fn benchmark(engine: Engine) {
    println!("Welcome to the 👽 programming language in {}", engine);
    // todo 优化中，go版本目前n=35是4.5s
    // no optimized, n = 21, takes 10s.
    // optimized 1, n = 30, takes 2s.
    // optimized 2, n = 35, takes 7.8s.
    // optimized 3, n = 35, takes 3.4s.
    let n = 35;
    let code = &format!(
        r"let fibonacci = fn(x) {{
             if x < 2 {{
                 x
             }} else {{
                 fibonacci(x - 1) + fibonacci(x - 2)
             }}
         }};
         fibonacci({});",
        n
    );

    let mut parser = Parser::from(code);
    let program = parser.parse_program();
    let start;
    let result = match engine {
        Engine::Eval => {
            let env = Rc::new(RefCell::new(environment::Environment::new()));
            start = Instant::now();
            evaluator::eval(&program, env).expect("eval error")
        }
        Engine::Compile => {
            let mut compiler = Compiler::new();
            let byte_code = compiler.compile(&program).expect("compile error");
            // let cs = &byte_code.constants;
            // for x in cs.borrow().iter() {
            //     println!("-----");
            //     println!("{}", x);
            // }
            let mut vm = Vm::new(byte_code);
            start = Instant::now();
            let result = vm.run().expect("runtime error");
            Object::clone(&result)
        }
    };
    let duration = start.elapsed();
    println!(
        "{} s {} ms, No.{}, result: {}",
        duration.as_secs(),
        duration.as_millis(),
        n,
        result
    );
}
