use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use crate::core::parser::Parser;
use crate::eval::{environment, evaluator};

pub fn benchmark() {
    let n = 30;
    let code = &format!(
        "let fibonacci = fn(x) {{
             if x < 2 {{
                 return x
             }} else {{
                 return fibonacci(x - 1) + fibonacci(x - 2)
             }}
         }};
         fibonacci({});",
        n
    );
    let env = Rc::new(RefCell::new(environment::Environment::new()));
    let mut parser = Parser::from(code);
    let program = parser.parse_program();
    // let errors = parser.errors();
    let start = Instant::now();
    let result = evaluator::eval(&program, env).unwrap();
    let duration = start.elapsed();
    println!(
        "{} s {} ms, No.{}, result: {}",
        duration.as_secs(),
        duration.as_millis(),
        n,
        result
    );
}
