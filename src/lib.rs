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
mod window;
mod rect;
mod painter;
//#[allow(clippy::type_complexity)]
//#[allow(clippy::too_many_arguments)]
//mod femtovg_painter;

use std::rc::Rc;
use keyboard_types::{KeyboardEvent, Key};
pub use window::open_window;
pub use rect::Rect;
use painter::Painter;

use std::fmt::Debug;

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
