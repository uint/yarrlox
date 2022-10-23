use errors::ErrorReporter;

use crate::{interpreter::interpret, parser::Parser};

mod ast;
pub mod errors;
mod interpreter;
mod lexer;
mod parser;
mod token;
mod value;

pub fn eval(source: &str, _error_reporter: impl ErrorReporter) -> String {
    let mut parser = Parser::new(source);
    let expr = parser.parse();

    match expr {
        Some(expr) => format!("{}", interpret(&expr).unwrap()),
        None => String::from("there was some error! boo"),
    }
}
