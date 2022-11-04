use errors::ErrorReporter;
use value::Value;

use crate::{interpreter::Interpreter, parser::Parser};

mod ast;
mod callable;
mod env;
pub mod errors;
pub mod interpreter;
mod lexer;
mod parser;
mod token;
pub mod value;

pub fn eval<'src>(
    source: &'src str,
    error_reporter: impl ErrorReporter,
    interpreter: &mut Interpreter,
) -> Option<Value> {
    let mut parser = Parser::new(source);

    match parser.parse() {
        Ok(stmts) => match interpreter.interpret(&stmts) {
            Ok(v) => Some(v),
            Err(errs) => {
                for err in errs {
                    println!("{}", err);
                }

                None
            }
        },
        Err(errors) => {
            println!("parsing failed!");
            for err in errors {
                error_reporter.report(source, &err);
            }

            None
        }
    }
}
