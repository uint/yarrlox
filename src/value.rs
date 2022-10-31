use std::fmt::Display;

use crate::callable::Callable;

#[derive(Clone, Debug)]
pub enum Value {
    String(String),
    Num(f64),
    Bool(bool),
    Nil,
    Callable(Box<dyn Callable>),
}

impl Eq for Value {}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Num(l0), Self::Num(r0)) => l0 == r0,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Callable(l0), Self::Callable(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    String,
    Num,
    Bool,
    Nil,
    Callable,
}

impl Value {
    pub fn string(s: impl Into<String>) -> Self {
        Self::String(s.into())
    }

    pub fn ty(&self) -> Type {
        match self {
            Value::String(_) => Type::String,
            Value::Num(_) => Type::Num,
            Value::Bool(_) => Type::Bool,
            Value::Nil => Type::Nil,
            Value::Callable(_) => Type::Callable,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Num(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Callable(_) => write!(f, "callable"),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::String => write!(f, "string"),
            Type::Num => write!(f, "number"),
            Type::Bool => write!(f, "bool"),
            Type::Nil => write!(f, "nil"),
            Type::Callable => write!(f, "callable"),
        }
    }
}
