use yarrlox::{
    interpreter::{Interpreter, InterpreterOutput},
    value::Value,
};

struct RunResult {
    v: Option<Value>,
    output: String,
}

impl RunResult {
    #[track_caller]
    fn assert_output(&self, expected: &str) {
        assert_eq!(self.output.trim(), expected.trim());
    }

    #[track_caller]
    fn assert_v(&self, expected: Value) {
        assert_eq!(self.v.clone().unwrap(), expected);
    }
}

fn run(source: impl AsRef<str>) -> RunResult {
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

#[test]
fn closure() {
    let closure = r#"
fun fun_gen(x) {
    // this function should capture the local x = 5 variable
    fun print_x() {
        return x;
    }

    return print_x;
}

var fn = fun_gen(5);

return fn();
    "#;

    run(closure).assert_v(Value::Num(5.));
}

#[test]
fn fib() {
    let fib = r#"
fun fib(n) {
  if (n <= 1) return n;
  return fib(n - 2) + fib(n - 1);
}

for (var i = 0; i < 15; i = i + 1) {
  print fib(i);
}
    "#;

    run(fib).assert_output(
        r#"
0
1
1
2
3
5
8
13
21
34
55
89
144
233
377
    "#,
    );
}
