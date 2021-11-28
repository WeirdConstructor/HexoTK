use crate::{InputEvent, EventCore, Control, Painter, Rect, UINotifierRef, Event};
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
    id:             usize,
    pub evc:        Option<EventCore>,
    parent:         Option<Weak<RefCell<Widget>>>,
    childs:         Option<Vec<Rc<RefCell<Widget>>>>,
    pub ctrl:       Option<Box<Control>>,
    handle_childs:  Option<Vec<(Rc<RefCell<Widget>>, Rc<RefCell<Widget>>)>>,
    pos:            PosInfo,
    style:          Rc<Style>,
    notifier:       Option<UINotifierRef>,
}

impl Widget {
    pub fn new(style: Rc<Style>) -> Self {
        Self {
            id:         0,
            evc:        Some(EventCore::new()),
            parent:     None,
            childs:     Some(vec![]),
            handle_childs: Some(vec![]),
            ctrl:       None,
            pos:        PosInfo::new(),
            notifier:   None,
            style,
        }
    }

    pub fn reg(&mut self, ev_name: &str, cb: Box<dyn Fn(Rc<RefCell<Widget>>, &Event)>) {
        if let Some(evc) = &mut self.evc {
            evc.reg(ev_name, cb);
        }
    }

    pub fn take_event_core(&mut self) -> Option<EventCore> {
        self.evc.take()
    }

    pub fn give_back_event_core(&mut self, evc: EventCore) {
        self.evc = Some(evc);
    }

    fn emit_layout_change(&self) {
        self.notifier.as_ref().map(|n| n.set_layout_changed());
    }

    pub fn emit_redraw_required(&self) {
        self.notifier.as_ref().map(|n| n.redraw(self.id));
    }

    pub fn activate(&self) {
        self.notifier.as_ref().map(|n| n.activate(self.id));
    }

    pub fn deactivate(&self) {
        self.notifier.as_ref().map(|n| n.deactivate(self.id));
    }

    pub fn is_active(&self) -> bool {
        self.id == self.notifier.as_ref().map(|n| n.active()).flatten().unwrap_or(0)
    }

    pub fn set_notifier(&mut self, not: UINotifierRef, id: usize) {
        self.notifier = Some(not);
        self.id = id;
    }

    pub fn is_hovered(&self) -> bool {
        self.id == self.notifier.as_ref().map(|n| n.hover()).unwrap_or(0)
    }

    pub fn id(&self) -> usize { self.id }

    pub fn check_data_change(&mut self) -> bool {
        if let Some(ctrl) = &mut self.ctrl {
            ctrl.check_change()
        } else {
            false
        }
    }

    pub fn set_direct_ctrl(&mut self, ctrl: Control, pos: Rect) {
        self.ctrl = Some(Box::new(ctrl));
        self.pos.pos = pos;
    }

    pub fn pos(&self) -> Rect { self.pos.pos }

    pub fn style(&self) -> &Style { &*self.style }

    pub fn set_style(&mut self, style: Rc<Style>) {
        self.style = style;
        self.emit_redraw_required();
    }

    pub fn new_ref(style: Rc<Style>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new(style)))
    }

    pub fn parent(&mut self) -> Option<Rc<RefCell<Widget>>> {
        if let Some(parent) = &self.parent {
            parent.upgrade()
        } else {
            None
        }
    }

    pub fn add(&mut self, child: Rc<RefCell<Widget>>) {
        if let Some(childs) = &mut self.childs {
            childs.push(child);
        }

        self.notifier.as_mut().map(|n| n.set_tree_changed());
    }

    pub fn set_parent(&mut self, parent: &Rc<RefCell<Widget>>) {
        self.parent = Some(Rc::downgrade(parent));
    }

    pub fn clear(&mut self, recursive: bool) {
        self.evc.as_mut().map(|evc| evc.clear());
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

pub fn widget_walk<F: FnMut(&Rc<RefCell<Widget>>, Option<&Rc<RefCell<Widget>>>)>(widget: &Rc<RefCell<Widget>>, mut cb: F) {
    cb(widget, None);

    let mut hc = {
        let cur_parent = widget.clone();
        let mut w = widget.borrow_mut();

        if let Some(mut hc) = w.handle_childs.take() {
            hc.clear();

            if let Some(childs) = &w.childs {
                for c in childs.iter() {
                    hc.push((c.clone(), cur_parent.clone()));
                }
            }

            Some(hc)
        } else {
            None
        }
    };

    if let Some(hc) = &mut hc {
        for (c, p) in hc.iter() {
            cb(c, Some(p));
        }

        hc.clear();
    }

    widget.borrow_mut().handle_childs = hc;
}
