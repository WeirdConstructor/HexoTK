use crate::{
    InputEvent, Painter, widget_draw,
    UINotifierRef, Rect, Event, EvPayload, MButton,
    widget_draw_frame,
    widget_annotate_drop_event,
    widget_handle_event,
};
use crate::painter::LblDebugTag;
use crate::WindowUI;
use crate::Widget;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashSet;
use std::collections::HashMap;

use crate::layout::LayoutCache;
use crate::widget_store::{WidgetStore, WidgetTree};

use morphorm::{
    PositionType, Units,
};

struct Layer {
    root:       Widget,
    tree:       Option<WidgetTree>,
}

#[derive(Debug)]
pub struct WidgetFeedback {
    labels: Option<Vec<(usize, &'static str, (i32, i32), Rect, String)>>,
}

pub struct TestDriver {
    injected_events: Vec<InputEvent>,
    widgets:         HashMap<usize, WidgetFeedback>,
}

impl TestDriver {
    pub fn new() -> Self {
        Self {
            injected_events: vec![],
            widgets:         HashMap::new(),
        }
    }

    pub fn apply_labels(
        &mut self, lbl_collection: Option<Vec<(LblDebugTag, (f32, f32, f32, f32, String))>>)
    {
        if let Some(coll) = lbl_collection {
            for item in coll {
                let info       = item.0.info();
                let _label_info = (
                    info.0,
                    info.1,
                    info.2,
                    Rect::from(item.1.0, item.1.1, item.1.2, item.1.3),
                    item.1.4
                );
                //d// println!("FEEDBACK: {:?}", label_info);
            }
        }
    }
}

enum FScriptStep {
    Callback(Box<dyn Fn(&mut dyn std::any::Any, Box<TestDriver>) -> Box<TestDriver>>),
}

pub struct FrameScript {
    queue:  Vec<FScriptStep>,
}

impl FrameScript {
    pub fn new() -> Self {
        Self { queue: vec![] }
    }

