use crate::ast::{
    Binary, BinaryOp, Expr, Grouping, Identifier, Literal, NumLit, StringLit, Unary, UnaryOp,
};
use crate::value::Value;

pub fn interpret<'src>(expr: &Expr<'src>) -> Value<'src> {
    match expr {
        Expr::Literal(l) => match l {
            Literal::StringLit(StringLit(l)) => Value::String(l),
            Literal::NumLit(NumLit(l)) => Value::Num(l.parse().unwrap()),
            Literal::Identifier(i) => todo!(),
        },
        Expr::Binary(Binary { left, op, right }) => match op {
            BinaryOp::Add => todo!(),
            BinaryOp::Sub => todo!(),
            BinaryOp::Mul => todo!(),
            BinaryOp::Div => todo!(),
            BinaryOp::Lt => todo!(),
            BinaryOp::Lte => todo!(),
            BinaryOp::Gt => todo!(),
            BinaryOp::Gte => todo!(),
            BinaryOp::Eq => todo!(),
            BinaryOp::NotEq => todo!(),
        },
        Expr::Grouping(Grouping { expr }) => interpret(expr),
        Expr::Unary(Unary { op, right }) => match op {
            UnaryOp::Not => Value::Bool(!is_truthy(&interpret(right))),
            UnaryOp::Negation => match interpret(right) {
                Value::Num(n) => Value::Num(-n),
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
