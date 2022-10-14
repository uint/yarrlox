use errors::ErrorReporter;

use crate::{errors::Error, lexer::Token, parser::Parser};

// TODO: remove this import by better encapsulating the lexer
use logos::Logos;

mod ast;
pub mod errors;
mod lexer;
mod parser;

pub fn eval(source: &str, error_reporter: impl ErrorReporter) -> String {
    let mut parser = Parser::new(source);
    let expr = parser.parse_expr();

    format!("{:#?}", expr)
}
