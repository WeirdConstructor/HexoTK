use crate::{InputEvent, EventCore, Control, Painter, Rect, UINotifierRef, Event};
use crate::painter::ImgRef;
use crate::style::Style;
use std::rc::{Weak, Rc};
use std::cell::RefCell;

#[derive(Debug, Clone, Copy)]
pub enum Align { Left, Right, Center }

#[derive(Debug, Clone, Copy)]
pub enum VAlign { Top, Bottom, Middle }

impl Align {
    pub fn calc_offs(&self, w: f32, max: f32) -> f32 {
        match self {
            Align::Left   => 0.0,
            Align::Right  => max - w,
            Align::Center => ((max - w) * 0.5).round(),
        }
    }
}

impl VAlign {
    pub fn calc_offs(&self, h: f32, max: f32) -> f32 {
        match self {
            VAlign::Top    => 0.0,
            VAlign::Bottom => max - h,
            VAlign::Middle => ((max - h) * 0.5).round(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Units {
    Px(f32),
    Perc(f32),
    S(f32),
}

impl Units {
    pub fn calc(&self, len: f32) -> f32 {
        match self {
            Units::Px(l)   => *l,
            Units::Perc(l) => (len * *l) / 100.0,
            Units::S(l)    => (len * *l) / 100.0,
        }
    }

    pub fn get_stretch(&self) -> Option<f32> {
        if let Units::S(l) = self {
            Some(*l)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LayoutType {
    None,
    HBox,
    VBox,
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub layout_type:    LayoutType,
    pub visible:        bool,
    pub width:          Units,
    pub height:         Units,
    pub pad_left:       Units,
    pub pad_right:      Units,
    pub pad_top:        Units,
    pub pad_bottom:     Units,
    pub margin_left:    Units,
    pub margin_right:   Units,
    pub margin_top:     Units,
    pub margin_bottom:  Units,
    pub spacing:        Units,
    pub align:          Align,
    pub valign:         VAlign,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            layout_type:    LayoutType::None,
            visible:        true,
            width:          Units::Perc(100.0),
            height:         Units::Perc(100.0),
            pad_left:       Units::Px(2.0),
            pad_right:      Units::Px(2.0),
            pad_top:        Units::Px(2.0),
            pad_bottom:     Units::Px(2.0),
            margin_left:    Units::Px(1.0),
            margin_right:   Units::Px(1.0),
            margin_top:     Units::Px(1.0),
            margin_bottom:  Units::Px(1.0),
            spacing:        Units::Px(4.0),
            align:          Align::Center,
            valign:         VAlign::Middle,
        }
    }
}

#[derive(Clone)]
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

    pub fn set_direct_ctrl(&self, ctrl: Control, pos: Rect) {
        self.0.borrow_mut().set_direct_ctrl(ctrl, pos)
    }

    pub fn take_ctrl(&self) -> Option<Box<Control>> {
        self.0.borrow_mut().ctrl.take()
    }

    pub fn give_ctrl_back(&self, ctrl: Box<Control>) {
        self.0.borrow_mut().ctrl = Some(ctrl);
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
    handle_childs:  Option<Vec<(Widget, Widget)>>,
    pos:            Rect,
    inner_pos:      Rect,
    layout:         Layout,
    layout_tmp:     Vec<Rect>,
    style:          Rc<Style>,
    notifier:       Option<UINotifierRef>,

    cached:         bool,
    cache_img:      Option<ImgRef>,
}

impl WidgetImpl {
    pub fn new(style: Rc<Style>) -> Self {
        Self {
            id:             0,
            evc:            Some(EventCore::new()),
            parent:         None,
            childs:         Some(vec![]),
            handle_childs:  Some(vec![]),
            ctrl:           None,
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
        let layout = &self.layout;
        if !layout.visible {
            self.pos = Rect::from(0.0, 0.0, 0.0, 0.0);
            return None;
        }

        self.pos = pos;
        self.emit_redraw_required();

        let pad_l = layout.pad_left  .calc(pos.w).round();
        let pad_t = layout.pad_top   .calc(pos.h).round();
        let pad_r = layout.pad_right .calc(pos.w).round();
        let pad_b = layout.pad_bottom.calc(pos.h).round();

        let inner_pos =
            Rect {
                x: pos.x + pad_l,
                y: pos.x + pad_t,
                w: pos.w - (pad_l + pad_r),
                h: pos.h - (pad_t + pad_b),
            };

        self.inner_pos = inner_pos;

        match layout.layout_type {
            LayoutType::None => { },
            LayoutType::VBox => { },
            LayoutType::HBox => {
                let avail_w = pos.w;
                if let Some(childs) = &self.childs {
                    let mut taken_w = 0.0;
                    let mut stretch_sum = 0.0;

//                    let mut max_h = 0.0;

                    for child in childs.iter() {
                        let c            = child.0.borrow();
                        let child_layout = &c.layout;
                        let border       = c.style.border.round();

                        let margin_l = layout.margin_left  .calc(inner_pos.w).round();
                        let margin_r = layout.margin_right .calc(inner_pos.w).round();
                        let margin_t = layout.margin_top   .calc(inner_pos.h).round();
                        let margin_b = layout.margin_bottom.calc(inner_pos.h).round();

                        let margin_w = 2.0 * border + margin_l + margin_r;
                        let margin_h = 2.0 * border + margin_t + margin_b;

                        let avail_child_w = inner_pos.w - margin_w;
                        let avail_child_h = inner_pos.h - margin_h;

                        if let Some(s) = child_layout.width.get_stretch() {
                            stretch_sum += s;
                            taken_w     += margin_w;
                        } else {
                            taken_w +=
                                margin_w
                                + child_layout.width.calc(avail_child_w).round();
                        }

//                        let h = child_layout.height.calc(avail_child_h);
//                        max_h = max_h.max(h);
                    }

                    let mut rest_w = inner_pos.w - taken_w;

                    let mut x = inner_pos.x;

                    for child in childs.iter() {
                        let mut c        = child.0.borrow_mut();
                        let child_layout = &c.layout;
                        let border       = c.style.border.round();

                        let margin_l = layout.margin_left  .calc(inner_pos.w).round();
                        let margin_r = layout.margin_right .calc(inner_pos.w).round();
                        let margin_t = layout.margin_top   .calc(inner_pos.h).round();
                        let margin_b = layout.margin_bottom.calc(inner_pos.h).round();

                        let margin_w = 2.0 * border + margin_l + margin_r;
                        let margin_h = 2.0 * border + margin_t + margin_b;

                        let avail_child_w = inner_pos.w - margin_w;
                        let avail_child_h = inner_pos.h - margin_h;

                        let cw =
                            if let Some(s) = child_layout.width.get_stretch() {
                                if rest_w > 0.0 && stretch_sum > 0.01 {
                                    ((rest_w * s) / stretch_sum).round()
                                } else {
                                    0.0
                                }
                            } else {
                                child_layout.width.calc(avail_child_w).round()
                            };

                        let ch = child_layout.height.calc(avail_child_h).round();

                        let yo = child_layout.valign.calc_offs(ch, avail_child_h);

                        let child_pos =
                            Rect {
                                x: x          + border + margin_l,
                                y: pos.y + yo + border + margin_t,
                                w: cw,
                                h: ch
                            };

                        x = child_pos.x + child_pos.w;

                        c.relayout(child_pos);
                    }
                }
            },
        }

        Some(self.pos)
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
        self.pos = pos;
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

pub fn widget_walk<F: FnMut(&Widget, Option<&Widget>)>(widget: &Widget, mut cb: F) {
    cb(widget, None);

    let mut hc = {
        let cur_parent = widget.clone();
        let mut w = widget.0.borrow_mut();

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

    widget.0.borrow_mut().handle_childs = hc;
}
