// Copyright (c) 2021-2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::painter::{ImgRef, LblDebugTag};
use crate::style::Style;
use crate::{Control, EvPayload, Event, EventCore, Painter, PopupPos, Rect, UINotifierRef};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use morphorm::{LayoutType, PositionType, Units};

thread_local! {
    pub static WIDGET_ID_COUNTER: RefCell<usize> = RefCell::new(1);
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub visible: bool,

    pub layout_type: Option<LayoutType>,
    pub position_type: Option<PositionType>,
    pub width: Option<Units>,
    pub height: Option<Units>,
    pub min_width: Option<Units>,
    pub min_height: Option<Units>,
    pub max_width: Option<Units>,
    pub max_height: Option<Units>,
    pub left: Option<Units>,
    pub right: Option<Units>,
    pub top: Option<Units>,
    pub bottom: Option<Units>,
    pub min_left: Option<Units>,
    pub max_left: Option<Units>,
    pub min_right: Option<Units>,
    pub max_right: Option<Units>,
    pub min_top: Option<Units>,
    pub max_top: Option<Units>,
    pub min_bottom: Option<Units>,
    pub max_bottom: Option<Units>,
    pub child_left: Option<Units>,
    pub child_right: Option<Units>,
    pub child_top: Option<Units>,
    pub child_bottom: Option<Units>,
    pub row_between: Option<Units>,
    pub col_between: Option<Units>,
    pub grid_rows: Option<Vec<Units>>,
    pub grid_cols: Option<Vec<Units>>,
    pub row_index: Option<usize>,
    pub col_index: Option<usize>,
    pub col_span: Option<usize>,
    pub row_span: Option<usize>,
    pub border_left: Option<Units>,
    pub border_right: Option<Units>,
    pub border_top: Option<Units>,
    pub border_bottom: Option<Units>,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            visible: true,
            layout_type: None,
            position_type: None,
            width: None,
            height: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            left: None,
            right: None,
            top: None,
            bottom: None,
            min_left: None,
            max_left: None,
            min_right: None,
            max_right: None,
            min_top: None,
            max_top: None,
            min_bottom: None,
            max_bottom: None,
            child_left: None,
            child_right: None,
            child_top: None,
            child_bottom: None,
            row_between: None,
            col_between: None,
            grid_rows: None,
            grid_cols: None,
            row_index: None,
            col_index: None,
            col_span: None,
            row_span: None,
            border_left: None,
            border_right: None,
            border_top: None,
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

    pub fn debug_tag(&self) -> LblDebugTag {
        LblDebugTag::from_id(self.unique_id())
    }

    pub fn event(&self, name: &str, data: EvPayload) -> (usize, Event) {
        (self.unique_id(), Event { name: name.to_string(), data })
    }

    pub fn set_tree_pos(&self, idx: usize, depth: usize) {
        self.0.borrow_mut().set_tree_pos(idx, depth)
    }

    pub fn layer_idx(&self) -> usize {
        self.0.borrow().layer_idx()
    }

    pub fn tree_depth(&self) -> usize {
        self.0.borrow().tree_depth()
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

    pub fn reg<F: 'static + FnMut(&mut dyn std::any::Any, Widget, &Event)>(
        &self,
        ev_name: &str,
        cb: F,
    ) {
        self.0.borrow_mut().reg(ev_name, Box::new(cb));
    }

    pub fn take_event_core(&self) -> Option<EventCore> {
        self.0.borrow_mut().take_event_core()
    }

    pub fn set_drag_widget(&self, wid: Widget) {
        self.0.borrow_mut().set_drag_widget(wid);
    }
    pub fn drag_widget(&self) -> Option<Widget> {
        self.0.borrow().drag_widget()
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

    pub fn set_notifier(&self, not: UINotifierRef) {
        self.0.borrow_mut().set_notifier(not)
    }

    pub fn is_hovered(&self) -> bool {
        self.0.borrow().is_hovered()
    }

    pub fn does_show_hover(&self) -> bool {
        self.0.borrow().does_show_hover()
    }

    pub fn unique_id(&self) -> usize {
        self.0.borrow().unique_id()
    }

    pub fn set_tag(&self, tag: String) {
        self.0.borrow_mut().set_tag(tag);
    }
    pub fn tag(&self) -> String {
        self.0.borrow().tag().to_string()
    }

    pub fn check_data_change(&self) -> bool {
        self.0.borrow_mut().check_data_change()
    }

    pub fn set_ctrl(&self, ctrl: Control) {
        self.0.borrow_mut().set_ctrl(ctrl)
    }

    pub fn has_default_style(&self) -> bool {
        self.0.borrow().has_default_style()
    }

    pub fn take_ctrl(&self) -> Option<Box<Control>> {
        self.0.borrow_mut().ctrl.take()
    }

    pub fn give_ctrl_back(&self, ctrl: Box<Control>) {
        self.0.borrow_mut().ctrl = Some(ctrl);
    }

    pub fn can_hover(&self) -> bool {
        self.0.borrow_mut().can_hover()
    }

    pub fn set_pos(&self, pos: Rect) {
        self.emit_redraw_required();
        self.0.borrow_mut().pos = pos;
    }

    pub fn pos(&self) -> Rect {
        self.0.borrow().pos
    }

    pub fn style(&self) -> Rc<Style> {
        self.0.borrow().style.clone()
    }

    pub fn set_style(&self, style: Rc<Style>) {
        self.0.borrow_mut().set_style(style);
    }

    pub fn parent(&self) -> Option<Widget> {
        self.0.borrow().parent()
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

    pub(crate) fn change_layout_silent<R, F: FnOnce(&mut Layout) -> R>(&self, f: F) -> R {
        self.0.borrow_mut().change_layout_silent(f)
    }

    pub fn is_visible(&self) -> bool {
        self.0.borrow().is_visible()
    }

    pub fn show(&self) {
        self.0.borrow_mut().change_layout(|layout| layout.visible = true);
    }

    pub fn hide(&self) {
        self.0.borrow_mut().change_layout(|layout| layout.visible = false);
    }

    pub fn auto_hide(&self) {
        self.0.borrow_mut().auto_hide = true;
        self.0.borrow_mut().notifier.as_mut().map(|n| n.set_tree_changed());
    }

    pub fn wants_auto_hide(&self) -> bool {
        self.0.borrow_mut().wants_auto_hide()
    }

    pub fn popup_at(&self, pos: PopupPos) {
        self.0.borrow().popup_at(pos)
    }
}

pub struct WidgetImpl {
    unique_id: usize,
    layer_idx: usize,
    tree_depth: usize,
    pub evc: Option<EventCore>,
    parent: Option<Weak<RefCell<WidgetImpl>>>,
    childs: Option<Vec<Widget>>,
    pub ctrl: Option<Box<Control>>,
    pos: Rect,
    layout: Layout,
    style: Rc<Style>,
    notifier: Option<UINotifierRef>,
    data_gen: u64,
    drag_widget: Option<Widget>,
    auto_hide: bool,
    show_hover: bool,
    tag: Option<String>,

    cached: bool,
    cache_img: Option<ImgRef>,
}

impl std::fmt::Debug for WidgetImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Widget")
            .field("unique_id", &self.unique_id)
            .field("tag", &self.tag)
            .finish()
    }
}

