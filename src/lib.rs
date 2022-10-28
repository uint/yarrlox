use errors::ErrorReporter;

use crate::{interpreter::Interpreter, parser::Parser};

mod ast;
mod env;
pub mod errors;
pub mod interpreter;
mod lexer;
mod parser;
mod token;
mod value;

pub fn eval<'src>(
    source: &'src str,
    error_reporter: impl ErrorReporter,
    interpreter: &mut Interpreter,
) {
    let mut parser = Parser::new(source);

    match parser.parse() {
        Ok(stmts) => {
            for err in interpreter.interpret(&stmts) {
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
