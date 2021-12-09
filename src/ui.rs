use crate::{
    InputEvent, Painter, widget_draw,
    widget_draw_frame,
    UINotifierRef, Rect, Event
};
use crate::WindowUI;
use crate::Widget;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashSet;

use crate::layout::LayoutCache;
use crate::widget_store::{WidgetStore, WidgetTree};

use morphorm::{
    PositionType, Units,
};

struct Layer {
    root:       Widget,
    tree:       Option<WidgetTree>,
}

pub struct UI {
    win_w:              f32,
    win_h:              f32,
    layers:             Vec<Layer>,
    widgets:            Rc<RefCell<WidgetStore>>,
    notifier:           UINotifierRef,
    zones:              Option<Vec<(Rect, bool, usize)>>,
    cur_redraw:         HashSet<usize>,
    cur_parent_lookup:  Vec<usize>,
    layout_cache:       LayoutCache,
    ftm:                crate::window::FrameTimeMeasurement,
}

impl UI {
    pub fn new() -> Self {
        let store = Rc::new(RefCell::new(WidgetStore::new()));
        Self {
            win_h:              0.0,
            win_w:              0.0,
            layers:             vec![],
            widgets:            store.clone(),
            notifier:           UINotifierRef::new(),
            zones:              Some(vec![]),
            cur_redraw:         HashSet::new(),
            cur_parent_lookup:  vec![],
            layout_cache:       LayoutCache::new(store),
            ftm:                crate::window::FrameTimeMeasurement::new("layout"),
        }
    }

    pub fn add_layer_root(&mut self, root: Widget) {
        self.layers.push(Layer { root, tree: None });

        self.on_tree_changed();
    }

    pub fn relayout(&mut self) {
        println!("start relayout");

        let (win_w, win_h) = (self.win_w, self.win_h);

        for layer in &mut self.layers {
            layer.root.change_layout(|l| {
                l.left   = Units::Pixels(0.0);
                l.right  = Units::Pixels(0.0);
                l.width  = Units::Pixels(win_w);
                l.height = Units::Pixels(win_h);
                l.position_type = PositionType::SelfDirected;
            });

            if layer.tree.is_none() {
                layer.tree =
                    Some(WidgetTree::from_root(
                        self.widgets.clone(), &layer.root));
            }

            let tree = layer.tree.as_ref().unwrap();

            morphorm::layout(
                &mut self.layout_cache,
                tree,
                &self.widgets.clone());

            tree.apply_layout_to_widgets(&self.layout_cache);
            layer.root.set_pos(Rect::from(0.0, 0.0, win_w, win_h));
        }

        self.on_layout_changed();
    }

    pub fn on_tree_changed(&mut self) {
        println!("tree changed");
        self.refresh_widget_list();

        let notifier = self.notifier.clone();

        self.widgets.borrow().for_each_widget_impl(|wid, id| {
            wid.set_notifier(notifier.clone(), id);
            notifier.redraw(id);
        });

        for layer in &mut self.layers {
            layer.tree = None;
        }

        self.notifier.reset_tree_changed();

        self.relayout();
    }

    pub fn on_layout_changed(&mut self) {
        //d// println!("layout changed");
        let zones = self.zones.take();

        if let Some(mut zones) = zones {
            zones.clear();

            self.widgets.borrow().for_each_widget(|wid, id| {
                zones.push((wid.pos(), wid.can_hover(), id));
            });

            self.zones = Some(zones);
        }

        self.notifier.reset_layout_changed();
    }
}

impl UI {
    fn refresh_widget_list(&mut self) {
        self.widgets.borrow_mut().clear();

        for layer in &self.layers {
            self.widgets.borrow_mut().add_root(&layer.root);
        }

        self.layout_cache.clear_to_len(self.widgets.borrow().len());
    }

    fn mark_parents_redraw(&mut self) {
        self.notifier.swap_redraw(&mut self.cur_redraw);

        self.cur_parent_lookup.clear();

        for id in self.cur_redraw.iter() {
            self.cur_parent_lookup.push(*id);
        }

        while let Some(wid_id) = self.cur_parent_lookup.pop() {
            if let Some(wid) = self.widgets.borrow().get(wid_id) {
                if let Some(parent) = wid.parent() {
                    let parent_id = parent.id();
                    self.cur_redraw.insert(parent_id);
                    self.cur_parent_lookup.push(parent_id);
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

        self.widgets.borrow().for_each_widget(|wid, _id| {
            if wid.check_data_change() {
                wid.emit_redraw_required();
            }
        });

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
                    for (pos, can_hover, id) in zones.iter() {
                        if !can_hover { continue; }

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

        self.widgets.borrow().for_each_widget(|wid, _id| {
            let ctrl = wid.take_ctrl();

            if let Some(mut ctrl) = ctrl {
                ctrl.handle(&wid, &event, &mut sent_events);

                wid.give_ctrl_back(ctrl);
            }
        });

        for (wid_id, event) in sent_events {
            if let Some(widget) = self.widgets.borrow().get(wid_id) {
                let evc = widget.take_event_core();

                if let Some(mut evc) = evc {
                    evc.call(&event, &widget);
                    widget.give_back_event_core(evc);
                }
            }
        }
    }

    fn draw(&mut self, painter: &mut Painter) {
        self.ftm.start_measure();
        let notifier = self.notifier.clone();

        if notifier.is_layout_changed() {
            self.relayout();
        }

        notifier.swap_redraw(&mut self.cur_redraw);
        notifier.clear_redraw();
        self.ftm.end_measure();

        //d// println!("REDRAW: {:?}", self.cur_redraw);
        for layer in &self.layers {
            widget_draw(&layer.root, &self.cur_redraw, painter);
        }
    }

    fn draw_frame(&mut self, painter: &mut Painter) {
        for layer in &self.layers {
            widget_draw_frame(&layer.root, painter);
        }
    }

    fn set_window_size(&mut self, w: f32, h: f32) {
        self.win_w = w;
        self.win_h = h;
        self.notifier.set_layout_changed();
        self.notifier.redraw(0);
    }
}
