use crate::{EvProp, InputEvent, EventCore, Control, Painter};
use std::rc::{Weak, Rc};
use std::cell::RefCell;

pub struct Widget {
    pub evc:    EventCore,
    parent:     Option<Weak<RefCell<Widget>>>,
    childs:     Option<Vec<Rc<RefCell<Widget>>>>,
    ctrl:       Option<Box<Control>>,
}

impl Widget {
    pub fn new() -> Self {
        Self {
            evc:    EventCore::new(),
            parent: None,
            childs: Some(vec![]),
            ctrl:   None,
        }
    }

    pub fn new_ref() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new()))
    }

    pub fn parent(&mut self) -> Option<Rc<RefCell<Widget>>> {
        if let Some(parent) = &self.parent {
            parent.upgrade()
        } else {
            None
        }
    }

    pub fn set_parent(&mut self, parent: Rc<RefCell<Widget>>) {
        self.parent = Some(Rc::downgrade(&parent));
    }

    pub fn clear(&mut self, recursive: bool) {
        self.evc.clear();
        self.ctrl   = None;
        self.parent = None;

        if let Some(childs) = &mut self.childs {
            if recursive {
                for c in childs.iter_mut() {
                    c.borrow_mut().clear(recursive);
                }
            }

            childs.clear();
        }
    }
}

pub fn widget_draw(widget: &Rc<RefCell<Widget>>, painter: &mut Painter) {
    if let Some(mut ctrl) = widget.borrow_mut().ctrl.take() {
        ctrl.draw(widget, painter);

        if let Some(childs) = widget.borrow_mut().childs.take() {
            for c in childs.iter() {
                widget_draw(c, painter);
            }
            widget.borrow_mut().childs = Some(childs);
        }

        widget.borrow_mut().ctrl = Some(ctrl);
    }
}

pub fn widget_handle(widget: &Rc<RefCell<Widget>>, event: &InputEvent) {
    let ctrl = widget.clone().borrow_mut().ctrl.take();

    if let Some(mut ctrl) = ctrl {
        let prop = ctrl.handle(widget, event);

        match prop {
            EvProp::Childs => {
                let childs = widget.borrow_mut().childs.take();

                if let Some(childs) = &childs {
                    for c in childs.iter() {
                        widget_handle(c, event);
                    }
                }

                widget.borrow_mut().childs = childs;
            },
            EvProp::Stop => {},
        }

        widget.borrow_mut().ctrl = Some(ctrl);
    }
}
