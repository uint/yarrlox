use std::collections::{vec_deque, HashMap, VecDeque};

use crate::value::Value;

pub struct Env {
    /// This is used as a stack. The front of the stack is the current (innermost, leaf) env.
    /// The reason to diverge from the book is to avoid trouble with mutable references
    /// and having Envs be a recursively defined type.
    scopes: VecDeque<HashMap<String, Value>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            scopes: [HashMap::new()].into(),
        }
    }

    pub fn child(&mut self) {
        self.scopes.push_front(HashMap::new());
    }

    pub fn pop(&mut self) {
        #[cfg(debug_assertions)]
        assert!(
            self.scopes.len() >= 2,
            "attempting to remove root environment"
        );
        self.scopes.pop_front();
    }

    fn root(&mut self) -> &mut HashMap<String, Value> {
        let last = self.scopes.len() - 1;
        &mut self.scopes[last]
    }

    fn current(&mut self) -> &mut HashMap<String, Value> {
        &mut self.scopes[0]
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.current().insert(name, value);
    }

    pub fn get(&self, name: &str) -> Value {
        for scope in self.scopes.iter() {
            if let Some(v) = scope.get(name) {
                return v.clone();
            }
        }

        Value::Nil
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), EnvError> {
        for scope in self.scopes.iter_mut() {
            #[allow(clippy::map_entry)]
            if scope.contains_key(&name) {
                scope.insert(name, value);
                return Ok(());
            }
        }

        Err(EnvError::AssignNonexistent(name))
    }
}

#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone)]
pub enum EnvError {
    #[error("can't assign to nonexistent l-value {0}")]
    AssignNonexistent(String),
}
