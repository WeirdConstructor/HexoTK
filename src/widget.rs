use crate::{InputEvent, EventCore, Control, Painter, Rect, UINotifierRef, Event};
use crate::painter::ImgRef;
use crate::style::Style;
use std::rc::{Weak, Rc};
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct Layout {
    visible:        bool,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            visible:    true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Widget(Rc<RefCell<WidgetImpl>>);

impl Widget {
    pub fn new(style: Rc<Style>) -> Self {
        Self(Rc::new(RefCell::new(WidgetImpl::new(style))))
    }

    pub fn from_weak(w: &Weak<RefCell<WidgetImpl>>) -> Option<Widget> {
        w.upgrade().map(|w| Widget(w))
    }

    pub fn as_weak(&self) -> Weak<RefCell<WidgetImpl>> {
        Rc::downgrade(&self.0)
    }

    pub fn reg<F: 'static + FnMut(Widget, &Event)>(&self, ev_name: &str, cb: F) {
        self.0.borrow_mut().reg(ev_name, Box::new(cb));
    }

    pub fn take_event_core(&self) -> Option<EventCore> {
        self.0.borrow_mut().take_event_core()
    }

    pub fn give_back_event_core(&self, evc: EventCore) {
        self.0.borrow_mut().give_back_event_core(evc)
    }

    pub fn is_cached(&self) -> bool {
        self.0.borrow_mut().is_cached()
    }
    pub fn enable_cache(&self) {
        self.0.borrow_mut().enable_cache();
    }

    pub fn take_cache_img(&self) -> Option<ImgRef> {
        self.0.borrow_mut().take_cache_img()
    }
    pub fn give_cache_img(&self, img: ImgRef) {
        self.0.borrow_mut().give_cache_img(img);
    }

    pub fn relayout(&self, pos: Rect) -> Option<Rect> {
        self.0.borrow_mut().relayout(pos)
    }

    fn emit_layout_change(&self) {
        self.0.borrow_mut().emit_layout_change();
    }

    pub fn emit_redraw_required(&self) {
        self.0.borrow_mut().emit_redraw_required();
    }

    pub fn activate(&self) {
        self.0.borrow_mut().activate();
    }

    pub fn deactivate(&self) {
        self.0.borrow_mut().deactivate();
    }

    pub fn is_active(&self) -> bool {
        self.0.borrow_mut().is_active()
    }

    pub fn set_notifier(&self, not: UINotifierRef, id: usize) {
        self.0.borrow_mut().set_notifier(not, id)
    }

    pub fn is_hovered(&self) -> bool {
        self.0.borrow().is_hovered()
    }

    pub fn id(&self) -> usize { self.0.borrow().id() }

    pub fn check_data_change(&self) -> bool {
        self.0.borrow_mut().check_data_change()
    }

    pub fn set_ctrl(&self, ctrl: Control) {
        self.0.borrow_mut().set_ctrl(ctrl)
    }

    pub fn take_ctrl(&self) -> Option<Box<Control>> {
        self.0.borrow_mut().ctrl.take()
    }

    pub fn give_ctrl_back(&self, ctrl: Box<Control>) {
        self.0.borrow_mut().ctrl = Some(ctrl);
    }

    pub fn can_hover(&self) -> bool { self.0.borrow_mut().can_hover() }

    pub fn pos(&self) -> Rect { self.0.borrow().pos }

    pub fn inner_pos(&self) -> Rect { self.0.borrow().inner_pos }

    pub fn style(&self) -> Rc<Style> { self.0.borrow().style.clone() }

    pub fn set_style(&self, style: Rc<Style>) {
        self.0.borrow_mut().set_style(style);
    }

    pub fn parent(&self) -> Option<Widget> {
        let w = self.0.borrow();
        if let Some(parent) = &w.parent {
            Self::from_weak(parent)
        } else {
            None
        }
    }

    pub fn add(&self, child: Widget) {
        self.0.borrow_mut().add(child);
    }

    pub fn set_parent(&self, parent: &Widget) {
        self.0.borrow_mut().set_parent(parent);
    }

    pub fn clear(&self, recursive: bool) {
        self.0.borrow_mut().clear(recursive);
    }

    pub fn with_layout<R, F: FnOnce(&Layout) -> R>(&self, f: F) -> R {
        self.0.borrow().with_layout(f)
    }

    pub fn change_layout<R, F: FnOnce(&mut Layout) -> R>(&self, f: F) -> R {
        self.0.borrow_mut().change_layout(f)
    }
}

pub struct WidgetImpl {
    id:             usize,
    pub evc:        Option<EventCore>,
    parent:         Option<Weak<RefCell<WidgetImpl>>>,
    childs:         Option<Vec<Widget>>,
    pub ctrl:       Option<Box<Control>>,
    handle_childs:  Option<Vec<(Widget, Widget, bool, bool)>>,
    pos:            Rect,
    inner_pos:      Rect,
    layout:         Layout,
    layout_tmp:     Vec<Rect>,
    style:          Rc<Style>,
    notifier:       Option<UINotifierRef>,

    cached:         bool,
    cache_img:      Option<ImgRef>,
}

impl std::fmt::Debug for WidgetImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Widget").field("id", &self.id).finish()
    }
}

