// Copyright (c) 2021-2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::painter::LblDebugTag;
use crate::Widget;
use crate::WindowUI;
use crate::{
    widget_annotate_drop_event, widget_draw, widget_draw_frame, widget_draw_shallow,
    widget_handle_event, EvPayload, Event, EventCore, InputEvent, MButton, Painter, PopupPos, Rect,
    UINotifierRef,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use crate::layout::{LayoutCache, LayoutTree};
use crate::widget::{widget_walk, widget_walk_parents};
use crate::widget_store::{WidgetStore, WidgetTree};

use keyboard_types::{Key, KeyboardEvent};
use morphorm::{PositionType, Units};

struct Layer {
    layer_idx: usize,
    root: Widget,
    tree: Option<WidgetTree>,
    popups: Vec<(Widget, PopupPos)>,
}

impl Layer {
    fn handle_popup_positioning_after_layout(
        &mut self,
        win_w: f32,
        win_h: f32,
        mouse_pos: (f32, f32),
        dpi_f: f32,
    ) {
        while let Some((wid, pos)) = self.popups.pop() {
            let dest_pos = match pos {
                PopupPos::MousePos => mouse_pos,
                PopupPos::MouseOffs(ox, oy) => (mouse_pos.0 + dpi_f * ox, mouse_pos.1 + dpi_f * oy),
            };

            let popup_pos = wid.pos();

            let (mut offs_x, mut offs_y) = (dest_pos.0 - popup_pos.x, dest_pos.1 - popup_pos.y);
            let overhang_x = (popup_pos.x + popup_pos.w + offs_x) - win_w;
            let overhang_y = (popup_pos.y + popup_pos.h + offs_y) - win_h;
            if overhang_x > 0.0 {
                offs_x -= overhang_x;
            }
            if overhang_y > 0.0 {
                offs_y -= overhang_y;
            }
            if popup_pos.x + offs_x < 0.0 {
                offs_x = 0.0 - popup_pos.x;
            }
            if popup_pos.y + offs_y < 0.0 {
                offs_y = 0.0 - popup_pos.y;
            }

            widget_walk(&wid, |wid, _parent, _is_first, _is_last, depth| {
                let pos = wid.pos();
                let pos = pos.offs(offs_x, offs_y);
                wid.set_pos(pos);
                wid.set_tree_pos(usize::MAX, depth);
            });

            let wid_pos = wid.pos();
            wid.change_layout_silent(|layout| {
                layout.left = Some(Units::Pixels(wid_pos.x));
                layout.top = Some(Units::Pixels(wid_pos.y));
            });
        }
    }
}

#[derive(Debug, Clone)]
pub struct WidgetFeedback {
    // widid, source,       logicpos,   pos,  text
    labels: Vec<(usize, &'static str, (i32, i32), Rect, String)>,
}

#[derive(Debug, Clone)]
pub struct TestDriver {
    injected_events: Vec<InputEvent>,
    widgets: HashMap<usize, WidgetFeedback>,
    //  tag,   tag_path, ctrl, widget pos
    widget_tags: HashMap<usize, (String, String, String, Rect)>,
}

#[derive(Debug, Clone)]
pub struct LabelInfo {
    pub wid_id: usize,
    pub wid_pos: Rect,
    pub source: &'static str,
    pub logic_pos: (i32, i32),
    pub pos: Rect,
    pub text: String,
    pub tag: String,
    pub tag_path: String,
    pub ctrl: String,
}

impl TestDriver {
    pub fn new() -> Self {
        Self { injected_events: vec![], widgets: HashMap::new(), widget_tags: HashMap::new() }
    }

    pub fn get_all_labels(&self) -> Vec<LabelInfo> {
        let mut ret = vec![];
        for (_id, wid) in self.widgets.iter() {
            for wfb in wid.labels.iter() {
                let (tag, tag_path, ctrl, wid_pos) =
                    self.widget_tags.get(&wfb.0).cloned().unwrap_or_else(|| {
                        (
                            "".to_string(),
                            "".to_string(),
                            "".to_string(),
                            Rect::from(0.0, 0.0, 0.0, 0.0),
                        )
                    });

                ret.push(LabelInfo {
                    wid_id: wfb.0,
                    source: wfb.1,
                    logic_pos: wfb.2,
                    pos: wfb.3,
                    text: wfb.4.to_string(),
                    wid_pos,
                    tag,
                    tag_path,
                    ctrl,
                });
            }
        }

        ret
    }

    pub fn inject_char(&mut self, chr: &str) {
        let mut ev = KeyboardEvent::default();
        ev.key = Key::Character(chr.to_string());
        ev.state = keyboard_types::KeyState::Down;
        self.injected_events.push(InputEvent::KeyPressed(ev));
    }

    pub fn inject_key_down(&mut self, key: Key) {
        let mut ev = KeyboardEvent::default();
        ev.key = key;
        ev.state = keyboard_types::KeyState::Down;
        self.injected_events.push(InputEvent::KeyPressed(ev));
    }

    pub fn inject_key_up(&mut self, key: Key) {
        let mut ev = KeyboardEvent::default();
        ev.key = key;
        ev.state = keyboard_types::KeyState::Up;
        self.injected_events.push(InputEvent::KeyReleased(ev));
    }

    pub fn inject_mouse_press_at(&mut self, x: f32, y: f32, btn: MButton) {
        self.injected_events.push(InputEvent::MousePosition(x, y));
        self.injected_events.push(InputEvent::MouseButtonPressed(btn));
    }

    pub fn inject_mouse_release_at(&mut self, x: f32, y: f32, btn: MButton) {
        self.injected_events.push(InputEvent::MousePosition(x, y));
        self.injected_events.push(InputEvent::MouseButtonReleased(btn));
    }

    pub fn inject_mouse_to(&mut self, x: f32, y: f32) {
        self.injected_events.push(InputEvent::MousePosition(x, y));
    }

    pub fn reset_tags(&mut self) {
        self.widget_tags.clear();
    }

    pub fn set_tag(&mut self, id: usize, tag: String, tag_path: String, ctrl: String, pos: Rect) {
        self.widget_tags.insert(id, (tag, tag_path, ctrl, pos));
    }

    pub fn apply_labels(
        &mut self,
        lbl_collection: Option<Vec<(LblDebugTag, (f32, f32, f32, f32, String))>>,
    ) {
        if let Some(coll) = lbl_collection {
            self.widgets.clear();

            for item in coll {
                let info = item.0.info();
                let label_info = (
                    info.0,
                    info.1,
                    info.2,
                    Rect::from(item.1 .0, item.1 .1, item.1 .2, item.1 .3),
                    item.1 .4,
                );

                if let Some(wf) = self.widgets.get_mut(&info.0) {
                    wf.labels.push(label_info);
                } else {
                    self.widgets.insert(info.0, WidgetFeedback { labels: vec![label_info] });
                }
            }
        }
    }
}

#[derive(Clone)]
enum FScriptStep {
    Callback(Rc<dyn Fn(&mut dyn std::any::Any, Box<TestDriver>) -> (bool, Box<TestDriver>)>),
}

#[derive(Clone)]
pub struct TestScript {
    name: String,
    queue: Vec<(String, FScriptStep)>,
}

impl TestScript {
    pub fn new(name: String) -> Self {
        Self { name, queue: vec![] }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn push_cb(
        &mut self,
        step_name: String,
        cb: Rc<dyn Fn(&mut dyn std::any::Any, Box<TestDriver>) -> (bool, Box<TestDriver>)>,
    ) {
        self.queue.push((step_name, FScriptStep::Callback(cb)));
    }
}

#[derive(Debug, Clone)]
pub struct DragState {
    button_pressed: bool,
    started: bool,
    hover_id: usize,
    last_query_id: usize,
    query_accept: bool,
    pos: (f32, f32),
    widget: Option<Widget>,
    userdata: Option<Rc<RefCell<Box<dyn std::any::Any>>>>,
}

impl DragState {
    fn new() -> Self {
        Self {
            button_pressed: false,
            started: false,
            last_query_id: 0,
            query_accept: false,
            hover_id: 0,
            pos: (0.0, 0.0),
            userdata: None,
            widget: None,
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }
}

pub struct UI {
    win_w: f32,
    win_h: f32,
    dpi_factor: f32,
    layers: Vec<Layer>,
    widgets: Rc<RefCell<WidgetStore>>,
    notifier: UINotifierRef,
    zones: Option<Vec<(Rect, bool, usize, usize, usize)>>,
    cur_redraw: HashSet<usize>,
    cur_parent_lookup: Vec<usize>,
    layout_cache: LayoutCache,
    ftm: crate::window::FrameTimeMeasurement,
    fb: Option<Box<TestDriver>>,
    scripts: Option<Vec<TestScript>>,
    cur_script: Option<TestScript>,
    tests_run: usize,
    tests_fail: usize,
    drag: DragState,
    drop_query_ev: Event,
    hover_ev: Event,
    last_hover_id: usize,
    auto_hide_queue: Vec<(usize, HashSet<usize>)>,
    frame_cb: Option<Box<dyn FnMut(&mut dyn std::any::Any)>>,
    ctx: Rc<RefCell<dyn std::any::Any>>,
    global_event_core: EventCore,
    driver_handle_cb: Option<Box<dyn FnMut(Box<TestDriver>) -> Box<TestDriver>>>,

    image_data: HashMap<String, Vec<u8>>,
}

impl UI {
    pub fn new(ctx: Rc<RefCell<dyn std::any::Any>>) -> Self {
        let store = Rc::new(RefCell::new(WidgetStore::new()));
        Self {
            win_h: 0.0,
            win_w: 0.0,
            dpi_factor: 1.0,

            layers: vec![],
            widgets: store.clone(),
            notifier: UINotifierRef::new(),
            zones: Some(vec![]),

            cur_redraw: HashSet::new(),
            cur_parent_lookup: vec![],
            layout_cache: LayoutCache::new(store),
            ftm: crate::window::FrameTimeMeasurement::new("layout"),
            fb: None,
            scripts: None,
            cur_script: None,
            global_event_core: EventCore::new(),
            driver_handle_cb: None,

            tests_run: 0,
            tests_fail: 0,
            drag: DragState::new(),
            auto_hide_queue: vec![],
            frame_cb: None,

            drop_query_ev: Event {
                name: "drop_query".to_string(),
                data: EvPayload::DropAccept(Rc::new(RefCell::new((
                    Rc::new(RefCell::new(Box::new(0))),
                    false,
                )))),
            },
            last_hover_id: usize::MAX,
            hover_ev: Event { name: "hover".to_string(), data: EvPayload::None },
            image_data: HashMap::new(),
            ctx,
        }
    }

    pub fn store_image_data(&mut self, file: &str, data: Vec<u8>) {
        self.image_data.insert(file.to_string(), data);
    }

    pub fn set_frame_callback(&mut self, cb: Box<dyn FnMut(&mut dyn std::any::Any)>) {
        self.frame_cb = Some(cb);
    }

    pub fn add_layer_root(&mut self, root: Widget) {
        let index = self.layers.len();
        // Roots must always be cached, otherwise they won't be redrawn in the
        // per frame drawing and you get heavy flickering :)
        root.enable_cache();
        self.layers.push(Layer { layer_idx: index, root, tree: None, popups: vec![] });

        self.on_tree_changed();
    }

    pub fn reg(&mut self, event: &str, cb: Box<dyn FnMut(&mut dyn std::any::Any, Widget, &Event)>) {
        self.global_event_core.reg(event, cb);
    }

    pub fn reg_driver_cb(&mut self, cb: Box<dyn FnMut(Box<TestDriver>) -> Box<TestDriver>>) {
        self.driver_handle_cb = Some(cb);
    }

    pub fn relayout(&mut self) {
        println!("start relayout w={}, h={}", self.win_w, self.win_h);

        let (win_w, win_h) = (self.win_w, self.win_h);

        if let Some(fb) = &mut self.fb {
            fb.reset_tags();
        }

        for layer in &mut self.layers {
            layer.root.change_layout(|l| {
                l.left = Some(Units::Pixels(0.0));
                l.top = Some(Units::Pixels(0.0));
                l.width = Some(Units::Pixels(win_w / self.dpi_factor));
                l.height = Some(Units::Pixels(win_h / self.dpi_factor));
                l.position_type = Some(PositionType::SelfDirected);
            });

            if layer.tree.is_none() {
                layer.tree = Some(WidgetTree::from_root(self.widgets.clone(), &layer.root));
            }

            let tree = layer.tree.as_ref().unwrap();

            morphorm::layout(
                &mut self.layout_cache,
                tree,
                &LayoutTree { dpi_factor: self.dpi_factor, store: self.widgets.clone() },
            );

            tree.apply_layout_to_widgets(&self.layout_cache);
            layer.root.set_pos(Rect::from(0.0, 0.0, win_w, win_h));

            let mouse_pos = self.notifier.mouse_pos();
            layer.handle_popup_positioning_after_layout(win_w, win_h, mouse_pos, self.dpi_factor);
        }

        if let Some(mut fb) = self.fb.take() {
            let wids = self.widgets.clone();
            wids.borrow().for_each_widget(|wid| {
                let tag = wid.tag();
                let mut tag_path: Vec<String> = vec![tag.clone()];
                let mut root = wid.clone();
                widget_walk_parents(&wid, |par| {
                    tag_path.push(par.tag());
                    root = par.clone();
                });

                let mut tag_path_str = format!(
                    "layer_{}",
                    self.find_layer_by_root_id(root.unique_id())
                        .map(|layer| layer.layer_idx)
                        .unwrap_or(0)
                );

                for tag in tag_path.iter().rev() {
                    tag_path_str += ".";
                    tag_path_str += &tag;
                }

                let ctrl = if let Some(ctrl) = wid.take_ctrl() {
                    let ret = format!("{:?}", ctrl);
                    wid.give_ctrl_back(ctrl);
                    ret
                } else {
                    "".to_string()
                };

                fb.set_tag(wid.unique_id(), tag, tag_path_str, ctrl, wid.pos());
            });

            self.fb = Some(fb);
        }

        self.on_layout_changed();
    }

    pub fn on_tree_changed(&mut self) {
        println!("tree changed");
        self.refresh_widget_list();

        let notifier = self.notifier.clone();

        let mut auto_hide_queue = vec![];

        let mut auto_hide_widgets = vec![];

        self.widgets.borrow().for_each_widget_impl(|wid| {
            if wid.wants_auto_hide() {
                auto_hide_widgets.push(wid.unique_id());
            }
            wid.set_notifier(notifier.clone());
            notifier.redraw(wid.unique_id());
        });

        for auto_hide_id in auto_hide_widgets {
            if let Some(wid) = self.widgets.borrow().get(auto_hide_id) {
                let mut subtree_set = HashSet::new();
                subtree_set.insert(wid.unique_id());
                widget_walk(&wid, |wid, _parent, _is_first, _is_last, _depth| {
                    subtree_set.insert(wid.unique_id());
                });
                auto_hide_queue.push((wid.unique_id(), subtree_set));
            }
        }

        self.auto_hide_queue = auto_hide_queue;

        for layer in &mut self.layers {
            layer.tree = None;
        }

        self.notifier.reset_tree_changed();

        self.relayout();
    }

    pub fn on_layout_changed(&mut self) {
        println!("layout changed");
        let zones = self.zones.take();

        if let Some(mut zones) = zones {
            zones.clear();

            self.widgets.borrow().for_each_widget(|wid| {
                if wid.is_visible() {
                    zones.push((
                        wid.pos(),
                        wid.can_hover(),
                        wid.layer_idx(),
                        wid.tree_depth(),
                        wid.unique_id(),
                    ));
                }
            });

            self.zones = Some(zones);
        }

        self.notifier.reset_layout_changed();
    }

    pub fn install_test_script(&mut self, script: TestScript) {
        if self.fb.is_none() {
            self.fb = Some(Box::new(TestDriver::new()));
        }

        if let Some(scripts) = &mut self.scripts {
            scripts.push(script);
        } else {
            self.scripts = Some(vec![script]);
        }
    }

    fn find_layer_by_root_id(&mut self, root_widget_id: usize) -> Option<&mut Layer> {
        for layer in &mut self.layers {
            if layer.root.unique_id() == root_widget_id {
                return Some(layer);
            }
        }

        None
    }
}

impl UI {
    fn refresh_widget_list(&mut self) {
        self.widgets.borrow_mut().clear();

        for (idx, layer) in self.layers.iter().enumerate() {
            self.widgets.borrow_mut().add_root(&layer.root, idx);
        }

        self.layout_cache.clear();
    }

    fn mark_parents_redraw(&mut self) {
        self.notifier.swap_redraw(&mut self.cur_redraw);

        self.cur_parent_lookup.clear();

        for id in self.cur_redraw.iter() {
            self.cur_parent_lookup.push(*id);
        }

        while let Some(wid_id) = self.cur_parent_lookup.pop() {
            if let Some(wid) = self.widgets.borrow().get(wid_id) {
                // Do not lookup parents and mark them for redraw, if the
                // widget that was marked for redraw is not visible anyways.
                if !wid.is_visible() {
                    continue;
                }

                if let Some(parent) = wid.parent() {
                    let parent_id = parent.unique_id();
                    self.cur_redraw.insert(parent_id);
                    self.cur_parent_lookup.push(parent_id);
                }
            }
        }

        self.notifier.swap_redraw(&mut self.cur_redraw);
    }

    fn handle_drag_mouse_pressed(&mut self) {
        let drag_hover_id = self.notifier.hover();
        if let Some(widget) = self.widgets.borrow().get(drag_hover_id) {
            if let Some(widget) = widget.drag_widget() {
                self.drag.button_pressed = true;
                self.drag.hover_id = drag_hover_id;

                widget.set_notifier(self.notifier.clone());
                self.drag.widget = Some(widget);
            }
        }
    }

    fn handle_drag_mouse_released(&mut self) {
        let hov_id = self.notifier.hover();
        if self.drag.started && self.drag.hover_id != hov_id {
            //d// println!("DROP! {} on {}", self.drag.hover_id, hov_id);
            if let Some(ud) = &self.drag.userdata {
                if let Some(widget) = self.widgets.borrow().get(hov_id) {
                    let ev =
                        Event { name: "drop".to_string(), data: EvPayload::UserData(ud.clone()) };
                    let ev = widget_annotate_drop_event(&widget, self.drag.pos, ev);

                    if let Some(widget) = self.widgets.borrow().get(hov_id) {
                        widget_handle_event(&widget, &mut *(self.ctx.borrow_mut()), &ev);
                    }
                }
            }
        }
        self.drag.reset();

        if let Some(widget) = &self.drag.widget {
            self.notifier.redraw(widget.unique_id());
        }
    }

    fn handle_drag_mouse_move(&mut self, x: f32, y: f32, hover_id: &mut usize) {
        if self.drag.button_pressed && !self.drag.started && self.drag.hover_id != *hover_id {
            // the starting case, the mouse button was just pressed, but it did
            // not yet hover a new widget and dragging is not started yet.

            // first query the widget if it supports dragging at all.
            // for this the widget needs to set the drag UserData to something
            // else than Option<()> None.
            let sentinel: Option<()> = None;
            let sentinel: Box<dyn std::any::Any> = Box::new(sentinel);
            let userdata = Rc::new(RefCell::new(sentinel));
            if let Some(widget) = self.widgets.borrow().get(self.drag.hover_id) {
                widget_handle_event(
                    &widget,
                    &mut *(self.ctx.borrow_mut()),
                    &Event {
                        name: "drag".to_string(),
                        data: EvPayload::UserData(userdata.clone()),
                    },
                );
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

                if let Some(drag_widget) = &self.drag.widget {
                    // the drag widget is positioned and marked for a redraw
                    let mut pos = drag_widget.pos();
                    pos.x = self.drag.pos.0;
                    pos.y = self.drag.pos.1;
                    drag_widget.set_pos(pos);
                    self.notifier.redraw(drag_widget.unique_id());
                }

                //d// println!("DRAG START {} (=>{})!", self.drag.hover_id, *hover_id);
            }
        } else if self.drag.started && self.drag.hover_id == *hover_id {
            // The drag gesture gets back to the origin widget of the drag
            // this resets the drag and it will/needs to be restarted next
            // time the cursor leaves the widget.
            self.drag.started = false;
            if let Some(drag_widget) = &self.drag.widget {
                self.notifier.redraw(drag_widget.unique_id());
            }
        } else if self.drag.started {
            // This case handles if the user actually drags something.
            // We need to query the (currently) hovered widget if it
            // can accept dropping what we drag at all.

            if self.drag.last_query_id != *hover_id {
                // The widget we hover is different from the most recent
                // widget we query again. For this we pass a shared reference
                // for a flag to the widget "drop_query" event handler.

                let user_data = self.drag.userdata.clone();
                let mut old_ud = None;

                let ev = &self.drop_query_ev;
                if let EvPayload::DropAccept(rc) = &ev.data {
                    old_ud = Some(rc.borrow().0.clone());
                    *rc.borrow_mut() = (user_data.unwrap(), false);
                }

                if let Some(widget) = self.widgets.borrow().get(*hover_id) {
                    widget_handle_event(&widget, &mut *(self.ctx.borrow_mut()), ev);
                }

                self.drag.last_query_id = *hover_id;
                self.drag.query_accept = false;

                if let EvPayload::DropAccept(rc) = &ev.data {
                    rc.borrow_mut().0 = old_ud.expect("old dummy userdata");
                    if rc.borrow_mut().1 {
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
                    *hover_id = drag_widget.unique_id();
                }

                let mut pos = drag_widget.pos();
                pos.x = self.drag.pos.0;
                pos.y = self.drag.pos.1;
                drag_widget.set_pos(pos);
            }
        }
    }

    fn deposit_popups_in_layers(&mut self) {
        while let Some((wid_id, pos)) = self.notifier.pop_popup() {
            let mut root_wid = None;
            let mut orig_wid = None;

            if let Some(wid) = self.widgets.borrow().get(wid_id) {
                orig_wid = Some(wid.clone());

                let mut cur_wid = wid.clone();
                while let Some(parent) = cur_wid.parent() {
                    cur_wid = parent;
                }

                root_wid = Some(cur_wid);
            }

            if let Some(root_wid) = root_wid {
                let wid = orig_wid.expect("orig_wid set when root_wid was found!");
                wid.show();
                let orig_pos = wid.pos();
                wid.set_pos(Rect { x: 0.0, y: 0.0, w: orig_pos.w, h: orig_pos.h });

                if let Some(layer) = self.find_layer_by_root_id(root_wid.unique_id()) {
                    layer.popups.push((wid, pos));
                }
            }
        }
    }

    fn do_auto_hide_if_not_inside(&mut self, pos: (f32, f32)) -> bool {
        for (wid_id, _subtree) in self.auto_hide_queue.iter() {
            if let Some(wid) = self.widgets.borrow().get(*wid_id) {
                if wid.is_visible() {
                    if !wid.pos().is_inside(pos.0, pos.1) {
                        wid.hide();
                        self.notifier.redraw(*wid_id);
                        return true;
                    }
                }
            }
        }

        false
    }

    fn do_auto_hide(&mut self, active_wid_id: Option<usize>) {
        //d// println!("DO AUTO HIDE {:?}", active_wid_id);
        for (wid_id, subtree) in self.auto_hide_queue.iter() {
            if let Some(active_wid_id) = active_wid_id {
                // Ignore if the active widget is a sub widget of the auto_hide widget!
                //d// println!("check autohide {} {:?} {:?}", wid_id, active_wid_id, subtree);
                if subtree.get(&active_wid_id).is_some() {
                    //d// println!("SKIP AUTO HIDE!?");
                    continue;
                }
            }

            if let Some(wid) = self.widgets.borrow().get(*wid_id) {
                if wid.is_visible() {
                    wid.hide();
                    self.notifier.redraw(*wid_id);
                }
            }
        }
    }
}

impl WindowUI for UI {
    fn pre_frame(&mut self) {
        if let Some(cb) = &mut self.frame_cb {
            cb(&mut *(self.ctx.borrow_mut()));
        }

        let notifier = self.notifier.clone();

        self.deposit_popups_in_layers();

        if notifier.is_tree_changed() {
            self.on_tree_changed();
        }

        self.widgets.borrow().for_each_widget(|wid| {
            if wid.check_data_change() {
                wid.emit_redraw_required();
            }
        });

        self.mark_parents_redraw();
    }

    fn post_frame(&mut self) {
        #[allow(unused_assignments)]
        if let Some(fb) = self.fb.take() {
            let ctx = self.ctx.clone();

            let fb_ret = if let Some(mut script) = self.cur_script.take() {
                if script.queue.is_empty() {
                    println!("*** PASS - {}", script.name());

                    fb
                } else {
                    let (step_name, step) = script.queue.remove(0);

                    let mut ok = false;

                    let fb_ret = match step {
                        FScriptStep::Callback(cb) => {
                            let (ok_flag, fb) = (*cb)(&mut *(ctx.borrow_mut()), fb);
                            ok = ok_flag;
                            fb
                        }
                    };

                    if !ok {
                        eprintln!("### FAIL - {} - step {}", script.name(), step_name);
                        self.tests_fail += 1;
                    } else {
                        self.cur_script = Some(script);
                    }

                    fb_ret
                }
            } else {
                fb
            };

            if let Some(scripts) = &mut self.scripts {
                if self.cur_script.is_none() && !scripts.is_empty() {
                    self.tests_run += 1;
                    self.cur_script = Some(scripts.remove(0));
                }

                if self.cur_script.is_none() && scripts.is_empty() {
                    if self.tests_run > 0 {
                        if self.tests_fail > 0 {
                            println!(
                                "### TESTS FAIL: {} run, {} pass, {} fail",
                                self.tests_run,
                                self.tests_run - self.tests_fail,
                                self.tests_fail
                            );
                        } else {
                            println!(
                                "*** TESTS OK: {} run, {} pass, {} fail",
                                self.tests_run,
                                self.tests_run - self.tests_fail,
                                self.tests_fail
                            );
                        }
                        self.tests_run = 0;
                    }
                }
            }

            let mut fb_ret =
                if let Some(cb) = &mut self.driver_handle_cb { (cb)(fb_ret) } else { fb_ret };

            for ev in fb_ret.injected_events.iter() {
                self.handle_input_event(ev.clone());
            }
            fb_ret.injected_events.clear();

            self.fb = Some(fb_ret);
        }
    }

    fn is_active(&mut self) -> bool {
        true
    }

    fn handle_input_event(&mut self, event: InputEvent) {
        let notifier = self.notifier.clone();

        let old_hover = notifier.hover();
        let old_active = notifier.active();

        let mut sent_events: Vec<(usize, Event)> = vec![];

        match &event {
            InputEvent::MouseButtonPressed(btn) => {
                if *btn == MButton::Left {
                    self.handle_drag_mouse_pressed();
                }

                if self.do_auto_hide_if_not_inside(notifier.mouse_pos()) {
                    return;
                }
            }
            InputEvent::MouseButtonReleased(btn) => {
                if *btn == MButton::Left {
                    self.handle_drag_mouse_released();
                }

                let (x, y) = notifier.mouse_pos();
                let ctx = self.ctx.clone();

                if let Some(layer) = self.layers.get(0) {
                    self.global_event_core.call(
                        &mut *(ctx.borrow_mut()),
                        &Event {
                            name: "click".to_string(),
                            data: EvPayload::Click { x, y, button: *btn },
                        },
                        &layer.root,
                    );
                }
            }
            InputEvent::MousePosition(x, y) => {
                let mut hover_id = 0;
                if let Some(zones) = &self.zones {
                    let mut layer_max_idx = 0;
                    let mut max_tree_depth = 0;
                    for (pos, can_hover, layer_idx, tree_depth, id) in zones.iter() {
                        if !can_hover {
                            continue;
                        }

                        //d// println!("CHECK {:?} in {:?}", (*x, *y), pos);
                        if pos.is_inside(*x, *y) {
                            if *layer_idx > layer_max_idx {
                                //d// eprintln!(
                                //d//     "*** NEW HOVER layer={}, treedepth={}, id={}",
                                //d//     *layer_idx, *tree_depth, *id
                                //d// );
                                layer_max_idx = *layer_idx;
                                max_tree_depth = *tree_depth;
                                hover_id = *id;
                            } else if *layer_idx == layer_max_idx && *tree_depth > max_tree_depth {
                                //d// eprintln!(
                                //d//     "*** NEW HOVER layer={}, treedepth={}, id={}",
                                //d//     *layer_idx, *tree_depth, *id
                                //d// );
                                max_tree_depth = *tree_depth;
                                hover_id = *id;
                            }
                        }
                    }
                }
                //d// println!("MOUSE POS {},{} hover old={}, id={}", x, y, old_hover, hover_id);

                self.handle_drag_mouse_move(*x, *y, &mut hover_id);

                notifier.set_mouse_pos((*x, *y));
                notifier.set_hover(hover_id);
            }
            InputEvent::KeyPressed(key) => match &key.key {
                Key::Escape => {
                    self.do_auto_hide(None);
                }
                _ => {}
            },
            _ => {}
        }

        let new_hover_id = notifier.hover();

        if old_hover != new_hover_id {
            notifier.redraw(old_hover);
            notifier.redraw(new_hover_id);
        }

        let ctx = self.ctx.clone();

        if self.last_hover_id != new_hover_id {
            self.last_hover_id = new_hover_id;

            if let Some(widget) = self.widgets.borrow().get(new_hover_id) {
                widget_handle_event(&widget, &mut *(ctx.borrow_mut()), &self.hover_ev);
            }
        }

        self.widgets.borrow().for_each_widget(|wid| {
            let ctrl = wid.take_ctrl();

            if let Some(mut ctrl) = ctrl {
                ctrl.handle(&wid, &event, &mut sent_events);

                wid.give_ctrl_back(ctrl);
            }
        });

        for (wid_id, event) in sent_events {
            if let Some(widget) = self.widgets.borrow().get(wid_id) {
                widget_handle_event(&widget, &mut *(ctx.borrow_mut()), &event);
            }
        }

        if old_active != notifier.active() {
            if notifier.active().is_some() {
                self.do_auto_hide(notifier.active());
            }
        }
    }

    fn draw(&mut self, painter: &mut Painter) {
        let notifier = self.notifier.clone();
        let need_redraw = notifier.need_redraw();

        let origin = Rect::from(0.0, 0.0, self.win_w, self.win_h);

        if need_redraw {
            self.ftm.start_measure();
            if self.fb.is_some() {
                painter.start_label_collector();
            }

            if notifier.is_layout_changed() {
                self.relayout();
            }

            notifier.swap_redraw(&mut self.cur_redraw);
            notifier.clear_redraw();
            self.ftm.end_measure();

            println!("redraw (lbl={}): {:?} ", self.fb.is_some(), self.cur_redraw.len());

            for layer in &self.layers {
                widget_draw(&layer.root, &self.cur_redraw, origin, painter);
                widget_draw_frame(&layer.root, painter);
            }

            if let Some(drag_widget) = &self.drag.widget {
                if self.drag.started {
                    let mut pos = drag_widget.pos();
                    let orig_pos = pos;
                    pos.w = pos.w * painter.dpi_factor;
                    pos.h = pos.h * painter.dpi_factor;
                    drag_widget.set_pos(pos);
                    widget_draw(drag_widget, &self.cur_redraw, origin, painter);
                    drag_widget.set_pos(orig_pos);
                }
            }

            if let Some(fb) = &mut self.fb {
                fb.apply_labels(painter.get_label_collection());
            }
        } else {
            for layer in &self.layers {
                widget_draw_shallow(&layer.root, false, origin, painter);
                widget_draw_frame(&layer.root, painter);
            }

            if let Some(drag_widget) = &self.drag.widget {
                if self.drag.started {
                    let mut pos = drag_widget.pos();
                    let orig_pos = pos;
                    pos.w = pos.w * painter.dpi_factor;
                    pos.h = pos.h * painter.dpi_factor;
                    drag_widget.set_pos(pos);
                    widget_draw(drag_widget, &self.cur_redraw, origin, painter);
                    drag_widget.set_pos(orig_pos);
                }
            }
        }
    }

    fn set_window_size(&mut self, w: f32, h: f32, dpi_factor: f32) {
        self.win_w = w;
        self.win_h = h;
        self.dpi_factor = dpi_factor;
        self.do_auto_hide(None);
        self.notifier.set_layout_changed();
        self.notifier.redraw(0);
    }

    fn get_image_data(&self) -> &HashMap<String, Vec<u8>> {
        &self.image_data
    }
}
