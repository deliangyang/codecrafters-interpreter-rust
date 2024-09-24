use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::objects::Object;

#[derive(PartialEq, Clone, Debug)]
pub struct Env {
    store: HashMap<String, Object>,
    outer: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            store: HashMap::new(),
            outer: None,
        }
    }

    pub fn from(store: HashMap<String, Object>) -> Self {
        Env { store, outer: None }
    }

    pub fn new_with_outer(outer: Rc<RefCell<Env>>) -> Self {
        Env {
            store: HashMap::new(),
            outer: Some(outer),
        }
    }

    pub fn get(&mut self, name: String) -> Option<Object> {
        match self.store.get(&name) {
            Some(value) => Some(value.clone()),
            None => match self.outer {
                Some(ref outer) => outer.borrow_mut().get(name),
                None => None,
            },
        }
    }

    pub fn set(&mut self, name: String, value: &Object) {
        match self.outer {
            Some(ref outer) => {
                // println!("outer: {:?} {:?}", name, outer.borrow().store.contains_key(&name));
                if outer.borrow().store.contains_key(&name) {
                    outer.borrow_mut().store.insert(name, value.clone());
                } else {
                    self.store.insert(name, value.clone());
                }
            }
            None => {
                self.store.insert(name, value.clone());
            }
        }
        //self.store.insert(name, value.clone());
    }

    pub fn set_store(&mut self, name: String, value: &Object) {
        self.store.insert(name, value.clone());
    }
}
