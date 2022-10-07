use std::ops::Range;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Error {
    span: Range<usize>,
    msg: String,
}

impl Error {
    pub fn new(span: Range<usize>, msg: impl ToString) -> Self {
        Self {
            span,
            msg: msg.to_string(),
        }
    }
}

pub trait ErrorReporter {
    fn report(&self, source: &str, e: &Error);
}

pub struct SimpleReporter;

impl ErrorReporter for SimpleReporter {
    fn report(&self, _source: &str, e: &Error) {
        eprintln!("Error in span {:?}: {}", e.span, e.msg);
    }
}
