use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use crate::compiler::Compiler;
use crate::eval::{evaluator, Environment};
use crate::object::Object;
use crate::parser::Parser;
use crate::vm::Vm;
use crate::Engine;

pub fn benchmark(engine: Engine) {
    println!("Welcome to the 👽 programming language in {}", engine);
    // todo 优化中，go版本目前n=35是4.5s
    // no optimized, n = 21, takes 10s.
    // optimized 1, n = 30, takes 2s.
    // optimized 2, n = 35, takes 7.8s.
    // optimized 3, n = 35, takes 3.4s.//rust更新1.47之后变4.4s了
    // optimized 4, n = 35, takes 3.35s
    // optimized 5, n = 35, takes 2.3s, n = 36, takes 4.4s
    // optimized 6, n = 35, takes 2.25s, n = 36, takes 4.4s
    // optimized 7, n = 35, takes 2.43s, n = 36, takes 4.0s// rust 2021 更新
    // 2024 rustc 1.83.0 macos (n=36, takes 3.66
    let n = 36;
    let code = &format!(
        r"
        let fibonacci = fn(x) {{
             if x < 2 {{
                 return x
             }} else {{
                 return fibonacci(x - 1) + fibonacci(x - 2)
             }}
         }};
         fibonacci({});
         // let all = 0;
         // for(let i = 0; i<10; i=i+1) {{
         //     let start = time();
         //     fibonacci();
         //     let end = time()
         //     all = all + end - start;
         // }}
         // all
         ",
        n
    );

    let mut parser = Parser::from(code);
    let program = parser.parse_program();
    let start;
    let result = match engine {
        Engine::Eval => {
            let env = Rc::new(RefCell::new(Environment::new()));
            start = Instant::now();
            evaluator::eval(&program, env).expect("eval error")
        }
        Engine::Compile => {
            let mut compiler = Compiler::new();
            let byte_code = compiler.compile(&program).expect("compile error");
            let cs = &byte_code.constants;
            for x in cs.iter() {
                println!("-----");
                println!("{}", x);
            }
            let mut vm = Vm::new(byte_code);
            start = Instant::now();
            let r = vm.run().unwrap_or_else(|err| {
                println!("err: {}", err);
                Rc::new(Object::Null)
            });
            Object::clone(&r)
            // let result = vm.run().expect("runtime error");
            // Object::clone(&result)
        }
    };
    let duration = start.elapsed();
    println!("{:?}, No.{}, result: {}", duration, n, result);
}
