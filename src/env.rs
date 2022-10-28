use std::collections::HashMap;

use crate::value::Value;

#[derive(Default)]
pub struct Env {
    values: HashMap<String, Value>,
}

impl Env {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Value {
        self.values
            .get(name)
            .map(Clone::clone)
            .unwrap_or(Value::Nil)
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), EnvError> {
        if self.values.contains_key(&name) {
            self.values.insert(name, value);
            Ok(())
        } else {
            Err(EnvError::AssignNonexistent(name))
        }
    }
}

#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone)]
pub enum EnvError {
    #[error("can't assign to nonexistent l-value {0}")]
    AssignNonexistent(String),
}
