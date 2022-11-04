mod runner;

use runner::run;

use yarrlox::value::Value;

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