    pub fn push_cb(
        &mut self,
        cb: Box<dyn Fn(&mut dyn std::any::Any, Box<TestDriver>) -> Box<TestDriver>>)
    {
        self.queue.push(FScriptStep::Callback(cb));
    }
}

#[derive(Debug, Clone)]
pub struct DragState {
    button_pressed: bool,
    started:        bool,
    hover_id:       usize,
    last_query_id:  usize,
    query_accept:   bool,
    pos:            (f32, f32),
    widget:         Option<Widget>,
    userdata:       Option<Rc<RefCell<Box<dyn std::any::Any>>>>,
}

impl DragState {
    fn new() -> Self {
        Self {
            button_pressed: false,
            started:        false,
            last_query_id:  0,
            query_accept:   false,
            hover_id:       0,
            pos:            (0.0, 0.0),
            userdata:       None,
            widget:         None,
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }
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
    fb:                 Option<Box<TestDriver>>,
    scripts:            Option<Vec<FrameScript>>,
    cur_script:         Option<FrameScript>,
    drag:               DragState,
    drop_query_ev:      Event,
    ctx:                Rc<RefCell<dyn std::any::Any>>,
}

impl UI {
    pub fn new(ctx: Rc<RefCell<dyn std::any::Any>>) -> Self {
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
            fb:                 None,
            scripts:            None,
            cur_script:         None,
            drag:               DragState::new(),
            drop_query_ev: Event {
                name: "drop_query".to_string(),
                data: EvPayload::DropAccept(Rc::new(RefCell::new(false))),
            },
            ctx,
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
                l.left   = Some(Units::Pixels(0.0));
                l.right  = Some(Units::Pixels(0.0));
                l.width  = Some(Units::Pixels(win_w));
                l.height = Some(Units::Pixels(win_h));
                l.position_type = Some(PositionType::SelfDirected);
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

    pub fn push_frame_script(&mut self, script: FrameScript) {
        if self.fb.is_none() {
            self.fb = Some(Box::new(TestDriver::new()));
        }

        if let Some(scripts) = &mut self.scripts {
            scripts.push(script);
        } else {
            self.scripts = Some(vec![script]);
        }
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

    fn handle_drag_mouse_move(&mut self, x: f32, y: f32, hover_id: &mut usize) {

        if    self.drag.button_pressed
           && !self.drag.started
           && self.drag.hover_id != *hover_id
        {
            // the starting case, the mouse button was just pressed, but it did
            // not yet hover a new widget and dragging is not started yet.

            // first query the widget if it supports dragging at all.
            // for this the widget needs to set the drag UserData to something
            // else than Option<()> None.
            let sentinel : Option<()> = None;
            let sentinel : Box<dyn std::any::Any> = Box::new(sentinel);
            let userdata = Rc::new(RefCell::new(sentinel));
            if let Some(widget) = self.widgets.borrow().get(self.drag.hover_id) {
                widget_handle_event(
                    &widget, &mut *(self.ctx.borrow_mut()), &Event {
                        name: "drag".to_string(),
                        data: EvPayload::UserData(userdata.clone()),
                    });
            }

            let mut cancel_drag = false;

            {
                let ud = userdata.clone();
                let ud = ud.borrow();

                if let Some(opt) = ud.downcast_ref::<Option<()>>() {
                    if opt.is_none() {
                        cancel_drag = true;
                    }
                }
            }

            if cancel_drag {
                self.drag.reset();

            } else {
                // If the widget actually has something to drag, note that down here:
                self.drag.userdata = Some(userdata);
                self.drag.pos = (x, y);
                self.drag.started = true;

                // the drag widget is positioned and marked for a redraw
                let pos = Rect {
                    x: self.drag.pos.0,
                    y: self.drag.pos.1,
                    w: 100.0,
                    h: 40.0,
                };
                if let Some(drag_widget) = &self.drag.widget {
                    drag_widget.set_pos(pos);
                    self.notifier.redraw(drag_widget.id());
                }

                println!("DRAG START {} (=>{})!", self.drag.hover_id, *hover_id);
            }
        }
        else if self.drag.started && self.drag.hover_id == *hover_id {
            // The drag gesture gets back to the origin widget of the drag
            // this resets the drag and it will/needs to be restarted next
            // time the cursor leaves the widget.
            self.drag.started = false;
            if let Some(drag_widget) = &self.drag.widget {
                self.notifier.redraw(drag_widget.id());
            }
        }
        else if self.drag.started {
            // This case handles if the user actually drags something.
            // We need to query the (currently) hovered widget if it
            // can accept dropping what we drag at all.

            if self.drag.last_query_id != *hover_id {
                // The widget we hover is different from the most recent
                // widget we query again. For this we pass a shared reference
                // for a flag to the widget "drop_query" event handler.

                let ev = &self.drop_query_ev;
                if let EvPayload::DropAccept(rc) = &ev.data {
                    *rc.borrow_mut() = false;
                }

                if let Some(widget) = self.widgets.borrow().get(*hover_id) {
                    widget_handle_event(
                        &widget, &mut *(self.ctx.borrow_mut()), ev);
                }

                self.drag.last_query_id = *hover_id;
                self.drag.query_accept = false;

                if let EvPayload::DropAccept(rc) = &ev.data {
                    if *rc.borrow_mut() {
                        self.drag.query_accept = true;
                    }
                }
            }

            self.drag.pos = (x, y);

            // Update the drag widget position and mark for redraw:
            if let Some(drag_widget) = &self.drag.widget {

                // if the queries widget does not accept dropping, signal
                // this by setting the hover widget to he drag widget at the
                // mouse cursor:
                if !self.drag.query_accept {
                    *hover_id = drag_widget.id();
                }

                let mut pos = drag_widget.pos();
                pos.x = self.drag.pos.0;
                pos.y = self.drag.pos.1;
                drag_widget.set_pos(pos);
            }
        }
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
        if let Some(fb) = self.fb.take() {
            let ctx = self.ctx.clone();

            let mut fb_ret =
                if let Some(mut script) = self.cur_script.take() {
                    if !script.queue.is_empty() {
                        let step = script.queue.remove(0);

                        let fb_ret =
                            match step {
                                FScriptStep::Callback(cb) => {
                                    (*cb)(&mut *(ctx.borrow_mut()), fb)
                                },
                            };

                        self.cur_script = Some(script);

                        fb_ret
                    } else {
                        fb
                    }

                } else {
                    fb
                };

            if let Some(scripts) = &mut self.scripts {
                if self.cur_script.is_none() && !scripts.is_empty() {
                    self.cur_script = Some(scripts.remove(0));
                }
            }

            for ev in fb_ret.injected_events.iter() {
                self.handle_input_event(ev.clone());
            }
            fb_ret.injected_events.clear();

            self.fb = Some(fb_ret);
        }
    }

    fn needs_redraw(&mut self) -> bool { self.notifier.need_redraw() }

    fn is_active(&mut self) -> bool { true }

    fn handle_input_event(&mut self, event: InputEvent) {
        let notifier = self.notifier.clone();

        let old_hover = notifier.hover();

        let mut sent_events : Vec<(usize, Event)> = vec![];

        match &event {
            InputEvent::MouseButtonPressed(btn) => {
                if *btn == MButton::Left {
                    let drag_hover_id = notifier.hover();
                    if let Some(widget) =
                        self.widgets.borrow().get(drag_hover_id)
                    {
                        if let Some(widget) = widget.drag_widget() {
                            self.drag.button_pressed = true;
                            self.drag.hover_id       = drag_hover_id;

                            widget.set_notifier(notifier.clone(), 9999991999);
                            self.drag.widget = Some(widget);
                        }
                    }
                }
            }
            InputEvent::MouseButtonReleased(btn) => {
                if *btn == MButton::Left {
                    let hov_id = notifier.hover();
                    if self.drag.started && self.drag.hover_id != hov_id {
                        println!("DROP! {} on {}", self.drag.hover_id, hov_id);
                        if let Some(ud) = &self.drag.userdata  {
                            if let Some(widget) =
                                self.widgets.borrow().get(hov_id)
                            {
                                let ev = Event {
                                    name: "drop".to_string(),
                                    data: EvPayload::UserData(ud.clone()),
                                };
                                let ev =
                                    widget_annotate_drop_event(
                                        &widget, self.drag.pos, ev);
                                sent_events.push((hov_id, ev));
                            }
                        }
                    }
                    self.drag.reset();

                    if let Some(widget) = &self.drag.widget {
                        notifier.redraw(widget.id());
                    }
                }
            }
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

                self.handle_drag_mouse_move(*x, *y, &mut hover_id);

                notifier.set_mouse_pos((*x, *y));
                notifier.set_hover(hover_id);
            },
            _ => {},
        }

        if old_hover != notifier.hover() {
            notifier.redraw(old_hover);
            notifier.redraw(notifier.hover());
        }

        self.widgets.borrow().for_each_widget(|wid, _id| {
            let ctrl = wid.take_ctrl();

            if let Some(mut ctrl) = ctrl {
                ctrl.handle(&wid, &event, &mut sent_events);

                wid.give_ctrl_back(ctrl);
            }
        });

        let ctx = self.ctx.clone();

        for (wid_id, event) in sent_events {
            if let Some(widget) = self.widgets.borrow().get(wid_id) {
                widget_handle_event(&widget, &mut *(ctx.borrow_mut()), &event);
            }
        }
    }

    fn draw(&mut self, painter: &mut Painter) {
        self.ftm.start_measure();
        let notifier = self.notifier.clone();

        if self.fb.is_some() {
            painter.start_label_collector();
        }

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

        if let Some(drag_widget) = &self.drag.widget {
            if self.drag.started {
                widget_draw(drag_widget, &self.cur_redraw, painter);
            }
        }

        if let Some(fb) = &mut self.fb {
            fb.apply_labels(painter.get_label_collection());
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