impl WidgetImpl {
    pub fn new(style: Rc<Style>) -> Self {
        Self {
            id:             0,
            evc:            Some(EventCore::new()),
            parent:         None,
            childs:         Some(vec![]),
            handle_childs:  Some(vec![]),
            ctrl:           Some(Box::new(Control::None)),
            pos:            Rect::from(0.0, 0.0, 0.0, 0.0),
            inner_pos:      Rect::from(0.0, 0.0, 0.0, 0.0),
            layout:         Layout::new(),
            layout_tmp:     vec![],
            notifier:       None,
            cached:         false,
            cache_img:      None,
            style,
        }
    }

    pub fn for_each_child<F: FnMut(&mut WidgetImpl, usize, bool)>(&self, mut f: F) {
        if let Some(childs) = &self.childs {
            let len = childs.len();
            for (i, w) in childs.iter().enumerate() {
                f(&mut w.0.borrow_mut(), i, (i + 1) == len)
            }
        }
    }

    pub fn reg(&mut self, ev_name: &str, cb: Box<dyn FnMut(Widget, &Event)>) {
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

    pub fn is_cached(&mut self) -> bool { self.cached }
    pub fn enable_cache(&mut self) { self.cached = true; }

    pub fn take_cache_img(&mut self) -> Option<ImgRef> { self.cache_img.take() }
    pub fn give_cache_img(&mut self, img: ImgRef) { self.cache_img = Some(img); }

    pub fn with_layout<R, F: FnOnce(&Layout) -> R>(&self, f: F) -> R {
        f(&self.layout)
    }

    pub fn change_layout<R, F: FnOnce(&mut Layout) -> R>(&mut self, f: F) -> R {
        let ret = f(&mut self.layout);
        self.emit_layout_change();
        ret
    }

    pub fn relayout(&mut self, pos: Rect) -> Option<Rect> {
        // call relayout on childs...
        Some(pos)
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

    pub fn set_ctrl(&mut self, ctrl: Control) {
        self.ctrl = Some(Box::new(ctrl));
    }

    pub fn can_hover(&self) -> bool {
        self.ctrl.as_ref().map(|c| c.can_hover()).unwrap_or(false)
    }

    pub fn pos(&self) -> Rect { self.pos }

    pub fn inner_pos(&self) -> Rect { self.inner_pos }

    pub fn style(&self) -> &Style { &*self.style }

    pub fn set_style(&mut self, style: Rc<Style>) {
        if self.style.border != style.border {
            self.emit_layout_change();
        }

        self.style = style;

        self.emit_redraw_required();
    }

    pub fn new_ref(style: Rc<Style>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new(style)))
    }

    pub fn parent(&self) -> Option<Widget> {
        self.parent.as_ref().map(|p| Widget::from_weak(p)).flatten()
    }

    pub fn add(&mut self, child: Widget) {
        if let Some(childs) = &mut self.childs {
            childs.push(child);
        }

        self.notifier.as_mut().map(|n| n.set_tree_changed());
    }

    pub fn set_parent(&mut self, parent: &Widget) {
        self.parent = Some(Rc::downgrade(&parent.0));
    }

    pub fn clear(&mut self, recursive: bool) {
        self.evc.as_mut().map(|evc| evc.clear());
        self.ctrl   = None;
        self.parent = None;

        if let Some(childs) = &mut self.childs {
            if recursive {
                for c in childs.iter_mut() {
                    c.clear(recursive);
                }
            }

            childs.clear();
        }
    }
}

pub fn widget_draw(
    widget: &Widget,
    redraw: &std::collections::HashSet<usize>,
    painter: &mut Painter)
{
    let visible  = widget.0.borrow().layout.visible;
    if !visible { return; }

    let mut ctrl = widget.0.borrow_mut().ctrl.take();
    let childs   = widget.0.borrow_mut().childs.take();
    let wid_id   = widget.0.borrow().id();

    if let Some(mut ctrl) = ctrl {
        ctrl.draw(widget, redraw.contains(&wid_id), painter);

        if let Some(childs) = childs {
            for c in childs.iter() {
                widget_draw(c, redraw, painter);
            }
            widget.0.borrow_mut().childs = Some(childs);
        }

        widget.0.borrow_mut().ctrl = Some(ctrl);
    }
}

pub fn widget_draw_frame(widget: &Widget, painter: &mut Painter) {
    let mut ctrl = widget.0.borrow_mut().ctrl.take();
    let childs   = widget.0.borrow_mut().childs.take();

    if let Some(mut ctrl) = ctrl {
        ctrl.draw_frame(widget, painter);

        if let Some(childs) = childs {
            for c in childs.iter() {
                widget_draw_frame(c, painter);
            }
            widget.0.borrow_mut().childs = Some(childs);
        }

        widget.0.borrow_mut().ctrl = Some(ctrl);
    }
}

pub fn widget_walk<F: FnMut(&Widget, Option<&Widget>, bool, bool)>(widget: &Widget, mut cb: F) {
    cb(widget, None, true, true);

    let mut hc = {
        let cur_parent = widget.clone();
        let mut w = widget.0.borrow_mut();

        if let Some(mut hc) = w.handle_childs.take() {
            hc.clear();

            if let Some(childs) = &w.childs {
                let len = childs.len();
                for (i, c) in childs.iter().enumerate() {
                    hc.push((c.clone(), cur_parent.clone(), i == 0, (i + 1) == len));
                }
            }

            Some(hc)
        } else {
            None
        }
    };

    if let Some(hc) = &mut hc {
        for (c, p, is_first, is_last) in hc.iter() {
            cb(c, Some(p), *is_first, *is_last);
        }

        hc.clear();
    }

    widget.0.borrow_mut().handle_childs = hc;
}
