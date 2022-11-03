use std::{
    io::{self, Write},
    path::Path,
};

use yarrlox::{interpreter::Interpreter, value::Value};

fn run_script(path: impl AsRef<Path>) -> anyhow::Result<Value> {
    let source = std::fs::read_to_string(path)?;
    let mut interpreter = Interpreter::new();
    let v = yarrlox::eval(&source, yarrlox::errors::SimpleReporter, &mut interpreter).unwrap();

    Ok(v)
}

#[test]
fn closure() {
    assert_eq!(run_script("scripts/closure.lox").unwrap(), Value::Num(5.));
}
