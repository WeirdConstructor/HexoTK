// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

//pub mod widgets;
//pub mod components;
//pub mod constants;
//
//mod driver;
//mod ui;
//#[allow(clippy::type_complexity)]
mod widget;
mod ui;
mod window;
mod rect;
mod painter;
mod style;
//#[allow(clippy::type_complexity)]
//#[allow(clippy::too_many_arguments)]
//mod femtovg_painter;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashSet;

use keyboard_types::{KeyboardEvent, Key};
pub use window::open_window;
pub use rect::Rect;
use painter::Painter;
pub use widget::Widget;
use widget::{widget_draw, widget_draw_frame, widget_walk};
pub use ui::UI;
pub use style::Style;

use std::fmt::Debug;

pub trait Mutable {
    fn check_change(&mut self) -> bool;
}

pub trait Text {
    fn fmt<'a>(&self, buf: &'a mut [u8]) -> &'a str {
        let l = self.fmt_l(buf);

        std::str::from_utf8(&buf[0..l]).unwrap_or("")
    }

    fn fmt_l<'a>(&self, buf: &'a mut [u8]) -> usize { 0 }
}

pub struct CloneMutable<T> where T: PartialEq + Clone {
    old: Option<T>,
    cur: T,
}

impl<T> CloneMutable<T> where T: PartialEq + Clone {
    pub fn new(cur: T) -> Self {
        Self {
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
    fn check_change(&mut self) -> bool {
        let change =
            self.old.as_ref()
                .map(|o| *o != self.cur)
                .unwrap_or(true);
        self.old = Some(self.cur.clone());
        change
    }
}

impl Mutable for String {
    fn check_change(&mut self) -> bool { false }
}

impl<T> Mutable for Rc<RefCell<T>> where T: Mutable {
    fn check_change(&mut self) -> bool { self.borrow_mut().check_change() }
}

impl<T> Mutable for std::sync::Arc<std::sync::Mutex<T>> where T: Mutable {
    fn check_change(&mut self) -> bool {
        if let Ok(mut data) = self.lock() {
            data.check_change()
        } else {
            false
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
        self.fmt(buf)
    }

    fn fmt_l<'a>(&self, buf: &'a mut [u8]) -> usize {
        self.fmt_l(buf)
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
    Button { label: Box<dyn TextMutable> }
}

impl Control {
    pub fn draw_frame(&mut self, w: &Widget, painter: &mut Painter) {
    }

    pub fn draw(&mut self, w: &Widget, redraw: bool, painter: &mut Painter) {
//        println!("     [draw widget id: {}]", w.id());

        let pos         = w.pos();
        let style       = w.style();
        let is_hovered  = w.is_hovered();
        let is_active   = w.is_active();
        let wid_id      = w.id();

        //d// println!("DRAW {:?}", pos);

        let has_default_style =
            match self {
                Control::Rect          => { true },
                Control::Button { .. } => { true },
                Control::None          => { false },
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

        if has_default_style {
            if    style.shadow_offs.0 > 0.1
               || style.shadow_offs.1 > 0.1
            {
                let (xo, yo) = style.shadow_offs;
                painter.rect_fill(
                    shadow_color,
                    pos.x + style.border + xo,
                    pos.y + style.border + yo,
                    pos.w, pos.h);
            }

            if style.border > 0.1 {
                painter.rect_fill(
                    border_color,
                    pos.x - style.border,
                    pos.y - style.border,
                    pos.w + style.border * 2.0,
                    pos.h + style.border * 2.0);
            }
        }

        let is_cached   = w.is_cached();
        let mut img_ref = w.take_cache_img();

        let orig_pos = pos;

        let pos =
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

                    println!("      start img {}", wid_id);
                    painter.start_image(img_ref.as_ref().unwrap());
                    Rect { x: 0.0, y: 0.0, w: pos.w, h: pos.h }
                } else {
                    pos
                }
            } else {
                pos
            };

        if !is_cached || redraw {
            if has_default_style {
                painter.rect_fill(style.bg_color, pos.x, pos.y, pos.w, pos.h);
            }

            match self {
                Control::Rect => { },
                Control::None => { },
                Control::Button { label } => {
                    let mut buf : [u8; 128] = [0; 128];
                    let s = label.fmt(&mut buf[..]);

                    painter.label(
                        style.font_size,
                        0,
                        color,
                        pos.x,
                        pos.y,
                        pos.w,
                        pos.h,
                        s);
                },
            }
        }

        if let Some(img_ref) = img_ref {
            if is_cached {
                if redraw {
                    println!("      finish img {}", wid_id);
                    painter.finish_image();
                }
            }

            painter.draw_image(&img_ref, orig_pos.x, orig_pos.y);
            println!("      give img {}", wid_id);
            w.give_cache_img(img_ref);
        }
    }

    pub fn check_change(&mut self) -> bool {
        match self {
            Control::None => false,
            Control::Rect => false,
            Control::Button { label } => label.check_change(),
        }
    }

    pub fn handle(
        &mut self,
        w: &Widget,
        event: &InputEvent,
        out_events: &mut Vec<(usize, Event)>)
    {
        let pos         = w.pos();
        let is_hovered  = w.is_hovered();

        match self {
            Control::Rect => { },
            Control::Button { .. } => {
                match event {
                    InputEvent::MouseButtonPressed(b) => {
                        if is_hovered { w.activate(); }
                    },
                    InputEvent::MouseButtonReleased(b) => {
                        if w.is_active() {
                            out_events.push((w.id(), Event {
                                name: "click".to_string(),
                                data: EvPayload::None,
                            }));
                            w.deactivate();
                        }
                    },
                    _ => {},
                }
            },
            Control::None => { },
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
        let mut r = self.0.borrow_mut();
        r.tree_changed
    }

    pub fn is_layout_changed(&self) -> bool {
        let mut r = self.0.borrow_mut();
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
        let mut r = self.0.borrow_mut();
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
        let mut r = self.0.borrow_mut();
        r.hover_id
    }

    pub fn redraw(&self, id: usize) {
        let mut r = self.0.borrow_mut();
        r.redraw.insert(id);
    }

    pub fn need_redraw(&self) -> bool {
        let mut r = self.0.borrow_mut();
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
        } else {
            let mut r = self.0.borrow_mut();
            r.active = Some(id);
        }

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
        let mut r = self.0.borrow_mut();
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
    None
}

pub struct EventCore {
    callbacks:
        std::collections::HashMap<
            String,
            Option<Vec<Box<dyn FnMut(Widget, &Event)>>>>,
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

    pub fn reg(&mut self, ev_name: &str, cb: Box<dyn FnMut(Widget, &Event)>) {
        if let Some(cbs) = self.callbacks.get_mut(ev_name) {
            if let Some(cbs) = cbs { cbs.push(cb); }
        } else {
            self.callbacks.insert(ev_name.to_string(), Some(vec![cb]));
        }
    }

    pub fn call(&mut self, ev: &Event, widget: &Widget) {
        if let Some(cbs) = self.callbacks.get_mut(&ev.name) {
            if let Some(cbs) = cbs {
                for cb in cbs {
                    (*cb)(widget.clone(), ev);
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
