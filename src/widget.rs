use crate::{EventCore, Control, Painter, Rect, UINotifierRef, Event};
use crate::painter::ImgRef;
use crate::style::Style;
use std::rc::{Weak, Rc};
use std::cell::RefCell;

use morphorm::{LayoutType, PositionType, Units};

#[derive(Debug, Clone)]
pub struct Layout {
    pub visible:        bool,

    pub layout_type:   Option<LayoutType>,
    pub position_type: Option<PositionType>,
    pub width:         Option<Units>,
    pub height:        Option<Units>,
    pub min_width:     Option<Units>,
    pub min_height:    Option<Units>,
    pub max_width:     Option<Units>,
    pub max_height:    Option<Units>,
    pub left:          Option<Units>,
    pub right:         Option<Units>,
    pub top:           Option<Units>,
    pub bottom:        Option<Units>,
    pub min_left:      Option<Units>,
    pub max_left:      Option<Units>,
    pub min_right:     Option<Units>,
    pub max_right:     Option<Units>,
    pub min_top:       Option<Units>,
    pub max_top:       Option<Units>,
    pub min_bottom:    Option<Units>,
    pub max_bottom:    Option<Units>,
    pub child_left:    Option<Units>,
    pub child_right:   Option<Units>,
    pub child_top:     Option<Units>,
    pub child_bottom:  Option<Units>,
    pub row_between:   Option<Units>,
    pub col_between:   Option<Units>,
    pub grid_rows:     Option<Vec<Units>>,
    pub grid_cols:     Option<Vec<Units>>,
    pub row_index:     Option<usize>,
    pub col_index:     Option<usize>,
    pub col_span:      Option<usize>,
    pub row_span:      Option<usize>,
    pub border_left:   Option<Units>,
    pub border_right:  Option<Units>,
    pub border_top:    Option<Units>,
    pub border_bottom: Option<Units>,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            visible:       true,
            layout_type:   None,
            position_type: None,
            width:         None,
            height:        None,
            min_width:     None,
            min_height:    None,
            max_width:     None,
            max_height:    None,
            left:          None,
            right:         None,
            top:           None,
            bottom:        None,
            min_left:      None,
            max_left:      None,
            min_right:     None,
            max_right:     None,
            min_top:       None,
            max_top:       None,
            min_bottom:    None,
            max_bottom:    None,
            child_left:    None,
            child_right:   None,
            child_top:     None,
            child_bottom:  None,
            row_between:   None,
            col_between:   None,
            grid_rows:     None,
            grid_cols:     None,
            row_index:     None,
            col_index:     None,
            col_span:      None,
            row_span:      None,
            border_left:   None,
            border_right:  None,
            border_top:    None,
            border_bottom: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Widget(Rc<RefCell<WidgetImpl>>);

impl Widget {
    pub fn new(style: Rc<Style>) -> Self {
        Self(Rc::new(RefCell::new(WidgetImpl::new(style))))
    }

    pub fn for_each_child<F: FnMut(&WidgetImpl, usize, bool)>(&self, f: F) {
        self.0.borrow().for_each_child(f);
    }

    pub fn from_weak(w: &Weak<RefCell<WidgetImpl>>) -> Option<Widget> {
        w.upgrade().map(|w| Widget(w))
    }

    pub fn as_weak(&self) -> Weak<RefCell<WidgetImpl>> {
        Rc::downgrade(&self.0)
    }

    pub fn reg<F: 'static + FnMut(&mut std::any::Any, Widget, &Event)>(&self, ev_name: &str, cb: F) {
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

    #[allow(unused)]
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

    pub fn set_fixed_id(&self, id: usize) {
        self.0.borrow_mut().set_fixed_id(id);
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

    pub fn set_pos(&self, pos: Rect) {
        self.emit_redraw_required();
        self.0.borrow_mut().pos       = pos;
        self.0.borrow_mut().inner_pos = pos;
    }

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

    pub fn remove_childs(&self) {
        self.0.borrow_mut().remove_childs();
    }

    pub fn remove_child(&self, child: Widget) {
        self.0.borrow_mut().remove_child(child);
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
    style:          Rc<Style>,
    notifier:       Option<UINotifierRef>,
    data_gen:       u64,

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
            data_gen:       0,
            pos:            Rect::from(0.0, 0.0, 0.0, 0.0),
            inner_pos:      Rect::from(0.0, 0.0, 0.0, 0.0),
            layout:         Layout::new(),
            notifier:       None,
            cached:         false,
            cache_img:      None,
            style,
        }
    }

    pub fn for_each_child<F: FnMut(&WidgetImpl, usize, bool)>(&self, mut f: F) {
        if let Some(childs) = &self.childs {
            let len = childs.len();
            for (i, w) in childs.iter().enumerate() {
                f(&mut w.0.borrow(), i, (i + 1) == len)
            }
        }
    }

    pub fn reg(&mut self, ev_name: &str, cb: Box<dyn FnMut(&mut std::any::Any, Widget, &Event)>) {
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

    pub fn set_fixed_id(&mut self, id: usize) {
        self.id = id;
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
            let current_data_gen = ctrl.get_generation();
            let has_changed = self.data_gen != current_data_gen;
            self.data_gen = current_data_gen;
            has_changed
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

    pub fn remove_childs(&mut self) {
        if let Some(childs) = &mut self.childs {
            if !childs.is_empty() {
                for c in childs.iter() {
                    c.0.borrow_mut().parent = None;
                }

                childs.clear();

                self.notifier.as_mut().map(|n| n.set_tree_changed());
            }
        }
    }

    pub fn remove_child(&mut self, child: Widget) {
        let child_addr = child.0.as_ptr();

        if let Some(childs) = &mut self.childs {
            let mut idx   = None;
            let mut child = None;

            for (i, c) in childs.iter().enumerate() {
                if std::ptr::eq(c.0.as_ptr(), child_addr) {
                    idx   = Some(i);
                    child = Some(c.clone());
                    break;
                }
            }

            if let Some(child) = child {
                child.0.borrow_mut().parent = None;
            }

            if let Some(idx) = idx {
                childs.remove(idx);
                self.notifier.as_mut().map(|n| n.set_tree_changed());
            }
        }
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

    let ctrl   = widget.0.borrow_mut().ctrl.take();
    let childs = widget.0.borrow_mut().childs.take();
    let wid_id = widget.0.borrow().id();

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
    let ctrl   = widget.0.borrow_mut().ctrl.take();
    let childs = widget.0.borrow_mut().childs.take();

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

pub fn widget_walk_impl<F: FnMut(&Widget, Option<&Widget>, bool, bool)>(widget: &Widget, parent: Option<&Widget>, mut cb: &mut F, is_first: bool, is_last: bool) {
    cb(widget, parent, is_first, is_last);

    if let Some(childs) = &widget.0.borrow().childs {
        let len = childs.len();
        for (i, c) in childs.iter().enumerate() {
            widget_walk_impl(&c, Some(&widget), cb, i == 0, (i + 1) == len)
        }
    }
}

pub fn widget_walk<F: FnMut(&Widget, Option<&Widget>, bool, bool)>(widget: &Widget, mut cb: F) {
    widget_walk_impl(widget, None, &mut cb, true, true);
}
