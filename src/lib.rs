// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

mod widget;
mod ui;
mod window;
mod rect;
mod painter;
#[allow(unused)]
pub mod style;
mod layout;
mod widget_store;
mod widgets;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashSet;

use keyboard_types::{KeyboardEvent}; // Key
pub use window::open_window;
pub use rect::Rect;
use painter::Painter;
pub use widget::Widget;
use widget::{widget_draw, widget_draw_frame};
pub use ui::UI;
pub use ui::FrameScript;
pub use ui::TestDriver;
pub use style::{Style, Align, VAlign, BorderStyle};

pub use widgets::Entry;
pub use widgets::WichText;
pub use widgets::{HexKnob, ParamModel, DummyParamModel, ChangeRes};
pub use widgets::{HexGrid, HexGridModel, HexCell, HexDir, HexEdge, HexHLight};
pub use widgets::WichTextSimpleDataStore;
pub use widgets::EditableText;
pub use widgets::TextField;

pub use morphorm::{Units, LayoutType, PositionType};

use std::fmt::Debug;

pub trait Mutable {
    fn get_generation(&mut self) -> u64;
}

pub trait Text {
    fn fmt<'a>(&self, buf: &'a mut [u8]) -> &'a str {
        let l = self.fmt_l(buf);

        std::str::from_utf8(&buf[0..l]).unwrap_or("")
    }

    fn fmt_l<'a>(&self, _buf: &'a mut [u8]) -> usize { 0 }
}

pub struct CloneMutable<T> where T: PartialEq + Clone {
    generation: u64,
    old: Option<T>,
    cur: T,
}

impl<T> CloneMutable<T> where T: PartialEq + Clone {
    pub fn new(cur: T) -> Self {
        Self {
            generation: 0,
            old: None,
            cur,
        }
    }
}

impl<T> std::ops::Deref for CloneMutable<T> where T: PartialEq + Clone {
    type Target = T;
    fn deref(&self) -> &T { &self.cur }
}
impl<T> std::ops::DerefMut for CloneMutable<T> where T: PartialEq + Clone {
    fn deref_mut(&mut self) -> &mut T { &mut self.cur }
}

impl<T> Mutable for CloneMutable<T> where T: PartialEq + Clone {
    fn get_generation(&mut self) -> u64 {
        let change =
            self.old.as_ref()
                .map(|o| *o != self.cur)
                .unwrap_or(true);
        if change {
            self.old = Some(self.cur.clone());
            self.generation += 1;
        }

        self.generation
    }
}

impl Mutable for String {
    fn get_generation(&mut self) -> u64 { 0 }
}

impl<T> Mutable for Rc<RefCell<T>> where T: Mutable {
    fn get_generation(&mut self) -> u64 { self.borrow_mut().get_generation() }
}

impl<T> Mutable for std::sync::Arc<std::sync::Mutex<T>> where T: Mutable {
    fn get_generation(&mut self) -> u64 {
        if let Ok(mut data) = self.lock() {
            data.get_generation()
        } else {
            0
        }
    }
}


impl Text for String {
    fn fmt<'a>(&self, buf: &'a mut [u8]) -> &'a str {
        let b = self.as_bytes();
        let l = buf.len().min(b.len());
        buf[0..l].copy_from_slice(&b[0..l]);
        std::str::from_utf8(&buf[0..l]).unwrap_or("")
    }
}

impl<T> Text for Box<T> where T: Text {
    fn fmt<'a>(&self, buf: &'a mut [u8]) -> &'a str {
        (**self).fmt(buf)
    }

    fn fmt_l<'a>(&self, buf: &'a mut [u8]) -> usize {
        (**self).fmt_l(buf)
    }
}

impl<T> Text for Rc<RefCell<T>> where T: Text {
    fn fmt<'a>(&self, buf: &'a mut [u8]) -> &'a str {
        self.borrow().fmt(buf)
    }

    fn fmt_l<'a>(&self, buf: &'a mut [u8]) -> usize {
        self.borrow().fmt_l(buf)
    }
}

impl<T> Text for std::sync::Arc<std::sync::Mutex<T>> where T: Text {
    fn fmt<'a>(&self, buf: &'a mut [u8]) -> &'a str {
        if let Ok(data) = self.lock() {
            data.fmt(buf)
        } else {
            "<mutexpoison>"
        }
    }

    fn fmt_l<'a>(&self, buf: &'a mut [u8]) -> usize {
        if let Ok(data) = self.lock() {
            data.fmt_l(buf)
        } else {
            0
        }
    }
}

