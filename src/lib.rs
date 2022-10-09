use errors::ErrorReporter;

use crate::{errors::Error, lexer::Token};

// TODO: remove this import by better encapsulating the lexer
use logos::Logos;

mod ast;
pub mod errors;
mod lexer;
mod parser;

pub fn eval(source: &str, error_reporter: impl ErrorReporter) -> String {
    let tokens: Vec<_> = Token::lexer(source).spanned().collect();

    for (token, span) in tokens.iter() {
        // should this translation of tokens -> errors happen somewhere else?
        match token {
            Token::InvalidToken => {
                error_reporter.report(source, &Error::new(span.clone(), "invalid token"))
            }
            Token::UnterminatedBlockComment => error_reporter.report(
                source,
                &Error::new(span.clone(), "unterminated block comment"),
            ),
            _ => {}
        }
    }

    format!("{:?}", tokens)
}
