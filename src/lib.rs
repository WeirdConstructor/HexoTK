// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

// trait for WidgetType
// - provides active zones
// - can set "input modes" which forward interaction to the widget
// - can draw themself
// - get a reference to their state data, which is stored externally
//   => need to define how and how to interact with client code!
// => Think if container types can be implemented this way.
mod widgets;
mod window;
mod constants;
mod femtovg_painter;

use keyboard_types::{Key, KeyboardEvent};

pub use window::open_window;

use std::fmt::Debug;

pub struct WidgetData {
    id:   usize,
    data: Box<dyn std::any::Any>,
}

impl WidgetData {
    fn with<T>(&mut self, f: &dyn FnOnce(&mut T)) {
        // TODO
    }
}

struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum MButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone)]
pub enum ActiveZone {
    ValueDrag  { widget_id: usize },
    ValueInput { widget_id: usize },
    Click      { widget_id: usize },
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    MousePosition(f64, f64),
    MouseButtonPressed(MButton),
    MouseButtonReleased(MButton),
    MouseWheel(f64),
    KeyPressed(KeyboardEvent),
    KeyReleased(KeyboardEvent),
    WindowClose,
}

#[derive(Debug, Clone)]
pub enum WidgetEvent {
    Clicked,
}

pub trait Painter {
    fn path_fill(&mut self, color: (f64, f64, f64), segments: &mut dyn std::iter::Iterator<Item = (f64, f64)>, closed: bool);
    fn path_stroke(&mut self, width: f64, color: (f64, f64, f64), segments: &mut dyn std::iter::Iterator<Item = (f64, f64)>, closed: bool);
    fn arc_stroke(&mut self, width: f64, color: (f64, f64, f64), radius: f64, from_rad: f64, to_rad: f64, x: f64, y: f64);
    fn rect_fill(&mut self, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64);
    fn rect_stroke(&mut self, width: f64, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64);
    fn label(&mut self, size: f64, align: i8, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64, text: &str);
    fn label_mono(&mut self, size: f64, align: i8, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64, text: &str);
    fn font_height(&mut self, size: f32, mono: bool) -> f32;
}

pub trait WindowUI {
    fn pre_frame(&mut self);
    fn post_frame(&mut self);
    fn needs_redraw(&mut self) -> bool;
    fn is_active(&mut self) -> bool;
    fn handle_input_event(&mut self, event: InputEvent);
    fn draw(&mut self, painter: &mut dyn Painter);
    fn set_window_size(&mut self, w: f64, h: f64);
}

trait WidgetUI: Painter {
    fn define_active_zone_rect(&self, az: ActiveZone, x: f64, y: f64, w: f64, h: f64);
    fn add_widget_type(&self, w_type_id: usize, wtype: Box<dyn WidgetType>);
    fn grab_focus(&self);
    fn release_focus(&self);
    fn emit_event(&self, event: UIEvent);
}

enum UIEvent {
    ValueDragStart,
    ValueDrag { steps: f64 },
    ValueDragEnd,
    EnteredValue { val: String },
    Click { button: MButton, x: f64, y: f64 },
    Hover { x: f64, y: f64 },
}

trait WidgetType: Debug {
    fn draw(&self, ui: &dyn WidgetUI, data: &mut WidgetData, pos: Rect);
    fn size(&self, ui: &dyn WidgetUI, data: &mut WidgetData);
    fn event(&self, ui: &dyn WidgetUI, data: &mut WidgetData, ev: UIEvent);
}
