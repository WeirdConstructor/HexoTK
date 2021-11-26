
use std::rc::{Weak, Rc};
use std::cell::RefCell;

pub struct Widget {
    parent: Option<Weak<RefCell<Widget>>>,
    childs: Vec<Rc<RefCell<Widget>>>,
}

impl Widget {
    pub fn new() -> Self {
        Self {
            parent: None,
            childs: vec![],
        }
    }

    pub fn new_ref() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new()))
    }
}
