use crate::ast::{
    Binary, BinaryOp, Expr, Grouping, Literal, NumLit, Stmt, StringLit, Unary, UnaryOp,
};
use crate::value::{Type, Value};

macro_rules! impl_arithmetic {
    ($left:tt $op:tt $right:tt) => {
        match (interpret_expr($left)?, interpret_expr($right)?) {
            (Num($left), Num($right)) => Num($left $op $right),
            (v, Num(_)) => return Err(InterpreterError::TypeError{
                expected: &[Type::Num],
                found: v.ty(),
            }),
            (_, v) => return Err(InterpreterError::TypeError{
                expected: &[Type::Num],
                found: v.ty(),
            }),
        }
    };
}

macro_rules! impl_comparison {
    ($left:tt $op:tt $right:tt) => {
        match (interpret_expr($left)?, interpret_expr($right)?) {
            (Num($left), Num($right)) => Bool($left $op $right),
            (v, Num(_)) => return Err(InterpreterError::TypeError{
                expected: &[Type::Num],
                found: v.ty(),
            }),
            (_, v) => return Err(InterpreterError::TypeError{
                expected: &[Type::Num],
                found: v.ty(),
            }),
        }
    };
}

pub fn interpret(stmts: &[Stmt]) -> Vec<InterpreterError> {
    stmts
        .iter()
        .map(|s| execute(s))
        .filter_map(Result::err)
        .collect()
}

pub fn execute(stmt: &Stmt<'_>) -> Result<(), InterpreterError> {
    match stmt {
        Stmt::Expr(expr) => {
            interpret_expr(expr)?;
            ()
        }
        Stmt::Print(expr) => print(expr)?,
    };

    Ok(())
}

fn print(expr: &Expr<'_>) -> Result<(), InterpreterError> {
    println!("{}", interpret_expr(expr)?);

    Ok(())
}

pub fn interpret_expr<'src>(expr: &Expr<'src>) -> Result<Value<'src>, InterpreterError> {
    use Value::*;

    Ok(match expr {
        Expr::Literal(l) => match l {
            Literal::StringLit(StringLit(l)) => Value::string(*l),
            Literal::NumLit(NumLit(l)) => Num(l.parse().unwrap()),
            Literal::Identifier(_) => todo!(),
            Literal::Nil => Value::Nil,
            Literal::Bool(b) => Value::Bool(*b),
        },
        Expr::Binary(Binary { left, op, right }) => match op {
            BinaryOp::Add => match (interpret_expr(left)?, interpret_expr(right)?) {
                (Num(left), Num(right)) => Num(left + right),
                (String(left), String(right)) => Value::string(format!("{}{}", left, right)),
                (Num(_), v) => {
                    return Err(InterpreterError::TypeError {
                        expected: &[Type::Num],
                        found: v.ty(),
                    })
                }
                (String(_), v) => {
                    return Err(InterpreterError::TypeError {
                        expected: &[Type::String],
                        found: v.ty(),
                    })
                }
                (v, _) => {
                    return Err(InterpreterError::TypeError {
                        expected: &[Type::Num, Type::String],
                        found: v.ty(),
                    })
                }
            },
            BinaryOp::Sub => impl_arithmetic!(left - right),
            BinaryOp::Mul => impl_arithmetic!(left * right),
            BinaryOp::Div => impl_arithmetic!(left / right),
            BinaryOp::Lt => impl_comparison!(left < right),
            BinaryOp::Lte => impl_comparison!(left <= right),
            BinaryOp::Gt => impl_comparison!(left > right),
            BinaryOp::Gte => impl_comparison!(left >= right),
            BinaryOp::Eq => Bool(is_equal(&interpret_expr(left)?, &interpret_expr(right)?)),
            BinaryOp::NotEq => Bool(!is_equal(&interpret_expr(left)?, &interpret_expr(right)?)),
        },
        Expr::Grouping(Grouping { expr }) => interpret_expr(expr)?,
        Expr::Unary(Unary { op, right }) => match op {
            UnaryOp::Not => Bool(!is_truthy(&interpret_expr(right)?)),
            UnaryOp::Negation => match interpret_expr(right)? {
                Num(n) => Num(-n),
                v => {
                    return Err(InterpreterError::TypeError {
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
pub enum InterpreterError {
    #[error("expected one of these types: {expected:?}, found: {found}")]
    TypeError {
        expected: &'static [Type],
        found: Type,
    },
}
