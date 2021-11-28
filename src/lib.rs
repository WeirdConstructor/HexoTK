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

use keyboard_types::{KeyboardEvent, Key};
pub use window::open_window;
pub use rect::Rect;
use painter::Painter;
pub use widget::Widget;
use widget::{widget_draw, widget_walk};
pub use ui::UI;

use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum Control {
    None,
    Rect,
    Button,
}

impl Control {
    pub fn draw(&mut self, widget: &Rc<RefCell<Widget>>, painter: &mut Painter) {
        let w           = widget.borrow();
        let pos         = w.pos();
        let style       = w.style();
        let is_hovered  = w.is_hovered();
        let is_active   = w.is_active();

        println!("DRAW {:?}", pos);

        let has_default_style =
            match self {
                Control::Rect   => { true },
                Control::Button => { true },
                Control::None   => { false },
            };


        let shadow_color =
            if is_active        { style.active_shadow_color }
            else if is_hovered  { style.hover_shadow_color }
            else                { style.shadow_color };

        let border_color =
            if is_active        { style.active_border_color }
            else if is_hovered  { style.hover_border_color }
            else                { style.border_color };

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

            painter.rect_fill(style.bg_color, pos.x, pos.y, pos.w, pos.h);
        }

        match self {
            Control::Rect => { },
            Control::Button => { },
            Control::None => { },
        }
    }

    pub fn handle(
        &mut self,
        widget: &Rc<RefCell<Widget>>,
        event: &InputEvent,
        out_events: &mut Vec<(usize, Event)>)
    {
        let w           = widget.borrow();
        let pos         = w.pos();
        let is_hovered  = w.is_hovered();

        match self {
            Control::Rect => { },
            Control::Button => {
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
    pub redraw:         Vec<usize>,
    pub active:         Option<usize>,
}

impl UINotifier {
    pub fn new_ref() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            tree_changed:   false,
            layout_changed: false,
            hover_id:       0,
            mouse_pos:      (0.0, 0.0),
            redraw:         vec![],
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
        r.redraw.push(id)
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
            Option<Vec<Box<dyn Fn(Rc<RefCell<Widget>>, &Event)>>>>,
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

    pub fn reg(&mut self, ev_name: &str, cb: Box<dyn Fn(Rc<RefCell<Widget>>, &Event)>) {
        if let Some(cbs) = self.callbacks.get_mut(ev_name) {
            if let Some(cbs) = cbs { cbs.push(cb); }
        } else {
            self.callbacks.insert(ev_name.to_string(), Some(vec![cb]));
        }
    }

    pub fn call(&mut self, ev: &Event, widget: &Rc<RefCell<Widget>>) {
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
