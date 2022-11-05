use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::value::Value;

#[derive(Debug, PartialEq)]
pub struct Env {
    up: Option<Rc<RefCell<Self>>>,
    names: HashMap<String, Value>,
}

impl Env {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            up: None,
            names: HashMap::new(),
        }))
    }

    pub fn child(this: &Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            up: Some(Rc::clone(this)),
            names: HashMap::new(),
        }))
    }

    pub fn define(&mut self, name: impl Into<String>, value: Value) {
        self.names.insert(name.into(), value);
    }

    pub fn get(&self, name: &str) -> Value {
        if let Some(v) = self.names.get(name) {
            return v.clone();
        }

        let mut scope = self.up.as_ref().map(Rc::clone);

        while let Some(s) = scope {
            if let Some(v) = s.borrow().names.get(name) {
                return v.clone();
            }
            scope = s.borrow().up.as_ref().map(Rc::clone);
        }

        Value::Nil
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), EnvError> {
        #[allow(clippy::map_entry)] // applying this lint would mean an extra string copy
        if self.names.contains_key(&name) {
            self.names.insert(name, value);
            return Ok(());
        }

        let mut scope = self.up.as_ref().map(|up| Rc::clone(up));

        while let Some(s) = scope {
            if s.borrow().names.contains_key(&name) {
                s.borrow_mut().names.insert(name, value);
                return Ok(());
            }
            scope = s.borrow().up.as_ref().map(Rc::clone);
        }

        Err(EnvError::AssignNonexistent(name))
    }
}

#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone)]
pub enum EnvError {
    #[error("can't assign to nonexistent l-value {0}")]
    AssignNonexistent(String),
}
