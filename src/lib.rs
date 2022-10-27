use errors::ErrorReporter;

use crate::{interpreter::interpret, parser::Parser};

mod ast;
pub mod errors;
mod interpreter;
mod lexer;
mod parser;
mod token;
mod value;

pub fn eval(source: &str, error_reporter: impl ErrorReporter) {
    let mut parser = Parser::new(source);
    match parser.parse() {
        Ok(stmts) => {
            for err in interpret(&stmts) {
                println!("{}", err);
            }
        }
        Err(errors) => {
            println!("parsing failed!");
            for err in errors {
                error_reporter.report(source, &err);
            }
        }
    }
}
