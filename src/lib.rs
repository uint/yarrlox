use errors::ErrorReporter;

use crate::parser::Parser;

mod ast;
pub mod errors;
mod lexer;
mod parser;

pub fn eval(source: &str, _error_reporter: impl ErrorReporter) -> String {
    let mut parser = Parser::new(source);
    let expr = parser.parse_expr();

    format!("{:#?}", expr)
}
