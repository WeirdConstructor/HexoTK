use crate::{EvProp, InputEvent, EventCore, Control, Painter, Rect};
use crate::style::Style;
use std::rc::{Weak, Rc};
use std::cell::RefCell;

// For morphom Cache
#[derive(Debug, Clone)]
pub struct PosInfo {
    pub pos: Rect,

    pub left:   f32,
    pub right:  f32,
    pub top:    f32,
    pub bottom: f32,
}

impl PosInfo {
    pub fn new() -> Self {
        Self {
            pos:    Rect::from(0.0, 0.0, 0.0, 0.0),
            left:   0.0,
            right:  0.0,
            top:    0.0,
            bottom: 0.0,
        }
    }
}

pub struct Widget {
    pub evc:        EventCore,
    parent:         Option<Weak<RefCell<Widget>>>,
    childs:         Option<Vec<Rc<RefCell<Widget>>>>,
    ctrl:           Option<Box<Control>>,
    handle_childs:  Option<Vec<Rc<RefCell<Widget>>>>,
    pos:            PosInfo,
    style:          Style,
}

impl Widget {
    pub fn new() -> Self {
        Self {
            evc:    EventCore::new(),
            parent: None,
            childs: Some(vec![]),
            handle_childs: Some(vec![]),
            ctrl:   None,
            pos:    PosInfo::new(),
            style:  Style::new(),
        }
    }

    pub fn set_direct_ctrl(&mut self, ctrl: Box<Control>, pos: Rect) {
        self.ctrl = Some(ctrl);
        self.pos.pos = pos;
    }

    pub fn style_mut(&mut self) -> &mut Style { &mut self.style }

    pub fn pos(&self) -> Rect { self.pos.pos }

    pub fn style(&self) -> &Style { &self.style }

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
    let mut ctrl = widget.borrow_mut().ctrl.take();
    let childs   = widget.borrow_mut().childs.take();

    if let Some(mut ctrl) = ctrl {
        ctrl.draw(widget, painter);

        if let Some(childs) = childs {
            for c in childs.iter() {
                widget_draw(c, painter);
            }
            widget.borrow_mut().childs = Some(childs);
        }

        widget.borrow_mut().ctrl = Some(ctrl);
    }
}

pub fn widget_walk<F: FnMut(&Rc<RefCell<Widget>>)>(widget: &Rc<RefCell<Widget>>, mut cb: F) {
    cb(widget);

    let mut hc = {
        let mut w = widget.borrow_mut();

        if let Some(mut hc) = w.handle_childs.take() {
            hc.clear();

            if let Some(childs) = &w.childs {
                for c in childs.iter() {
                    hc.push(c.clone());
                }
            }

            Some(hc)
        } else {
            None
        }
    };

    if let Some(hc) = &mut hc {
        for c in hc.iter() {
            cb(c);
        }

        hc.clear();
    }

    widget.borrow_mut().handle_childs = hc;
}

pub fn widget_handle(widget: &Rc<RefCell<Widget>>, event: &InputEvent) {
    let ctrl = widget.clone().borrow_mut().ctrl.take();

    if let Some(mut ctrl) = ctrl {
        let prop = ctrl.handle(widget, event);

        match prop {
            EvProp::Childs => {
                let mut hc = {
                    let mut w = widget.borrow_mut();
                    if let Some(mut hc) = w.handle_childs.take() {

                        hc.clear();

                        if let Some(childs) = &w.childs {
                            for c in childs.iter() {
                                hc.push(c.clone());
                            }
                        }

                        Some(hc)
                    } else {
                        None
                    }
                };

                if let Some(childs) = &mut hc {
                    for c in childs.iter() {
                        widget_handle(c, event);
                    }

                    childs.clear();
                }

                widget.borrow_mut().handle_childs = hc;
            },
            EvProp::Stop => {},
        }

        widget.borrow_mut().ctrl = Some(ctrl);
    }
}