impl<T> Text for CloneMutable<T> where T: Text + PartialEq + Clone {
    fn fmt<'a>(&self, buf: &'a mut [u8]) -> &'a str {
        (*(*self)).fmt(buf)
    }

    fn fmt_l<'a>(&self, buf: &'a mut [u8]) -> usize {
        (*(*self)).fmt_l(buf)
    }
}

impl Text for (String, i64) {
    fn fmt_l<'a>(&self, buf: &'a mut [u8]) -> usize {
        use std::io::Write;
        let mut bw = std::io::BufWriter::new(buf);

        match write!(bw, "{} {}", self.0, self.1) {
            Ok(_) => bw.buffer().len(),
            _ => 0,
        }
    }
}

pub trait TextMutable: Text + Mutable { }
impl<T> TextMutable for T where T: Text + Mutable { }

pub enum Control {
    None,
    Rect,
    Button   { label: Box<dyn TextMutable> },
    Label    { label: Box<dyn TextMutable> },
    WichText { wt:    Box<WichText> },
    Entry    { entry: Box<Entry> },
    HexKnob  { knob:  Box<HexKnob> },
    HexGrid  { grid:  Box<HexGrid> },
}

fn bevel_points(pos: Rect, corner_offsets: (f32, f32, f32, f32)) -> [(f32, f32); 8] {
    let x     = pos.x;
    let y     = pos.y;
    let y_max = pos.y + pos.h;
    let x_max = pos.x + pos.w;
    let (o_tl, o_tr, o_bl, o_br) = corner_offsets;

    [
        (x,                        y + o_tl),
        (x + o_tl,                 y),
        (x_max - o_tr,             y),
        (x_max,                    y + o_tr),
        (x_max,                    y_max - o_br),
        (x_max - o_br,             y_max),
        (x + o_bl,                 y_max),
        (x,                        y_max - o_bl),
    ]
}

fn hex_points(pos: Rect, offset: f32) -> [(f32, f32); 6] {
    let ymid = (pos.y + pos.h / 2.0).round();
    [
        (pos.x,                    ymid),
        (pos.x + offset,           pos.y),
        (pos.x + (pos.w - offset), pos.y),
        (pos.x + pos.w,            ymid),
        (pos.x + (pos.w - offset), pos.y + pos.h),
        (pos.x + offset,           pos.y + pos.h),
    ]
}

impl Control {
    pub fn draw_frame(&mut self, _w: &Widget, _painter: &mut Painter) {
        match self {
            Control::Rect => { },
            Control::None => { },
            Control::Button   { .. } => { },
            Control::Label    { .. } => { },
            Control::WichText { .. } => { },
            Control::Entry    { .. } => { },
            Control::HexKnob  { .. } => { },
            Control::HexGrid  { .. } => { },
        }
    }

    pub fn can_hover(&self) -> bool {
        match self {
            Control::Rect => false,
            Control::None => false,
            Control::Label    { .. } => false,
            Control::Button   { .. } => true,
            Control::WichText { .. } => true,
            Control::Entry    { .. } => true,
            Control::HexKnob  { .. } => true,
            Control::HexGrid  { .. } => true,
        }
    }

