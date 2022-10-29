use crate::ast::{
    Assign, Binary, BinaryOp, Expr, Grouping, Identifier, Literal, NumLit, Stmt, StringLit, Unary,
    UnaryOp,
};
use crate::env::{Env, EnvError};
use crate::value::{Type, Value};

macro_rules! impl_arithmetic {
    ($self:tt $left:tt $op:tt $right:tt) => {
        match ($self.interpret_expr($left)?, $self.interpret_expr($right)?) {
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
    ($self:tt $left:tt $op:tt $right:tt) => {
        match ($self.interpret_expr($left)?, $self.interpret_expr($right)?) {
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

pub struct Interpreter {
    env: Env,
}

impl<'v> Interpreter {
    pub fn new() -> Self {
        Self { env: Env::new() }
    }

    pub fn interpret(&mut self, stmts: &[Stmt]) -> Vec<InterpreterError> {
        stmts
            .iter()
            .map(|s| self.execute(s))
            .filter_map(Result::err)
            .collect()
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<(), InterpreterError> {
        match stmt {
            Stmt::Expr(expr) => {
                self.interpret_expr(expr)?;
            }
            Stmt::Print(expr) => self.print(expr)?,
            Stmt::Var {
                name: Identifier(name),
                initializer,
            } => {
                let value = match initializer {
                    Some(init) => self.interpret_expr(init)?,
                    None => Value::Nil,
                };
                self.env.define(name.to_string(), value)
            }
            Stmt::Block(stmts) => self.execute_block(stmts)?,
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if is_truthy(&self.interpret_expr(condition)?) {
                    self.execute(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)?;
                }
            }
            Stmt::While { condition, body } => {
                while is_truthy(&self.interpret_expr(condition)?) {
                    self.execute(body)?;
                }
            }
        };

        Ok(())
    }

    fn execute_block(&mut self, stmts: &[Stmt]) -> Result<(), InterpreterError> {
        self.env.child();

        for stmt in stmts {
            self.execute(stmt)?;
        }

        self.env.pop();

        Ok(())
    }

    fn print(&mut self, expr: &Expr<'v>) -> Result<(), InterpreterError> {
        println!("{}", self.interpret_expr(expr)?);

        Ok(())
    }

    fn eval_logic(
        &mut self,
        is_or: bool,
        left: &Expr,
        right: &Expr,
    ) -> Result<Value, InterpreterError> {
        let left = self.interpret_expr(left)?;

        match (is_or, is_truthy(&left)) {
            (false, false) | (true, true) => Ok(left),
            _ => self.interpret_expr(right),
        }
    }

    pub fn interpret_expr(&mut self, expr: &Expr) -> Result<Value, InterpreterError> {
        use Value::*;

        Ok(match expr {
            Expr::Assign(Assign {
                name: Identifier(ident),
                value,
            }) => {
                let value = self.interpret_expr(value)?;
                self.env.assign(ident.to_string(), value.clone())?;
                value
            }
            Expr::Literal(l) => match l {
                Literal::StringLit(StringLit(l)) => Value::string(*l),
                Literal::NumLit(NumLit(l)) => Num(l.parse().unwrap()),
                Literal::Identifier(Identifier(ident)) => self.env.get(ident),
                Literal::Nil => Value::Nil,
                Literal::Bool(b) => Value::Bool(*b),
            },
            Expr::Binary(Binary { left, op, right }) => match op {
                BinaryOp::LogicOr => self.eval_logic(true, left, right)?,
                BinaryOp::LogicAnd => self.eval_logic(false, left, right)?,
                BinaryOp::Add => match (self.interpret_expr(left)?, self.interpret_expr(right)?) {
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
                BinaryOp::Sub => impl_arithmetic!(self  left - right),
                BinaryOp::Mul => impl_arithmetic!(self  left * right),
                BinaryOp::Div => impl_arithmetic!(self  left / right),
                BinaryOp::Lt => impl_comparison!(self  left < right),
                BinaryOp::Lte => impl_comparison!(self  left <= right),
                BinaryOp::Gt => impl_comparison!(self  left > right),
                BinaryOp::Gte => impl_comparison!(self  left >= right),
                BinaryOp::Eq => Bool(is_equal(
                    &self.interpret_expr(left)?,
                    &self.interpret_expr(right)?,
                )),
                BinaryOp::NotEq => Bool(!is_equal(
                    &self.interpret_expr(left)?,
                    &self.interpret_expr(right)?,
                )),
            },
            Expr::Grouping(Grouping { expr }) => self.interpret_expr(expr)?,
            Expr::Unary(Unary { op, right }) => match op {
                UnaryOp::Not => Bool(!is_truthy(&self.interpret_expr(right)?)),
                UnaryOp::Negation => match self.interpret_expr(right)? {
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
    #[error("{0}")]
    EnvError(#[from] EnvError),
}
