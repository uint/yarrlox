use std::fmt::Display;

use crate::token::SpannedToken;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Error<'src, E> {
    token: Option<SpannedToken<'src>>,
    error_kind: E,
}

impl<'src, E: Display + std::error::Error> Error<'src, E> {
    pub fn new(token: impl Into<Option<SpannedToken<'src>>>, error_kind: E) -> Self {
        Self {
            token: token.into(),
            error_kind,
        }
    }
}

pub trait ErrorReporter {
    fn report<E: std::error::Error>(&self, source: &str, e: &Error<'_, E>);
}

pub struct SimpleReporter;

impl ErrorReporter for SimpleReporter {
    fn report<E: std::error::Error>(&self, _source: &str, e: &Error<'_, E>) {
        // TODO: calculate the line number at least
        // bonus points: print a source code fragment and point to the problematic span
        match &e.token {
            Some(token) => {
                eprintln!(
                    "Error in span {:?}, token {:?}: {}",
                    token.span, token.token, e.error_kind
                )
            }
            None => eprintln!("Error: {}", e.error_kind),
        }
    }
}
