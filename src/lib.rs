// trait for WidgetType
// - provides active zones
// - can set "input modes" which forward interaction to the widget
// - can draw themself
// - get a reference to their state data, which is stored externally
//   => need to define how and how to interact with client code!
// => Think if container types can be implemented this way.
mod widgets;
mod window;

use keyboard_types::{Key, KeyboardEvent};

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
pub enum MouseButton {
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
pub enum WindowEvent {
    MousePosition(f64, f64),
    MouseButtonPressed(MouseButton),
    MouseButtonReleased(MouseButton),
    MouseWheel(f64),
    KeyPressed(KeyboardEvent),
    KeyReleased(KeyboardEvent),
    WindowClose,
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

trait WindowUI<EV: Copy + Clone + Debug> {
    fn pre_frame(&mut self);
    fn post_frame(&mut self);
    fn needs_redraw(&mut self) -> bool;
    fn handle_window_event(&mut self, event: WindowEvent);
    fn draw(&mut self, painter: &mut dyn Painter);
    fn set_window_size(&mut self, w: f64, h: f64);
}

trait WidgetUI<EV: Copy + Clone + Debug>: Painter {
    fn define_active_zone_rect(&self, az: ActiveZone, x: f64, y: f64, w: f64, h: f64);
    fn add_widget_type(&self, w_type_id: usize, wtype: Box<dyn WidgetType<EV>>);
    fn grab_focus(&self);
    fn release_focus(&self);
    fn emit_event(&self, event: EV);
}

enum UIEvent {
    ValueDragStart,
    ValueDrag { steps: f64 },
    ValueDragEnd,
    Click { button: MouseButton, x: f64, y: f64 },
    Hover { x: f64, y: f64 },
}

trait WidgetType<EV: Copy + Clone + Debug>: Debug {
    fn draw(&self, ui: &dyn WidgetUI<EV>, data: &mut WidgetData, pos: Rect);
    fn size(&self, ui: &dyn WidgetUI<EV>, data: &mut WidgetData);
    fn event(&self, ui: &dyn WidgetUI<EV>, data: &mut WidgetData, ev: UIEvent);
}
