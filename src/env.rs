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
}