impl Drop for WidgetImpl {
    fn drop(&mut self) {
        eprintln!("HexoTK drop widget ID={}", self.unique_id);
    }
}

impl WidgetImpl {
    pub fn new(style: Rc<Style>) -> Self {
        let unique_id = WIDGET_ID_COUNTER.with(|cnt| {
            let unique_id = *cnt.borrow();
            *cnt.borrow_mut() += 1;
            unique_id
        });

        Self {
            unique_id,
            layer_idx: 0,
            tree_depth: 0,
            evc: Some(EventCore::new()),
            parent: None,
            childs: Some(vec![]),
            ctrl: Some(Box::new(Control::None)),
            data_gen: 0,
            pos: Rect::from(0.0, 0.0, 0.0, 0.0),
            layout: Layout::new(),
            notifier: None,
            drag_widget: None,
            cached: false,
            cache_img: None,
            auto_hide: false,
            show_hover: false,
            tag: None,
            style,
        }
    }

    pub fn set_tree_pos(&mut self, idx: usize, depth: usize) {
        self.layer_idx = idx;
        self.tree_depth = depth;
    }

    pub fn layer_idx(&self) -> usize {
        self.layer_idx
    }

    pub fn tree_depth(&self) -> usize {
        self.tree_depth
    }

    pub fn for_each_child<F: FnMut(&WidgetImpl, usize, bool)>(&self, mut f: F) {
        if let Some(childs) = &self.childs {
            let len = childs.len();
            for (i, w) in childs.iter().enumerate() {
                f(&mut w.0.borrow(), i, (i + 1) == len)
            }
        }
    }

