use yarrlox::{
    interpreter::{Interpreter, InterpreterError, InterpreterOutput},
    value::Value,
    EvalErrors, ParserErrorKind, ResolverError,
};

pub struct RunResults<'src> {
    v: Result<Value, EvalErrors<'src>>,
    output: String,
}

impl<'src> RunResults<'src> {
    #[track_caller]
    pub fn assert_output(&self, expected: &str) {
        if let Err(err) = &self.v {
            panic!("interpreter failed with: {}", err);
        }
        assert_eq!(self.output.trim(), expected.trim());
    }

    #[track_caller]
    pub fn assert_v(&self, expected: Value) {
        assert_eq!(self.v.as_ref().cloned().unwrap(), expected);
    }

    #[track_caller]
    pub fn assert_syn_err(self, expected: &[ParserErrorKind]) {
        assert_eq!(
            self.v
                .unwrap_err()
                .unwrap_syn()
                .into_iter()
                .map(|err| err.error_kind)
                .collect::<Vec<_>>(),
            expected
        );
    }

    #[track_caller]
    pub fn assert_resolution_err(self, expected: ResolverError) {
        assert_eq!(self.v.unwrap_err().unwrap_resolution(), expected);
    }

    #[track_caller]
    pub fn assert_runtime_err(self, expected: &[InterpreterError]) {
        assert_eq!(self.v.unwrap_err().unwrap_runtime(), expected);
    }
}

pub fn run(source: &str) -> RunResults<'_> {
    let mut interpreter = Interpreter::new(InterpreterOutput::String(Vec::new()));
    let v = yarrlox::eval(
        source.as_ref(),
        yarrlox::errors::SimpleReporter,
        &mut interpreter,
    );

    RunResults {
        v,
        output: interpreter.get_output(),
    }
}
