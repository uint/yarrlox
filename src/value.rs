use std::{borrow::Cow, fmt::Display};

#[derive(Clone, Debug, PartialEq)]
pub enum Value<'v> {
    String(Cow<'v, str>),
    Num(f64),
    Bool(bool),
    Nil,
}

impl<'v> Value<'v> {
    pub fn string(s: impl Into<Cow<'v, str>>) -> Self {
        Self::String(s.into())
    }
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Num(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
        }
    }
}
