// trait for WidgetType
// - provides active zones
// - can set "input modes" which forward interaction to the widget
// - can draw themself
// - get a reference to their state data, which is stored externally
//   => need to define how and how to interact with client code!
// => Think if container types can be implemented this way.
mod widgets;

use std::fmt::Debug;

trait WidgetData: std::any::Any {}

struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

enum UIEvent {
    MousePressed,
    MouseReleased,
    Keyboard,
    DragValue { steps: f64 },
}

trait Painter {
}

trait UI<EV: Copy + Clone + Debug>: Painter {
    fn define_active_zone(&self, data: &dyn WidgetData, type_id: usize, x: f64, y: f64, w: f64, h: f64);
    fn add_widget_type(&self, w_type_id: usize, wtype: Box<dyn WidgetType<EV>>);
    fn grab_focus(&self);
    fn release_focus(&self);
    fn emit_event(&self, event: EV);
}

trait WidgetType<EV: Copy + Clone + Debug>: Debug {
    fn draw(&self, ui: &dyn UI<EV>, data: &dyn WidgetData, pos: Rect);
    fn size(&self, ui: &dyn UI<EV>, data: &dyn WidgetData);
    fn event(&self, ui: &dyn UI<EV>, ev: UIEvent);
}