    pub fn draw(&mut self, w: &Widget, redraw: bool, painter: &mut Painter) {
//        println!("     [draw widget id: {}]", w.id());

        let pos         = w.pos();
        let inner_pos   = w.inner_pos();
        let style       = w.style();
        let is_hovered  = w.is_hovered();
        let is_active   = w.is_active();

        //d// println!("DRAW {:?}", pos);

        let has_default_style =
            match self {
                Control::Rect            => { true },
                Control::Label    { .. } => { true },
                Control::Button   { .. } => { true },
                Control::WichText { .. } => { true },
                Control::Entry    { .. } => { true },
                Control::HexKnob  { .. } => { true },
                Control::HexGrid  { .. } => { true },
                Control::None            => { false },
            };


        let shadow_color =
            if is_active        { style.active_shadow_color }
            else if is_hovered  { style.hover_shadow_color }
            else                { style.shadow_color };

        let border_color =
            if is_active        { style.active_border_color }
            else if is_hovered  { style.hover_border_color }
            else                { style.border_color };

        let color =
            if is_active        { style.active_color }
            else if is_hovered  { style.hover_color }
            else                { style.color };

        let is_cached   = w.is_cached();
        let mut img_ref = w.take_cache_img();

        let orig_pos = pos;

        let inner_pos =
            if has_default_style {
                match style.border_style {
                    BorderStyle::Rect => {
                        inner_pos.shrink(style.border, style.border)
                    }
                    BorderStyle::Hex { offset } => {
                        inner_pos.shrink(offset * 2.0, style.border)
                    }
                    BorderStyle::Bevel { corner_offsets } => {
                        let max =
                            corner_offsets.0.max(
                                corner_offsets.1.max(
                                    corner_offsets.2.max(
                                        corner_offsets.3)));
                        inner_pos.shrink(style.border + max, style.border)
                    }
                }
            } else { inner_pos };

        let orig_inner_pos = inner_pos;

        if has_default_style {
            if    style.shadow_offs.0 > 0.1
               || style.shadow_offs.1 > 0.1
            {
                let (xo, yo) = style.shadow_offs;

                match style.border_style {
                    BorderStyle::Rect => {
                        painter.rect_fill(
                            shadow_color,
                            pos.x + xo,
                            pos.y + yo,
                            pos.w, pos.h);
                    }
                    BorderStyle::Hex { offset } => {
                        let points = hex_points(pos, offset);
                        painter.path_fill(
                            shadow_color,
                            &mut points.iter().copied().map(|p| (p.0 + xo, p.1 + yo)),
                            true);
                    }
                    BorderStyle::Bevel { corner_offsets } => {
                        let pos    = pos.shrink(style.border * 0.5, style.border * 0.5);
                        let points = bevel_points(pos, corner_offsets);
                        painter.path_fill(
                            shadow_color,
                            &mut points.iter().copied().map(|p| (p.0 + xo, p.1 + yo)),
                            true);
                    }
                }
            }
        }

        let (pos, inner_pos) =
            if is_cached {
                if redraw {
                    if let Some(img) = &img_ref {
                        if    img.w() != pos.w.floor()
                           || img.h() != pos.h.floor()
                        {
                            img_ref = Some(painter.new_image(pos.w, pos.h));
                        }
                    } else {
                        img_ref = Some(painter.new_image(pos.w, pos.h));
                    }
//                    img_ref = Some(painter.new_image(pos.w, pos.h));

                    //d// println!("      start img {} ({}:{})", w.id(), pos.w, pos.h);
                    painter.start_image(img_ref.as_ref().unwrap());
                    let new_pos = Rect::from(0.0, 0.0, pos.w, pos.h);
                    let (inner_xo, inner_yo) = (
                        inner_pos.x - pos.x,
                        inner_pos.y - pos.y
                    );
                    let new_inner_pos =
                        Rect::from(inner_xo, inner_yo, inner_pos.w, inner_pos.h);
                    (new_pos, new_inner_pos)
                } else {
                    (pos, inner_pos)
                }
            } else {
                (pos, inner_pos)
            };

        if !is_cached || redraw {
            if has_default_style {
                match style.border_style {
                    BorderStyle::Rect => {
                        if style.border > 0.1 {
                            painter.rect_fill(
                                border_color, pos.x, pos.y, pos.w, pos.h);
                        }

                        painter.rect_fill(
                            style.bg_color,
                            inner_pos.x, inner_pos.y,
                            inner_pos.w, inner_pos.h);
                    }
                    BorderStyle::Bevel { corner_offsets } => {
                        let pos    = pos.shrink(style.border * 0.5, style.border * 0.5);
                        let points = bevel_points(pos, corner_offsets);
                        painter.path_fill(
                            style.bg_color,
                            &mut points.iter().copied(),
                            true);
                        painter.path_stroke(
                            style.border, border_color,
                            &mut points.iter().copied(),
                            true);
                    }
                    BorderStyle::Hex { offset } => {
                        let points = hex_points(pos, offset);
                        painter.path_fill(
                            border_color,
                            &mut points.iter().copied(),
                            true);
                        let points2 = [
                            (points[0].0 + style.border, points[0].1),
                            (points[1].0 + style.border, points[1].1 + style.border),
                            (points[2].0 - style.border, points[2].1 + style.border),
                            (points[3].0 - style.border, points[3].1),
                            (points[4].0 - style.border, points[4].1 - style.border),
                            (points[5].0 + style.border, points[5].1 - style.border),
                        ];
                        painter.path_fill(
                            style.bg_color,
                            &mut points2.iter().copied(),
                            true);
                    }
                }
            }

            // We need to apply also padding to the original inner position, or else
            // the widget will compensate the padding away by comparing these two
            // positions!
            let orig_inner_pos = style.apply_padding(orig_inner_pos);
            let inner_pos      = style.apply_padding(inner_pos);

            match self {
                Control::Rect => { },
                Control::None => { },
                Control::Button { label } | Control::Label { label } => {
                    let mut buf : [u8; 128] = [0; 128];
                    let s = label.fmt(&mut buf[..]);

                    let align =
                        match style.text_align {
                            style::Align::Left   => -1,
                            style::Align::Center => 0,
                            style::Align::Right  => 1,
                        };

                    let fs =
                        painter::calc_font_size_from_text(
                            painter, s, style.font_size, inner_pos.w);

                    let mut dbg = painter::LblDebugTag::from_id(w.id());
                    dbg.set_offs(
                        (orig_inner_pos.x - inner_pos.x,
                         orig_inner_pos.y - inner_pos.y));

                    painter.label(
                        fs,
                        align,
                        color,
                        inner_pos.x,
                        inner_pos.y,
                        inner_pos.w,
                        inner_pos.h,
                        s,
                        dbg.source("label"));
                },
                Control::Entry { entry } => {
                    entry.draw(w, &style, inner_pos, orig_inner_pos, painter);
                },
                Control::WichText { wt } => {
                    wt.draw(w, &style, inner_pos, orig_inner_pos, painter);
                },
                Control::HexKnob { knob } => {
                    knob.draw(w, &style, inner_pos, orig_inner_pos, painter);
                },
                Control::HexGrid { grid } => {
                    grid.draw(w, &style, inner_pos, orig_inner_pos, painter);
                },
            }
        }

        if let Some(img_ref) = img_ref {
            if is_cached {
                if redraw {
                    //d// println!("      finish img {}", wid_id);
                    painter.finish_image();
                }
            }

            painter.draw_image(&img_ref, orig_pos.x, orig_pos.y);
            //d// println!("      give img {}", wid_id);
            w.give_cache_img(img_ref);
        }
    }

