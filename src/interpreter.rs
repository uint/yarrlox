use std::cell::RefCell;
use std::io::{stdout, Stdout, Write};
use std::rc::Rc;

use crate::callable::Clock;
use crate::env::{Env, EnvError};
use crate::resolver::Resolver;
use crate::value::{Type, Value};
use crate::{ast::*, ResolverError};

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
    globals: Rc<RefCell<Env>>,
    env: Rc<RefCell<Env>>,
    out: InterpreterOutput,
    resolver: Resolver,
}

pub enum ExecResult {
    Nothing,
    LoopUnwind,
}

fn make_global_env() -> Rc<RefCell<Env>> {
    let env = Env::new();

    env.borrow_mut()
        .define("clock", Value::Callable(Rc::new(Clock)));

    env
}

impl Default for Interpreter {
    fn default() -> Self {
        let env = make_global_env();

        Self {
            globals: Rc::clone(&env),
            env,
            out: InterpreterOutput::Stdout(stdout()),
            resolver: Resolver::new(),
        }
    }
}

impl Interpreter {
    pub fn new(out: InterpreterOutput) -> Self {
        let env = make_global_env();

        Self {
            globals: Rc::clone(&env),
            env,
            out,
            resolver: Resolver::new(),
        }
    }

    pub fn interpret(
        &mut self,
        stmts: &[Stmt],
        var_count: usize,
    ) -> Result<Value, Vec<InterpreterError>> {
        self.resolver
            .resolve(stmts, var_count)
            .map_err(|err| vec![err.into()])?;

        let errs = stmts
            .iter()
            .map(|s| self.execute(s))
            .filter_map(Result::err);

        let (returns, errs): (Vec<_>, Vec<_>) =
            errs.partition(|err| matches!(err, InterpreterError::FunReturn(_)));

        if !errs.is_empty() {
            Err(errs)
        } else {
            Ok(returns
                .get(0)
                .map(InterpreterError::ret)
                .unwrap_or(Value::Nil))
        }
    }

    pub fn get_output(&mut self) -> String {
        match &mut self.out {
            InterpreterOutput::Stdout(_) => String::new(),
            InterpreterOutput::String(s) => {
                let bytes = std::mem::take(s);
                String::from_utf8(bytes).unwrap()
            }
        }
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<(), InterpreterError> {
        match stmt {
            Stmt::Expr(expr) => {
                self.interpret_expr(expr)?;
            }
            Stmt::Print(expr) => self.print(expr)?,
            Stmt::Return(expr) => self.ret(expr.into())?,
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(init) => self.interpret_expr(init)?,
                    None => Value::Nil,
                };
                self.env.borrow_mut().define(name.to_string(), value)
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
                    match self.execute(body) {
                        Err(InterpreterError::LoopUnwind) => break,
                        err @ Err(_) => return err,
                        Ok(_) => (),
                    }
                }
            }
            Stmt::Break => return Err(InterpreterError::LoopUnwind),
            Stmt::Function(fun) => self.declare_fun(fun),
        };

        Ok(())
    }

    fn declare_fun(&mut self, fun: &Function) {
        let fun_env = Rc::clone(&self.env);

        self.env.borrow_mut().define(
            fun.name.clone(),
            Value::Callable(Rc::new(crate::callable::Function::new(
                fun.clone(),
                fun_env,
            ))),
        );
    }

    pub fn execute_fun_call(
        &mut self,
        stmts: &[Stmt],
        params: &[String],
        closure: Rc<RefCell<Env>>,
        args: Vec<Value>,
    ) -> Result<Value, InterpreterError> {
        let new_env = Env::child(&closure);
        let prev_env = std::mem::replace(&mut self.env, new_env);

        for (param, arg) in params.iter().zip(args) {
            self.env.borrow_mut().define(param.clone(), arg);
        }

        let val = match self.execute_block(stmts) {
            Ok(()) => Value::Nil,
            Err(InterpreterError::FunReturn(v)) => v,
            Err(err) => return Err(err),
        };

        self.env = prev_env;

        Ok(val)
    }

    fn execute_block(&mut self, stmts: &[Stmt]) -> Result<(), InterpreterError> {
        let new_env = Env::child(&self.env);
        let prev_env = std::mem::replace(&mut self.env, new_env);

        for stmt in stmts {
            self.execute(stmt)?;
        }

        self.env = prev_env;

        Ok(())
    }

    fn print(&mut self, expr: &Expr) -> Result<(), InterpreterError> {
        let v = self.interpret_expr(expr)?;

        writeln!(&mut self.out, "{}", v).unwrap();

        Ok(())
    }

    fn ret(&mut self, expr: Option<&Expr>) -> Result<(), InterpreterError> {
        let val = expr
            .map(|expr| self.interpret_expr(expr))
            .unwrap_or(Ok(Value::Nil))?;

        Err(InterpreterError::FunReturn(val))
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
                name: Reference { ident, .. },
                value,
            }) => {
                let value = self.interpret_expr(value)?;
                self.env
                    .borrow_mut()
                    .assign(ident.to_string(), value.clone())?;
                value
            }
            Expr::Literal(l) => match l {
                Literal::StringLit(StringLit(l)) => Value::string(l),
                Literal::NumLit(NumLit(l)) => Num(l.parse().unwrap()),
                Literal::Identifier(reference) => self.look_up_variable(reference),
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
            Expr::Call(c) => self.interpret_call(c)?,
        })
    }

    fn look_up_variable(&self, Reference { id, ident }: &Reference) -> Value {
        if let Some(Some(distance)) = self.resolver.locals.get(*id) {
            self.env.borrow().get_at(*distance, ident)
        } else {
            self.globals.borrow().get(ident)
        }
    }

    fn interpret_call(
        &mut self,
        Call { callee, args, .. }: &Call,
    ) -> Result<Value, InterpreterError> {
        let callee = self.interpret_expr(callee)?;

        let args: Vec<_> = args
            .iter()
            .map(|arg| self.interpret_expr(arg))
            .collect::<Result<_, _>>()?;

        if let Value::Callable(callable) = callee {
            if args.len() == callable.arity() as usize {
                Ok(callable.call(self, args)?)
            } else {
                Err(InterpreterError::ArityMismatch {
                    expected: callable.arity(),
                    got: args.len(),
                })
            }
        } else {
            Err(InterpreterError::NotCallable)
        }
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

pub enum InterpreterOutput {
    Stdout(Stdout),
    String(Vec<u8>),
}

impl Write for InterpreterOutput {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            InterpreterOutput::Stdout(stdout) => stdout.write(buf),
            InterpreterOutput::String(bufw) => bufw.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            InterpreterOutput::Stdout(stdout) => stdout.flush(),
            InterpreterOutput::String(bufw) => bufw.flush(),
        }
    }
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
    #[error("unwinding loop")]
    LoopUnwind,
    #[error("returning from function")]
    FunReturn(Value),
    #[error("not callable")]
    NotCallable,
    #[error("function expected {expected} arguments, but received {got}")]
    ArityMismatch { expected: u8, got: usize },
    #[error("{0}")]
    Resolution(#[from] ResolverError),
}

impl InterpreterError {
    fn ret(&self) -> Value {
        if let Self::FunReturn(v) = self {
            v.clone()
        } else {
            panic!("should never happen! I'm serious! gosh!");
        }
    }
}
