use std::{
    any::Any,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{interpreter::Interpreter, value::Value};

type Args = Vec<Value>;

pub trait Callable: std::fmt::Debug {
    fn call(&self, interpreter: &mut Interpreter, args: Args) -> Value;

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

#[derive(Debug, PartialEq, Clone)]
pub struct Clock;

impl Callable for Clock {
    fn call(&self, _interpreter: &mut Interpreter, _args: Args) -> Value {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        Value::Num(since_the_epoch.as_millis() as f64 / 1000.0)
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