    pub fn is_visible(&self) -> bool {
        if let Some(parent) = self.parent() {
            parent.is_visible() && self.with_layout(|layout| layout.visible)
        } else {
            self.with_layout(|layout| layout.visible)
        }
    }

    pub fn reg(
        &mut self,
        ev_name: &str,
        cb: Box<dyn FnMut(&mut dyn std::any::Any, Widget, &Event)>,
    ) {
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

    pub fn set_drag_widget(&mut self, wid: Widget) {
        self.drag_widget = Some(wid);
    }
    pub fn drag_widget(&self) -> Option<Widget> {
        self.drag_widget.clone()
    }

    pub fn is_cached(&mut self) -> bool {
        self.cached
    }
    pub fn enable_cache(&mut self) {
        self.cached = true;
    }

    pub fn take_cache_img(&mut self) -> Option<ImgRef> {
        self.cache_img.take()
    }
    pub fn give_cache_img(&mut self, img: ImgRef) {
        self.cache_img = Some(img);
    }

    pub fn with_layout<R, F: FnOnce(&Layout) -> R>(&self, f: F) -> R {
        f(&self.layout)
    }

    pub fn change_layout<R, F: FnOnce(&mut Layout) -> R>(&mut self, f: F) -> R {
        let ret = f(&mut self.layout);
        self.emit_layout_change();
        ret
    }

    pub(crate) fn change_layout_silent<R, F: FnOnce(&mut Layout) -> R>(&mut self, f: F) -> R {
        f(&mut self.layout)
    }

    pub fn relayout(&mut self, pos: Rect) -> Option<Rect> {
        // call relayout on childs...
        Some(pos)
    }

    fn emit_layout_change(&self) {
        self.notifier.as_ref().map(|n| n.set_layout_changed());
    }

    pub fn emit_redraw_required(&self) {
        if self.is_visible() {
            //d// println!("Emit Redraw by {} / {}", self.unique_id(), self.tag());
            self.notifier.as_ref().map(|n| n.redraw(self.unique_id));
        }
    }

    pub fn activate(&self) {
        self.notifier.as_ref().map(|n| n.activate(self.unique_id));
    }

    pub fn deactivate(&self) {
        self.notifier.as_ref().map(|n| n.deactivate(self.unique_id));
    }

    pub fn is_active(&self) -> bool {
        self.unique_id == self.notifier.as_ref().map(|n| n.active()).flatten().unwrap_or(0)
    }

    pub fn set_notifier(&mut self, not: UINotifierRef) {
        self.notifier = Some(not);
    }

    pub fn is_hovered(&self) -> bool {
        self.unique_id == self.notifier.as_ref().map(|n| n.hover()).unwrap_or(0)
    }

    pub fn does_show_hover(&self) -> bool {
        self.show_hover
    }

    pub fn unique_id(&self) -> usize {
        self.unique_id
    }

    pub fn set_tag(&mut self, tag: String) {
        self.tag = Some(tag);
    }
    pub fn tag(&self) -> &str {
        self.tag.as_ref().map(|t| &t[..]).unwrap_or("")
    }

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
        self.show_hover = ctrl.can_show_hover();
        self.ctrl = Some(Box::new(ctrl));
    }

    pub fn has_default_style(&self) -> bool {
        self.ctrl.as_ref().map(|c| c.has_default_style()).unwrap_or(false)
    }