    pub fn get_generation(&mut self) -> u64 {
        match self {
            Control::None => 0,
            Control::Rect => 0,
            Control::Button   { label } => label.get_generation(),
            Control::Label    { label } => label.get_generation(),
            Control::WichText { wt }    => wt.data().get_generation(),
            Control::Entry    { entry } => entry.get_generation(),
            Control::HexKnob  { knob }  => knob.get_generation(),
            Control::HexGrid  { grid }  => grid.get_generation(),
        }
    }

    pub fn handle(
        &mut self,
        w: &Widget,
        event: &InputEvent,
        out_events: &mut Vec<(usize, Event)>)
    {
        let is_hovered = w.is_hovered();

        match self {
            Control::Rect  => { },
            Control::None  => { },
            Control::Label { .. } => { },
            Control::Button { .. } => {
                match event {
                    InputEvent::MouseButtonPressed(_button) => {
                        if is_hovered { w.activate(); }
                    },
                    InputEvent::MouseButtonReleased(button) => {
                        if w.is_active() {
                            out_events.push((w.id(), Event {
                                name: "click".to_string(),
                                data: EvPayload::Button(*button),
                            }));
                            w.deactivate();
                        }
                    },
                    _ => {},
                }
            },
            Control::Entry { entry } => {
                entry.handle(w, event, out_events);
            },
            Control::HexKnob { knob } => {
                knob.handle(w, event, out_events);
            },
            Control::HexGrid { grid } => {
                grid.handle(w, event, out_events);
            },
            Control::WichText { wt } => {
                wt.handle(w, event, out_events);
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct UINotifier {
    pub tree_changed:   bool,
    pub layout_changed: bool,
    pub hover_id:       usize,
    pub mouse_pos:      (f32, f32),
    pub redraw:         HashSet<usize>,
    pub active:         Option<usize>,
}

impl UINotifier {
    pub fn new_ref() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            tree_changed:   false,
            layout_changed: false,
            hover_id:       0,
            mouse_pos:      (0.0, 0.0),
            redraw:         HashSet::new(),
            active:         None,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct UINotifierRef(Rc<RefCell<UINotifier>>);

impl UINotifierRef {
    pub fn new() -> Self {
        Self(UINotifier::new_ref())
    }

    pub fn swap_redraw(&self, cur_redraw: &mut HashSet<usize>) {
        std::mem::swap(&mut self.0.borrow_mut().redraw, cur_redraw);
    }

    pub fn is_tree_changed(&self) -> bool {
        let r = self.0.borrow_mut();
        r.tree_changed
    }

    pub fn is_layout_changed(&self) -> bool {
        let r = self.0.borrow_mut();
        r.layout_changed
    }

    pub fn set_layout_changed(&self) {
        let mut r = self.0.borrow_mut();
        r.layout_changed = true;
    }

    pub fn set_tree_changed(&self) {
        let mut r = self.0.borrow_mut();
        r.tree_changed = true;
    }

    pub fn reset_layout_changed(&self) {
        let mut r = self.0.borrow_mut();
        r.layout_changed = false;
    }

    pub fn reset_tree_changed(&self) {
        let mut r = self.0.borrow_mut();
        r.tree_changed = false;
    }


    pub fn mouse_pos(&self) -> (f32, f32) {
        let r = self.0.borrow_mut();
        r.mouse_pos
    }

    pub fn set_mouse_pos(&self, pos: (f32, f32)) {
        let mut r = self.0.borrow_mut();
        r.mouse_pos = pos;
    }

    pub fn set_hover(&self, id: usize) {
        let mut r = self.0.borrow_mut();
        r.hover_id = id;
    }

    pub fn hover(&self) -> usize {
        let r = self.0.borrow_mut();
        r.hover_id
    }

    pub fn redraw(&self, id: usize) {
        let mut r = self.0.borrow_mut();
        r.redraw.insert(id);
    }

    pub fn need_redraw(&self) -> bool {
        let r = self.0.borrow_mut();
        !r.redraw.is_empty()
    }

    pub fn clear_redraw(&self) {
        let mut r = self.0.borrow_mut();
        r.redraw.clear()
    }

    pub fn activate(&self, id: usize) {
        let active = {
            let mut r = self.0.borrow_mut();
            r.active.take()
        };

        if let Some(old_active_id) = active {
            if old_active_id != id {
                self.redraw(old_active_id);
            }
        }

        self.0.borrow_mut().active = Some(id);

        self.redraw(id);
    }

    pub fn deactivate(&self, id: usize) {
        let active = {
            let mut r = self.0.borrow_mut();
            r.active.take()
        };

        if let Some(old_active_id) = active {
            if old_active_id == id {
                self.redraw(old_active_id);
            }
        }
    }

    pub fn active(&self) -> Option<usize> {
        let r = self.0.borrow_mut();
        r.active
    }
}


#[derive(Debug, Clone)]
pub struct Event {
    pub name: String,
    pub data: EvPayload,
}

#[derive(Debug, Clone)]
pub enum EvPayload {
    None,
    WichTextCommand { line: usize, frag: usize, cmd: String },
    HexGridClick {
        x: usize,
        y: usize,
        button: MButton,
    },
    HexGridDrag {
        x_src: usize,
        y_src: usize,
        x_dst: usize,
        y_dst: usize,
        button: MButton,
    },
    Button(MButton),
    Text(String),
    Pos { x: f32, y: f32 },
}

pub struct EventCore {
    callbacks:
        std::collections::HashMap<
            String,
            Option<Vec<Box<dyn FnMut(&mut std::any::Any, Widget, &Event)>>>>,
}

impl EventCore {
    pub fn new() -> Self {
        Self {
            callbacks: std::collections::HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.callbacks.clear();
    }

    pub fn reg(&mut self, ev_name: &str, cb: Box<dyn FnMut(&mut std::any::Any, Widget, &Event)>) {
        if let Some(cbs) = self.callbacks.get_mut(ev_name) {
            if let Some(cbs) = cbs { cbs.push(cb); }
        } else {
            self.callbacks.insert(ev_name.to_string(), Some(vec![cb]));
        }
    }

    pub fn call(&mut self, ctx: &mut std::any::Any, ev: &Event, widget: &Widget) {
        if let Some(cbs) = self.callbacks.get_mut(&ev.name) {
            if let Some(cbs) = cbs {
                for cb in cbs {
                    (*cb)(ctx, widget.clone(), ev);
                }
            }
        }
    }
}

pub trait WindowUI {
    fn pre_frame(&mut self);
    fn post_frame(&mut self);
    fn needs_redraw(&mut self) -> bool;
    fn is_active(&mut self) -> bool;
    fn handle_input_event(&mut self, event: InputEvent);
    fn draw(&mut self, painter: &mut Painter);
    fn draw_frame(&mut self, painter: &mut Painter);
    fn set_window_size(&mut self, w: f32, h: f32);
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    MousePosition(f32, f32),
    MouseButtonPressed(MButton),
    MouseButtonReleased(MButton),
    MouseWheel(f32),
    KeyPressed(KeyboardEvent),
    KeyReleased(KeyboardEvent),
    WindowClose,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MButton {
    Left,
    Right,
    Middle,
}
