use yarrlox::{
    interpreter::{Interpreter, InterpreterOutput},
    value::Value,
};

pub struct RunResult {
    v: Option<Value>,
    output: String,
}

impl RunResult {
    #[track_caller]
    pub fn assert_output(&self, expected: &str) {
        assert_eq!(self.output.trim(), expected.trim());
    }

    #[track_caller]
    pub fn assert_v(&self, expected: Value) {
        assert_eq!(self.v.clone().unwrap(), expected);
    }
}

pub fn run(source: impl AsRef<str>) -> RunResult {
    let mut interpreter = Interpreter::new(InterpreterOutput::String(Vec::new()));
    let v = yarrlox::eval(
        source.as_ref(),
        yarrlox::errors::SimpleReporter,
        &mut interpreter,
    );

    RunResult {
        v,
        output: interpreter.get_output(),
    }
}
