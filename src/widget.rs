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
    pub fn calc_offs(&self, w: f32, rest: f32) -> f32 {
        match self {
            Align::Left   => 0.0,
            Align::Right  => rest,
            Align::Center => (rest * 0.5).round(),
        }
    }
}

impl VAlign {
    pub fn calc_offs(&self, h: f32, rest: f32) -> f32 {
        match self {
            VAlign::Top    => 0.0,
            VAlign::Bottom => rest,
            VAlign::Middle => (rest * 0.5).round(),
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
pub struct MarginCalc {
    lft:  f32,
    rgt:  f32,
    top:  f32,
    bot:  f32,
    w:    f32,
    h:    f32,
}

impl MarginCalc {
    pub fn from(widget: &WidgetImpl, unit_pos: Rect) -> Self {
        let border = widget.style.border.round();
        let layout = &widget.layout;

        let lft = layout.margin_left  .calc(unit_pos.w).round();
        let rgt = layout.margin_right .calc(unit_pos.w).round();
        let top = layout.margin_top   .calc(unit_pos.h).round();
        let bot = layout.margin_bottom.calc(unit_pos.h).round();

        let w = 2.0 * border + lft + rgt;
        let h = 2.0 * border + top + bot;

        Self { lft, rgt, top, bot, w, h }
    }
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub layout_type:    LayoutType,
    pub visible:        bool,
    pub width:          Units,
    pub height:         Units,
    pub min_width:      Units,
    pub max_width:      Units,
    pub min_height:     Units,
    pub max_height:     Units,
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
            min_width:      Units::Px(0.0),
            max_width:      Units::Px(0.0),
            min_height:     Units::Px(0.0),
            max_height:     Units::Px(0.0),
            pad_left:       Units::Px(0.0),
            pad_right:      Units::Px(0.0),
            pad_top:        Units::Px(1.0),
            pad_bottom:     Units::Px(0.0),
//            pad_left:       Units::Px(2.0),
//            pad_right:      Units::Px(2.0),
//            pad_top:        Units::Px(2.0),
//            pad_bottom:     Units::Px(2.0),
            margin_left:    Units::Px(0.0),
            margin_right:   Units::Px(0.0),
            margin_top:     Units::Px(0.0),
            margin_bottom:  Units::Px(0.0),
            spacing:        Units::Px(0.0),
//            spacing:        Units::Px(0.0),
            align:          Align::Center,
            valign:         VAlign::Middle,
        }
    }

    pub fn calc_static_height(&self, reference_h: f32) -> (f32, f32) {
        let h     = self.height.calc(reference_h).round();
        let min_h = self.min_height.calc(reference_h).round();
        let max_h = self.max_height.calc(reference_h).round();

        let h = min_h.max(h);

        if max_h > 0.1 && h > max_h {
            (max_h, h - max_h)
        } else {
            (h, 0.0)
        }
    }

    pub fn calc_static_width(&self, reference_w: f32) -> (f32, f32) {
        let w     = self.width.calc(reference_w).round();
        let min_w = self.min_width.calc(reference_w).round();
        let max_w = self.max_width.calc(reference_w).round();

        let w = min_w.max(w);

        if max_w > 0.1 && w > max_w {
            (max_w, w - max_w)
        } else {
            (w, 0.0)
        }
    }

    pub fn calc_stretch(
        &self, perc_reference: f32, rest_px: f32,
        stretch_reference: f32, width: bool
    ) -> Option<(f32, f32)>
    {
        let (min, max, nominal) =
            if width { (self.min_width,  self.max_width,  self.width) }
            else     { (self.min_height, self.max_height, self.height) };

        let min_px = min.calc(perc_reference).round();
        let max_px = max.calc(perc_reference).round();

        let s = nominal.get_stretch()?;
        let ret =
            if rest_px > 0.0 && stretch_reference > 0.01 {
                ((rest_px * s) / stretch_reference).round()
            } else {
                0.0
            };

        let px = ret + min_px;

        if max_px > 0.1 && px > max_px {
            Some((max_px, px - max_px))
        } else {
            Some((ret + min_px, 0.0))
        }
    }

    pub fn apply_align_offs(
        &self, mut pos: Rect, w: f32, align_w: f32, h: f32, align_h: f32
    ) -> Rect
    {
        let y_align_offs = self.valign.calc_offs(h, align_h);
        let x_align_offs = self.align.calc_offs(w, align_w);

        pos.x += x_align_offs;
        pos.y += y_align_offs;

        pos
    }


    /// Calculates the box size of this layout.
    ///
    /// * `perc_reference` is the reference for the 100%
    /// * `stretch_size` is the amount of pixels the child could stretch
    /// * `stretch_sum` is the sum of all stretch values of the childs.
    ///
    /// Returns (px, align_px)  - the actually taken up size and the
    /// space that is left over for the child to align into.
    pub fn calc_box_size(
        &self, perc_reference: f32, stretch_size: f32, stretch_sum: f32,
        width: bool
    ) -> (f32, f32)
    {
        if width {
            self.calc_stretch(perc_reference, stretch_size, stretch_sum, true)
                .unwrap_or_else(||
                    self.calc_static_width(perc_reference))
        } else {
            self.calc_stretch(perc_reference, stretch_size, stretch_sum, false)
                .unwrap_or_else(||
                    self.calc_static_height(perc_reference))
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

    pub fn set_ctrl(&self, ctrl: Control) {
        self.0.borrow_mut().set_ctrl(ctrl)
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

    /// Calculate the space that is "wasted" on margins,
    /// borders of the childs and spacing between them,
    /// in relation to the inner padded space of the HBox widget.
    fn sum_box_child_margins(&self, relation_pos: Rect, width: bool) -> f32 {
        let spacing =
            self.layout.spacing.calc(
                if width { relation_pos.w } else { relation_pos.h });

        if let Some(childs) = &self.childs {
            let mut sum = 0.0;

            for (i, child) in childs.iter().enumerate() {
                let c = child.0.borrow();

                let margin = MarginCalc::from(&c, relation_pos);

                if width { sum += margin.w; }
                else     { sum += margin.h; }

                if i > 0 { sum += spacing; }
            }

            sum
        } else {
            0.0
        }
    }

    /// Calculates the stretch sum of the childs that are stretching.
    fn sum_child_stretch(&self, width: bool) -> f32 {
        let mut stretch_sum = 0.0;

        if let Some(childs) = &self.childs {
            for (i, child) in childs.iter().enumerate() {
                let c            = child.0.borrow();
                let child_layout = &c.layout;

                let units =
                    if width { child_layout.width }
                    else     { child_layout.height };

                if let Some(s) = units.get_stretch() {
                    stretch_sum += s;
                }
            }
        }

        stretch_sum
    }

    fn box_min_size(&self, perc_reference: f32, relation_pos: Rect, width: bool) -> f32 {

        let margin = MarginCalc::from(self, relation_pos);

        let mut min_size =
            if width { margin.w }
            else     { margin.h };

        // Stretch takes into account the minimal width/height
        // of the widgets. The goal is, that min_size
        // can be used as basis to calculate the rest
        // width/height that the stretch widgets can distribute
        // among each other.

        let layout = &self.layout;
        if width {
            if let Some(_) = layout.width.get_stretch() {
                min_size += layout.min_width.calc(perc_reference);
            } else {
                let (w, align_w) =
                    layout.calc_static_width(perc_reference);

                min_size += w + align_w;
            }
        } else {
            if let Some(_) = layout.height.get_stretch() {
                min_size += layout.min_height.calc(perc_reference);
            } else {
                let (h, align_h) =
                    layout.calc_static_height(perc_reference);

                min_size += h + align_h;
            }
        }

        min_size
    }

    /// Calculates the minimal size that will be occupied by the children.
    /// This is neccessary to calculate the available space for stretching
    /// the childs that stretch.
    ///
    /// `perc_reference` is decoupled from `relation_pos` for the Units::Perc,
    /// because the 100% of the childs need to relate to the actually available
    /// space.
    fn sum_box_child_min_size(
        &self, perc_reference: f32, relation_pos: Rect, width: bool
    ) -> f32
    {
        let spacing =
            self.layout.spacing.calc(
                if width { relation_pos.w } else { relation_pos.h });

        let mut min_size = 0.0;

        if let Some(childs) = &self.childs {
            for (i, child) in childs.iter().enumerate() {
                min_size +=
                    child.0.borrow().box_min_size(perc_reference, relation_pos, width);

                if i > 0 { min_size += spacing; }
            }
        }

        min_size
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
                y: pos.y + pad_t,
                w: pos.w - (pad_l + pad_r),
                h: pos.h - (pad_t + pad_b),
            };

        self.inner_pos = inner_pos;

        match layout.layout_type {
            LayoutType::None => { },
            LayoutType::VBox => {
                let avail_h = pos.h;

                let spacing = layout.spacing.calc(inner_pos.h);

                let margin_sum_h = self.sum_box_child_margins(inner_pos, false);

                // The reference size is the size without margins and padding
                // of the parent that could in theory be allocated to the children.
                // It defines the 100% for calculating the Units::Perc.
                let perc_ref_h = inner_pos.h - margin_sum_h;

                let min_child_h =
                    self.sum_box_child_min_size(perc_ref_h, inner_pos, false);
                let stretch_h     = inner_pos.h - min_child_h;
                let stretch_sum_h = self.sum_child_stretch(false);

                if let Some(childs) = &self.childs {
                    // Now calculate the space taken by the fixed size
                    // widgets and the margins of the stretch widgets:
                    let mut y = inner_pos.y;

                    for child in childs.iter() {
                        let mut c        = child.0.borrow_mut();
                        let child_layout = &c.layout;

                        let (ch, align_h) =
                            child_layout.calc_box_size(
                                perc_ref_h, stretch_h, stretch_sum_h, false);

                        let margin = MarginCalc::from(&c, inner_pos);
                        let (cw, align_w) =
                            child_layout.calc_box_size(
                                inner_pos.w - margin.w, 0.0, 0.0, true);

                        let border = c.style.border.round();
                        let mut child_pos =
                            Rect {
                                x: border + margin.lft,
                                y: border + margin.top,
                                w: cw,
                                h: ch
                            };

                        let child_pos =
                            child_layout.apply_align_offs(
                                child_pos, cw, align_w, ch, align_h);

                        let child_pos = child_pos.offs(inner_pos.x, y);

                        c.relayout(child_pos);

                        y += margin.h + ch + align_h + spacing;
                    }
                }
            },
            LayoutType::HBox => {
                let avail_w = pos.w;

                let spacing = layout.spacing.calc(inner_pos.w);

                let margin_sum_w = self.sum_box_child_margins(inner_pos, true);

                // The reference size is the size without margins and padding
                // of the parent that could in theory be allocated to the children.
                // It defines the 100% for calculating the Units::Perc.
                let perc_ref_w = inner_pos.w - margin_sum_w;

                let min_child_w =
                    self.sum_box_child_min_size(perc_ref_w, inner_pos, true);
                let stretch_w     = inner_pos.w - min_child_w;
                let stretch_sum_w = self.sum_child_stretch(true);

                if let Some(childs) = &self.childs {
                    // Now calculate the space taken by the fixed size
                    // widgets and the margins of the stretch widgets:
                    let mut x = inner_pos.x;

                    for child in childs.iter() {
                        let mut c        = child.0.borrow_mut();
                        let child_layout = &c.layout;

                        let (cw, align_w) =
                            child_layout.calc_box_size(
                                perc_ref_w, stretch_w, stretch_sum_w, true);

                        let margin = MarginCalc::from(&c, inner_pos);
                        let (ch, align_h) =
                            child_layout.calc_box_size(
                                inner_pos.h - margin.h, 0.0, 0.0, false);

                        let border = c.style.border.round();
                        let mut child_pos =
                            Rect {
                                x: border + margin.lft,
                                y: border + margin.top,
                                w: cw,
                                h: ch
                            };

                        let child_pos =
                            child_layout.apply_align_offs(
                                child_pos, cw, align_w, ch, align_h);

                        let child_pos = child_pos.offs(x, inner_pos.y);

                        c.relayout(child_pos);

                        x += margin.w + cw + align_w + spacing;
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

    pub fn set_ctrl(&mut self, ctrl: Control) {
        self.ctrl = Some(Box::new(ctrl));
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
