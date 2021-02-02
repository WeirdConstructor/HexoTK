use super::*;

#[derive(Debug)]
struct Container {
    w: f64,
}

impl<EV: Copy + Clone + Debug> WidgetType<EV> for Container {
    fn draw(&self, ui: &dyn WidgetUI<EV>, data: &dyn WidgetData, pos: Rect) {}
    fn size(&self, ui: &dyn WidgetUI<EV>, data: &dyn WidgetData) {}
    fn event(&self, ui: &dyn WidgetUI<EV>, ev: UIEvent) {}
}
