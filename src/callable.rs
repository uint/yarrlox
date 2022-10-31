use std::{
    any::Any,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::ast::{self, Expr};
use crate::{
    interpreter::{Interpreter, InterpreterError},
    value::Value,
};

type Args = Vec<Value>;

pub trait Callable: std::fmt::Debug {
    fn call(&self, interpreter: &mut Interpreter, args: Args) -> Result<Value, InterpreterError>;

    fn arity(&self) -> u8;

    fn boxed_clone(&self) -> Box<dyn Callable>;

    fn as_any(&self) -> &dyn Any;

    fn equals_callable(&self, other: &dyn Callable) -> bool;
}

impl Clone for Box<dyn Callable> {
    fn clone(&self) -> Self {
        self.boxed_clone()
    }
}

impl PartialEq for &dyn Callable {
    fn eq(&self, other: &Self) -> bool {
        self.equals_callable(*other)
    }
}

impl PartialEq for Box<dyn Callable> {
    fn eq(&self, other: &Self) -> bool {
        self.equals_callable(other.as_ref())
    }
}

// -- User-defined --

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    decl: ast::Function,
}

impl Function {
    pub fn new(decl: ast::Function) -> Self {
        Self { decl }
    }
}

impl Callable for Function {
    fn call(&self, interpreter: &mut Interpreter, args: Args) -> Result<Value, InterpreterError> {
        interpreter.execute_fun_call(&self.decl.body, &self.decl.params, args)
    }

    fn arity(&self) -> u8 {
        self.decl.params.len() as u8
    }

    fn boxed_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn equals_callable(&self, other: &dyn Callable) -> bool {
        // TODO: registered functions should probably end up in a table in `Interpreter`
        // with unique indexes, and we would then just compare those indexes
        other
            .as_any()
            .downcast_ref::<Function>()
            .map_or(false, |a| self == a)
    }
}

// -- Built-ins --

#[derive(Debug, PartialEq, Clone)]
pub struct Clock;

impl Callable for Clock {
    fn call(&self, _interpreter: &mut Interpreter, _args: Args) -> Result<Value, InterpreterError> {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        Ok(Value::Num(since_the_epoch.as_millis() as f64 / 1000.0))
    }

    fn arity(&self) -> u8 {
        0
    }

    fn boxed_clone(&self) -> Box<dyn Callable> {
        Box::new(Self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn equals_callable(&self, other: &dyn Callable) -> bool {
        other
            .as_any()
            .downcast_ref::<Clock>()
            .map_or(false, |a| self == a)
    }
}
