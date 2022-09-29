#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Error {
    line: u32,
    msg: String,
}

pub trait ErrorReporter {
    fn report(&self, source: &str, e: &Error);
}

pub struct SimpleReporter;

impl ErrorReporter for SimpleReporter {
    fn report(&self, _source: &str, e: &Error) {
        eprintln!("Error on line {}: {}", e.line, e.msg);
    }
}
