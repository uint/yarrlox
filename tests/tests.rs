mod runner;

use runner::run;

use yarrlox::value::Value;

// These aren't meant to be complete branch coverage tests or anything like that. These
// are smoke tests for key functionality. There's the official Lox test suite already.
// Once the interpreter is finished, I'll run it against that.

#[test]
fn var_definition() {
    let closure = r#"
var x = 5;
return x;
"#;

    run(closure).assert_v(Value::Num(5.));
}

#[test]
fn undefined_var_is_nil() {
    let closure = r#"
return x;
"#;

    run(closure).assert_v(Value::Nil);
}

#[test]
fn var_out_of_scope() {
    let closure = r#"
{
  var x = 5;
}
return x;
"#;

    run(closure).assert_v(Value::Nil);
}

#[test]
fn shadowing() {
    let closure = r#"
var x = 5;

{
  var x = 10;
  print x;
}

return x;
"#;

    let run = run(closure);
    run.assert_output("10");
    run.assert_v(Value::Num(5.));
}

#[test]
fn mutation_in_a_scope() {
    let closure = r#"
var x = 5;

{
  x = 10;
  print x;
}

return x;
"#;

    let run = run(closure);
    run.assert_output("10");
    run.assert_v(Value::Num(10.));
}

#[test]
fn string_concat() {
    let closure = r#"
return "foo" + "bar";
"#;

    run(closure).assert_v(Value::string("foobar"));
}

#[test]
fn fun_def() {
    let closure = r#"
fun pow(x) {
  return x * x;
}

return pow(5);
"#;

    run(closure).assert_v(Value::Num(25.));
}

#[test]
fn partial_application() {
    let closure = r#"
fun adder(x) {
  fun result(a) {
    return x + a;
  }

  return result;
}

var my_adder = adder(3);
return my_adder(5);
"#;

    run(closure).assert_v(Value::Num(8.));
}

#[test]
fn fun_with_fun_as_arg() {
    let closure = r#"
fun inc(x) {
  return x + 1;
}

fun applier(fn, el) {
  return fn(el);
}

return applier(inc, 5);
"#;

    run(closure).assert_v(Value::Num(6.));
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
