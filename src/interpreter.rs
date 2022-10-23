use crate::ast::{Binary, BinaryOp, Expr, Grouping, Literal, NumLit, StringLit, Unary, UnaryOp};
use crate::value::{Type, Value};

macro_rules! impl_arithmetic {
    ($left:tt $op:tt $right:tt) => {
        match (interpret($left)?, interpret($right)?) {
            (Num($left), Num($right)) => Num($left $op $right),
            _ => panic!("oh noes"),
        }
    };
}

macro_rules! impl_comparison {
    ($left:tt $op:tt $right:tt) => {
        match (interpret($left)?, interpret($right)?) {
            (Num($left), Num($right)) => Bool($left $op $right),
            _ => panic!("oh noes"),
        }
    };
}

pub fn interpret<'src>(expr: &Expr<'src>) -> Result<Value<'src>, InterpretError> {
    use Value::*;

    Ok(match expr {
        Expr::Literal(l) => match l {
            Literal::StringLit(StringLit(l)) => Value::string(*l),
            Literal::NumLit(NumLit(l)) => Num(l.parse().unwrap()),
            Literal::Identifier(_) => todo!(),
        },
        Expr::Binary(Binary { left, op, right }) => match op {
            BinaryOp::Add => match (interpret(left)?, interpret(right)?) {
                (Num(left), Num(right)) => Num(left + right),
                (String(left), String(right)) => Value::string(format!("{}{}", left, right)),
                _ => panic!("oh noes"),
            },
            BinaryOp::Sub => impl_arithmetic!(left - right),
            BinaryOp::Mul => impl_arithmetic!(left * right),
            BinaryOp::Div => impl_arithmetic!(left / right),
            BinaryOp::Lt => impl_comparison!(left < right),
            BinaryOp::Lte => impl_comparison!(left <= right),
            BinaryOp::Gt => impl_comparison!(left > right),
            BinaryOp::Gte => impl_comparison!(left >= right),
            BinaryOp::Eq => Bool(is_equal(&interpret(left)?, &interpret(right)?)),
            BinaryOp::NotEq => Bool(!is_equal(&interpret(left)?, &interpret(right)?)),
        },
        Expr::Grouping(Grouping { expr }) => interpret(expr)?,
        Expr::Unary(Unary { op, right }) => match op {
            UnaryOp::Not => Bool(!is_truthy(&interpret(right)?)),
            UnaryOp::Negation => match interpret(right)? {
                Num(n) => Num(-n),
                v => {
                    return Err(InterpretError::TypeError {
                        expected: &[Type::Num],
                        found: v.ty(),
                    })
                }
            },
        },
    })
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

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum InterpretError {
    #[error("expected one of these types: {expected:?}, found: {found}")]
    TypeError {
        expected: &'static [Type],
        found: Type,
    },
}
