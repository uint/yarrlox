use std::collections::{HashMap, VecDeque};

use crate::value::Value;

pub struct Env {
    globals: HashMap<String, Value>,
    /// This is used as a series of stacks. The front of the stack is the current
    /// (innermost, leaf) env.
    /// The reason to diverge from the book is to avoid trouble with mutable references
    /// and having Envs be a recursively defined type.
    branches: VecDeque<VecDeque<HashMap<String, Value>>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            branches: [[].into()].into(),
        }
    }

    pub fn branch(&mut self) {
        self.branches.push_front([HashMap::new()].into());
    }

    pub fn pop_branch(&mut self) {
        self.branches.pop_front();
    }

    pub fn child(&mut self) {
        self.branches[0].push_front(HashMap::new());
    }

    pub fn pop(&mut self) {
        self.branches[0].pop_front().expect("nothing to pop");
    }

    fn current(&mut self) -> &mut HashMap<String, Value> {
        if let Some(scope) = self.branches[0].get_mut(0) {
            scope
        } else {
            &mut self.globals
        }
    }

    pub fn define(&mut self, name: impl Into<String>, value: Value) {
        self.current().insert(name.into(), value);
    }

    fn walk_env(&self) -> impl Iterator<Item = &HashMap<String, Value>> {
        self.branches[0]
            .iter()
            .chain(std::iter::once(&self.globals))
    }

    fn walk_env_mut(&mut self) -> impl Iterator<Item = &mut HashMap<String, Value>> {
        self.branches[0]
            .iter_mut()
            .chain(std::iter::once(&mut self.globals))
    }

    pub fn get(&self, name: &str) -> Value {
        for scope in self.walk_env() {
            if let Some(v) = scope.get(name) {
                return v.clone();
            }
        }

        Value::Nil
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), EnvError> {
        for scope in self.walk_env_mut() {
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
