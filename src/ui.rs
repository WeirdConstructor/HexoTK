use crate::{InputEvent, Painter, widget_draw, widget_handle, widget_walk};
use crate::WindowUI;
use crate::Widget;
use std::rc::{Weak, Rc};
use std::cell::RefCell;

pub struct UI {
    win_w:      f32,
    win_h:      f32,
    root:       Rc<RefCell<Widget>>,
    widgets:    Option<Vec<Weak<RefCell<Widget>>>>,
}

impl UI {
    pub fn new() -> Self {
        Self {
            win_h: 0.0,
            win_w: 0.0,
            root: Widget::new_ref(),
            widgets: Some(vec![]),
        }
    }

    pub fn set_root(&mut self, root: Rc<RefCell<Widget>>) {
        self.root = root;
    }
}

impl WindowUI for UI {
    fn pre_frame(&mut self) { }
    fn post_frame(&mut self) { }
    fn needs_redraw(&mut self) -> bool { true }
    fn is_active(&mut self) -> bool { true }

    fn post_relayout(&mut self) {
        let wids = self.widgets.take();
        if let Some(mut wids) = wids {
            wids.clear();

            widget_walk(&self.root, |wid| {
                wids.push(Rc::downgrade(wid));
            });

            self.widgets = Some(wids);
        }
    }

    fn handle_input_event(&mut self, event: InputEvent) {
        widget_handle(&self.root, &event);
    }

    fn draw(&mut self, painter: &mut Painter) {
        widget_draw(&self.root, painter);
    }

    fn set_window_size(&mut self, w: f32, h: f32) {
        self.win_w = w;
        self.win_h = h;
    }
}
