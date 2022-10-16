use crate::ast::{
    Binary, BinaryOp, Expr, Grouping, Identifier, Literal, NumLit, StringLit, Unary, UnaryOp,
};
use crate::value::Value;

macro_rules! impl_arithmetic {
    ($left:tt $op:tt $right:tt) => {
        match (interpret($left), interpret($right)) {
            (Num($left), Num($right)) => Num($left $op $right),
            _ => panic!("oh noes"),
        }
    };
}

macro_rules! impl_comparison {
    ($left:tt $op:tt $right:tt) => {
        match (interpret($left), interpret($right)) {
            (Num($left), Num($right)) => Bool($left $op $right),
            _ => panic!("oh noes"),
        }
    };
}

pub fn interpret<'src>(expr: &Expr<'src>) -> Value<'src> {
    use Value::*;

    match expr {
        Expr::Literal(l) => match l {
            Literal::StringLit(StringLit(l)) => String(l),
            Literal::NumLit(NumLit(l)) => Num(l.parse().unwrap()),
            Literal::Identifier(i) => todo!(),
        },
        Expr::Binary(Binary { left, op, right }) => match op {
            BinaryOp::Add => impl_arithmetic!(left + right),
            BinaryOp::Sub => impl_arithmetic!(left - right),
            BinaryOp::Mul => impl_arithmetic!(left * right),
            BinaryOp::Div => impl_arithmetic!(left / right),
            BinaryOp::Lt => impl_comparison!(left < right),
            BinaryOp::Lte => impl_comparison!(left <= right),
            BinaryOp::Gt => impl_comparison!(left > right),
            BinaryOp::Gte => impl_comparison!(left >= right),
            BinaryOp::Eq => Bool(is_equal(&interpret(left), &interpret(right))),
            BinaryOp::NotEq => Bool(!is_equal(&interpret(left), &interpret(right))),
        },
        Expr::Grouping(Grouping { expr }) => interpret(expr),
        Expr::Unary(Unary { op, right }) => match op {
            UnaryOp::Not => Bool(!is_truthy(&interpret(right))),
            UnaryOp::Negation => match interpret(right) {
                Num(n) => Num(-n),
                _ => panic!("nope"),
            },
        },
    }
}

fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Bool(b) => *b,
        Value::Nil => false,
        _ => true,
    }
}

fn is_equal(left: &Value, right: &Value) -> bool {
    left == right
}
