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
    Rect,
}

impl Control {
    pub fn draw(&mut self, widget: &Rc<RefCell<Widget>>, painter: &mut Painter) {
        let w       = widget.borrow();
        let pos     = w.pos();
        let style   = w.style();

        if style.border > 0.1 {
            painter.rect_fill(
                style.border_color,
                pos.x - style.border,
                pos.y - style.border,
                pos.w + style.border * 2.0,
                pos.h + style.border * 2.0);
        }

        painter.rect_fill(style.bg_color, pos.x, pos.y, pos.w, pos.h);

        println!("DRAW {:?}", pos);

        match self {
            Control::Rect => { },
        }
    }
    pub fn handle(&mut self, widget: &Rc<RefCell<Widget>>, event: &InputEvent) {
    }
}

#[derive(Debug, Clone)]
pub struct UINotifier {
    pub tree_changed:   bool,
    pub layout_changed: bool,
    pub hover_id:       usize,
    pub mouse_pos:      (f32, f32),
    pub redraw:         Vec<usize>,
}

impl UINotifier {
    pub fn new_ref() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            tree_changed:   false,
            layout_changed: false,
            hover_id:       0,
            mouse_pos:      (0.0, 0.0),
            redraw:         vec![],
        }))
    }
}


#[derive(Debug, Clone)]
pub struct Event {
    pub name: String,
    pub data: EvPayload,
}

#[derive(Debug, Clone)]
pub enum EvPayload {
}

pub struct EventCore {
    callbacks:
        std::collections::HashMap<
            String,
            Option<Vec<Box<dyn Fn(Rc<RefCell<Widget>>, Event)>>>>,
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

    pub fn reg(&mut self, ev_name: &str, cb: Box<dyn Fn(Rc<RefCell<Widget>>, Event)>) {
        if let Some(cbs) = self.callbacks.get_mut(ev_name) {
            if let Some(cbs) = cbs { cbs.push(cb); }
        } else {
            self.callbacks.insert(ev_name.to_string(), Some(vec![cb]));
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
