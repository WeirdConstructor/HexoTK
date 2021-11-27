use crate::{
    InputEvent, Painter, widget_draw,
    widget_walk, UINotifier, Rect
};
use crate::WindowUI;
use crate::Widget;
use std::rc::{Weak, Rc};
use std::cell::RefCell;

pub struct UI {
    win_w:      f32,
    win_h:      f32,
    root:       Rc<RefCell<Widget>>,
    widgets:    Option<Vec<Weak<RefCell<Widget>>>>,
    notifier:   Rc<RefCell<UINotifier>>,
    zones:      Option<Vec<(Rect, usize, Weak<RefCell<Widget>>)>>,
}

impl UI {
    pub fn new() -> Self {
        Self {
            win_h:    0.0,
            win_w:    0.0,
            root:     Widget::new_ref(),
            widgets:  Some(vec![]),
            notifier: UINotifier::new_ref(),
            zones:    Some(vec![]),
        }
    }

    pub fn set_root(&mut self, root: Rc<RefCell<Widget>>) {
        self.root = root;
        self.refresh_widget_list();
        self.on_tree_changed();
    }

    pub fn on_tree_changed(&mut self) {
        let notifier = self.notifier.clone();

        self.for_each_widget_ref(|wid, id| {
            wid.set_notifier(notifier.clone(), id);
            notifier.borrow_mut().redraw.push(id);
        });
    }

    pub fn on_layout_changed(&mut self) {
        let zones = self.zones.take();

        if let Some(mut zones) = zones {
            zones.clear();

            self.for_each_widget(|wid, id| {
                zones.push((wid.borrow().pos(), id, Rc::downgrade(&wid)));
            });

            self.zones = Some(zones);
        }
    }

    pub fn for_each_widget<F: FnMut(Rc<RefCell<Widget>>, usize)>(&self, mut f: F) {
        if let Some(widgets) = &self.widgets {
            for (id, w) in widgets.iter().enumerate() {
                if let Some(w) = w.upgrade() {
                    f(w, id);
                }
            }
        }
    }

    pub fn for_each_widget_ref<F: FnMut(&mut Widget, usize)>(&self, mut f: F) {
        if let Some(widgets) = &self.widgets {
            for (id, w) in widgets.iter().enumerate() {
                if let Some(w) = w.upgrade() {
                    f(&mut w.borrow_mut(), id);
                }
            }
        }
    }
}

impl UI {
    fn refresh_widget_list(&mut self) {
        let wids = self.widgets.take();
        if let Some(mut wids) = wids {
            wids.clear();

            widget_walk(&self.root, |wid, parent| {
                if let Some(parent) = parent {
                    wid.borrow_mut().set_parent(parent);
                }
                wids.push(Rc::downgrade(wid));
            });

            self.widgets = Some(wids);
        }
    }
}

impl WindowUI for UI {
    fn pre_frame(&mut self) { }
    fn post_frame(&mut self) {
        let notifier = self.notifier.clone();

        if notifier.borrow().tree_changed {
            self.on_tree_changed();
            notifier.borrow_mut().tree_changed = false;
        }

        if notifier.borrow().layout_changed {
            self.on_layout_changed();
            notifier.borrow_mut().layout_changed = false;
        }
    }

    fn needs_redraw(&mut self) -> bool {
        let notifier = self.notifier.clone();
        let mut not = notifier.borrow_mut();
        !not.redraw.is_empty()
    }

    fn is_active(&mut self) -> bool { true }

    fn handle_input_event(&mut self, event: InputEvent) {
        let notifier = self.notifier.clone();

        match &event {
            InputEvent::MousePosition(x, y) => {
                let mut hover_id = 0;
                if let Some(zones) = &self.zones {
                    for (pos, id, wid) in zones.iter() {
                        if pos.is_inside(*x, *y) {
                            hover_id = *id;
                        }
                    }
                }

                let mut not = notifier.borrow_mut();
                not.hover_id = hover_id;
                not.mouse_pos = (*x, *y);
            },
            _ => {},
        }

        self.for_each_widget(|wid, _id| {
            let ctrl = wid.borrow_mut().ctrl.take();

            if let Some(mut ctrl) = ctrl {
                ctrl.handle(&wid, &event);

                wid.borrow_mut().ctrl = Some(ctrl);
            }
        });
    }

    fn draw(&mut self, painter: &mut Painter) {
        let notifier = self.notifier.clone();
        let mut not = notifier.borrow_mut();

        println!("DRAW");
        widget_draw(&self.root, painter);
        not.redraw.clear();
    }

    fn set_window_size(&mut self, w: f32, h: f32) {
        self.win_w = w;
        self.win_h = h;
        self.notifier.borrow_mut().redraw.push(0);
    }
}
