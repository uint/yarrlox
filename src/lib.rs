mod ast;
mod callable;
mod env;
pub mod errors;
pub mod interpreter;
mod lexer;
pub mod parser;
mod resolver;
mod token;
pub mod value;

use errors::ErrorReporter;
use value::Value;

use crate::{interpreter::Interpreter, parser::Parser};

pub use interpreter::InterpreterError;
pub use parser::{ParserError, ParserErrorKind};
pub use resolver::ResolverError;

pub fn eval<'src>(
    source: &'src str,
    error_reporter: impl ErrorReporter,
    parser: &mut Parser,
    interpreter: &mut Interpreter,
) -> Result<Value, EvalErrors<'src>> {
    match parser.parse(source) {
        Ok(stmts) => {
            match interpreter.interpret(&stmts, parser.var_count()) {
                Ok(v) => Ok(v),
                Err(errs) => {
                    for err in errs.iter() {
                        // TODO: use the error reporter here
                        println!("{}", err);
                    }

                    Err(EvalErrors::Interpreter(errs))
                }
            }
        }
        Err(errs) => {
            println!("parsing failed!");
            for err in errs.iter() {
                error_reporter.report(source, err);
            }

            Err(EvalErrors::Syntax(errs))
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum EvalErrors<'src> {
    #[error("one or more syntax errors")]
    Syntax(Vec<ParserError<'src>>),
    #[error("{0}")]
    Resolution(#[from] ResolverError),
    #[error("one or more runtime errors")]
    Interpreter(Vec<InterpreterError>),
}

impl<'src> EvalErrors<'src> {
    pub fn unwrap_syn(self) -> Vec<ParserError<'src>> {
        if let Self::Syntax(err) = self {
            err
        } else {
            panic!()
        }
    }

    pub fn unwrap_resolution(self) -> ResolverError {
        if let Self::Resolution(err) = self {
            err
        } else {
            panic!()
        }
    }

    pub fn unwrap_runtime(self) -> Vec<InterpreterError> {
        if let Self::Interpreter(err) = self {
            err
        } else {
            panic!()
        }
    }
}
