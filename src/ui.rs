use crate::{
    InputEvent, Painter, widget_draw,
    widget_draw_frame,
    widget_walk, UINotifierRef, Rect,
    Event, Style
};
use crate::WindowUI;
use crate::Widget;
use crate::widget::WidgetImpl;
use std::rc::{Weak, Rc};
use std::cell::RefCell;
use std::collections::HashSet;

pub struct UI {
    win_w:              f32,
    win_h:              f32,
    root:               Widget,
    widgets:            Option<Vec<Weak<RefCell<WidgetImpl>>>>,
    notifier:           UINotifierRef,
    zones:              Option<Vec<(Rect, usize)>>,
    cur_redraw:         HashSet<usize>,
    cur_parent_lookup:  Vec<usize>,
}

impl UI {
    pub fn new() -> Self {
        Self {
            win_h:              0.0,
            win_w:              0.0,
            root:               Widget::new(Rc::new(Style::new())),
            widgets:            Some(vec![]),
            notifier:           UINotifierRef::new(),
            zones:              Some(vec![]),
            cur_redraw:         HashSet::new(),
            cur_parent_lookup:  vec![],
        }
    }

    pub fn set_root(&mut self, root: Widget) {
        self.root = root;
        self.on_tree_changed();
    }

    pub fn on_tree_changed(&mut self) {
        println!("tree changed");
        self.refresh_widget_list();

        let notifier = self.notifier.clone();

        self.for_each_widget_impl(|wid, id| {
            wid.set_notifier(notifier.clone(), id);
            notifier.redraw(id);
        });

        self.notifier.reset_tree_changed();

        self.on_layout_changed();
    }

    pub fn on_layout_changed(&mut self) {
        println!("layout changed");
        let zones = self.zones.take();

        if let Some(mut zones) = zones {
            zones.clear();

            self.for_each_widget(|wid, id| {
                zones.push((wid.pos(), id));
            });

            self.zones = Some(zones);
        }

        self.notifier.reset_layout_changed();
    }

    pub fn for_each_widget<F: FnMut(Widget, usize)>(&self, mut f: F) {
        if let Some(widgets) = &self.widgets {
            for (id, w) in widgets.iter().enumerate() {
                if let Some(w) = Widget::from_weak(w) {
                    f(w, id);
                }
            }
        }
    }

    pub fn for_each_widget_impl<F: FnMut(&mut WidgetImpl, usize)>(&self, mut f: F) {
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
                    wid.set_parent(parent);
                }

                wids.push(wid.as_weak());
            });

            self.widgets = Some(wids);
        }
    }

    fn mark_parents_redraw(&mut self) {
        self.notifier.swap_redraw(&mut self.cur_redraw);

        self.cur_parent_lookup.clear();

        for id in self.cur_redraw.iter() {
            self.cur_parent_lookup.push(*id);
        }

        while let Some(wid_id) = self.cur_parent_lookup.pop() {
            if let Some(widgets) = &mut self.widgets {
                if let Some(wid) = widgets.get(wid_id) {
                    if let Some(wid) = wid.upgrade() {
                        if let Some(parent) = wid.borrow().parent() {
                            let parent_id = parent.id();
                            self.cur_redraw.insert(parent_id);
                            self.cur_parent_lookup.push(parent_id);
                        }
                    }
                }
            }
        }

        self.notifier.swap_redraw(&mut self.cur_redraw);
    }
}

impl WindowUI for UI {
    fn pre_frame(&mut self) {
        let notifier = self.notifier.clone();

        if notifier.is_tree_changed() {
            self.on_tree_changed();
        }

        if notifier.is_layout_changed() {
            self.on_layout_changed();
        }

        if let Some(widgets) = &mut self.widgets {
            for widget in widgets {
                if let Some(widget) = widget.upgrade() {
                    if widget.borrow_mut().check_data_change() {
                        widget.borrow_mut().emit_redraw_required();
                    }
                }
            }
        }

        self.mark_parents_redraw();
    }

    fn post_frame(&mut self) {
    }

    fn needs_redraw(&mut self) -> bool { self.notifier.need_redraw() }

    fn is_active(&mut self) -> bool { true }

    fn handle_input_event(&mut self, event: InputEvent) {
        let notifier = self.notifier.clone();

        let old_hover = notifier.hover();

        match &event {
            InputEvent::MousePosition(x, y) => {
                let mut hover_id = 0;
                if let Some(zones) = &self.zones {
                    for (pos, id) in zones.iter() {
                        //d// println!("CHECK {:?} in {:?}", (*x, *y), pos);
                        if pos.is_inside(*x, *y) {
                            hover_id = *id;
                        }
                    }
                }

                notifier.set_mouse_pos((*x, *y));
                notifier.set_hover(hover_id);
            },
            _ => {},
        }

        if old_hover != notifier.hover() {
            notifier.redraw(old_hover);
            notifier.redraw(notifier.hover());
        }

        let mut sent_events : Vec<(usize, Event)> = vec![];

        self.for_each_widget(|wid, _id| {
            let ctrl = wid.take_ctrl();

            if let Some(mut ctrl) = ctrl {
                ctrl.handle(&wid, &event, &mut sent_events);

                wid.give_ctrl_back(ctrl);
            }
        });

        for (wid_id, event) in sent_events {
            if let Some(widgets) = &mut self.widgets {
                if let Some(widget) = widgets.get(wid_id) {
                    if let Some(widget) = Widget::from_weak(widget) {
                        let evc = widget.take_event_core();

                        if let Some(mut evc) = evc {
                            evc.call(&event, &widget);
                            widget.give_back_event_core(evc);
                        }
                    }
                }
            }
        }
    }

    fn draw(&mut self, painter: &mut Painter) {
        let notifier = self.notifier.clone();
        notifier.swap_redraw(&mut self.cur_redraw);
        notifier.clear_redraw();

        println!("REDRAW: {:?}", self.cur_redraw);
        widget_draw(&self.root, &self.cur_redraw, painter);
    }

    fn draw_frame(&mut self, painter: &mut Painter) {
        widget_draw_frame(&self.root, painter);
    }

    fn set_window_size(&mut self, w: f32, h: f32) {
        self.win_w = w;
        self.win_h = h;
        self.notifier.redraw(0);
    }
}