    pub fn can_hover(&self) -> bool {
        self.ctrl.as_ref().map(|c| c.can_hover()).unwrap_or(false)
    }

    pub fn pos(&self) -> Rect {
        self.pos
    }

    pub fn style(&self) -> &Style {
        &*self.style
    }

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

    pub fn popup_at(&self, pos: PopupPos) {
        self.notifier.as_ref().map(|n| n.popup(self.unique_id, pos));
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
            let mut idx = None;
            let mut child = None;

            for (i, c) in childs.iter().enumerate() {
                if std::ptr::eq(c.0.as_ptr(), child_addr) {
                    idx = Some(i);
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

    pub fn wants_auto_hide(&self) -> bool {
        self.auto_hide
    }

    pub fn set_parent(&mut self, parent: &Widget) {
        self.parent = Some(Rc::downgrade(&parent.0));
    }

    pub fn clear(&mut self, recursive: bool) {
        self.evc.as_mut().map(|evc| evc.clear());
        self.ctrl = None;
        self.parent = None;
        self.unique_id = usize::MAX;

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
    draw_origin: Rect,
    painter: &mut Painter,
) {
    let visible = widget.0.borrow().layout.visible;
    if !visible {
        return;
    }

    //d// println!("REDRAW {} ({}) is_cached={}", widget.unique_id(), widget.tag(), widget.is_cached());

    let ctrl = widget.0.borrow_mut().ctrl.take();
    let childs = widget.0.borrow_mut().childs.take();

    if let Some(mut ctrl) = ctrl {
        ctrl.draw(draw_origin, widget, Some(redraw), childs.as_ref(), painter);

        widget.0.borrow_mut().childs = childs;
        widget.0.borrow_mut().ctrl = Some(ctrl);
    }
}

pub fn widget_draw_shallow(
    widget: &Widget,
    _redraw: bool,
    draw_origin: Rect,
    painter: &mut Painter,
) {
    let visible = widget.0.borrow().layout.visible;
    if !visible {
        return;
    }

    let ctrl = widget.0.borrow_mut().ctrl.take();
    if let Some(mut ctrl) = ctrl {
        ctrl.draw(draw_origin, widget, None, None, painter);
        widget.0.borrow_mut().ctrl = Some(ctrl);
    }
}

pub fn widget_draw_frame(widget: &Widget, painter: &mut Painter) {
    let ctrl = widget.0.borrow_mut().ctrl.take();
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

pub fn widget_annotate_drop_event(widget: &Widget, mouse_pos: (f32, f32), ev: Event) -> Event {
    let ctrl = widget.0.borrow_mut().ctrl.take();

    if let Some(mut ctrl) = ctrl {
        let ret = ctrl.annotate_drop_event(mouse_pos, ev);
        widget.0.borrow_mut().ctrl = Some(ctrl);
        ret
    } else {
        ev
    }
}

pub fn widget_walk_impl<F: FnMut(&Widget, Option<&Widget>, bool, bool, usize)>(
    widget: &Widget,
    parent: Option<&Widget>,
    cb: &mut F,
    is_first: bool,
    is_last: bool,
    depth: usize,
) {
    cb(widget, parent, is_first, is_last, depth);

    if let Some(childs) = &widget.0.borrow().childs {
        let len = childs.len();
        for (i, c) in childs.iter().enumerate() {
            widget_walk_impl(&c, Some(&widget), cb, i == 0, (i + 1) == len, depth + 1)
        }
    }
}

pub fn widget_walk<F: FnMut(&Widget, Option<&Widget>, bool, bool, usize)>(
    widget: &Widget,
    mut cb: F,
) {
    widget_walk_impl(widget, None, &mut cb, true, true, 0);
}

pub fn widget_walk_parents<F: FnMut(&Widget)>(widget: &Widget, mut cb: F) {
    let mut cur_parent = widget.parent();

    while cur_parent.is_some() {
        cb(cur_parent.as_ref().unwrap());
        cur_parent = cur_parent.unwrap().parent();
    }
}
