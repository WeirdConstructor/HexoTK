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

use morphorm::{
    Node, GeometryChanged, Cache, Hierarchy,
    LayoutType, PositionType, Units,
};

#[derive(Debug, Clone, Copy)]
pub struct CachedLayout {
    geometry_changed: GeometryChanged,

    width:            f32,
    height:           f32,
    posx:             f32,
    posy:             f32,

    left:             f32,
    right:            f32,
    top:              f32,
    bottom:           f32,

    new_width:        f32,
    new_height:       f32,

    child_width_max:  f32,
    child_width_sum:  f32,
    child_height_max: f32,
    child_height_sum: f32,
    grid_row_max:     f32,
    grid_col_max:     f32,

    horizontal_free_space: f32,
    vertical_free_space: f32,

    vertical_stretch_sum: f32,
    horizontal_stretch_sum: f32,

    stack_first_child:  bool,
    stack_last_child:  bool,
}

impl CachedLayout {
    pub fn new() -> Self {
        Self {
            geometry_changed: GeometryChanged::empty(),

            width:            0.0,
            height:           0.0,
            posx:             0.0,
            posy:             0.0,

            left:             0.0,
            right:            0.0,
            top:              0.0,
            bottom:           0.0,

            new_width:        0.0,
            new_height:       0.0,

            child_width_max:  0.0,
            child_width_sum:  0.0,
            child_height_max: 0.0,
            child_height_sum: 0.0,
            grid_row_max:     0.0,
            grid_col_max:     0.0,

            horizontal_free_space: 0.0,
            vertical_free_space: 0.0,

            vertical_stretch_sum: 0.0,
            horizontal_stretch_sum: 0.0,

            stack_first_child:  false,
            stack_last_child:  false,
        }
    }
}

pub struct LayoutCache {
    layouts: Vec<CachedLayout>,
    store:   Rc<RefCell<WidgetStore>>,
}

impl LayoutCache {
    pub fn new(store: Rc<RefCell<WidgetStore>>) -> Self {
        Self {
            layouts: vec![],
            store,
        }
    }

    pub fn clear_to_len(&mut self, len: usize) {
        self.layouts.clear();
        self.layouts.resize_with(len, || CachedLayout::new());
    }
}

impl Cache for LayoutCache {
    type Item = WidgetId;

    fn geometry_changed(&self, node: Self::Item) -> GeometryChanged {
        self.layouts[node.id].geometry_changed
    }

    fn visible(&self, node: Self::Item) -> bool {
        self.store.borrow().get(node.id).map(|w| {
            w.with_layout(|l| l.visible)
        }).unwrap_or(false)
    }

    fn width(&self, node: Self::Item) -> f32 {
        self.layouts[node.id].width
    }

    fn height(&self, node: Self::Item) -> f32 {
        self.layouts[node.id].height
    }

    fn posx(&self, node: Self::Item) -> f32 {
        self.layouts[node.id].posx
    }

    fn posy(&self, node: Self::Item) -> f32 {
        self.layouts[node.id].posy
    }

    fn left(&self, node: Self::Item) -> f32 {
        self.layouts[node.id].left
    }
    fn right(&self, node: Self::Item) -> f32 {
        self.layouts[node.id].right
    }
    fn top(&self, node: Self::Item) -> f32 {
        self.layouts[node.id].top
    }
    fn bottom(&self, node: Self::Item) -> f32 {
        self.layouts[node.id].bottom
    }

    fn new_width(&self, node: Self::Item) -> f32 {
        self.layouts[node.id].new_width
    }
    fn new_height(&self, node: Self::Item) -> f32 {
        self.layouts[node.id].new_height
    }

    fn child_width_max(&self, node: Self::Item) -> f32 {
        self.layouts[node.id].child_width_max
    }

    fn child_width_sum(&self, node: Self::Item) -> f32 {
        self.layouts[node.id].child_width_sum
    }

    fn child_height_max(&self, node: Self::Item) -> f32 {
        self.layouts[node.id].child_height_max
    }

    fn child_height_sum(&self, node: Self::Item) -> f32 {
        self.layouts[node.id].child_height_sum
    }

    fn grid_row_max(&self, node: Self::Item) -> f32 {
        self.layouts[node.id].grid_row_max
    }

    fn grid_col_max(&self, node: Self::Item) -> f32 {
        self.layouts[node.id].grid_col_max
    }

    fn set_grid_col_max(&mut self, node: Self::Item, value: f32) {
        self.layouts[node.id].grid_col_max = value;
    }

    fn set_grid_row_max(&mut self, node: Self::Item, value: f32) {
        self.layouts[node.id].grid_row_max = value;
    }

    fn set_visible(&mut self, node: Self::Item, value: bool) {
        // nop
    }

    fn set_geo_changed(&mut self, node: Self::Item, flag: GeometryChanged, value: bool) {
        self.layouts[node.id].geometry_changed.set(flag, value);
    }

    fn set_child_width_sum(&mut self, node: Self::Item, value: f32) {
        self.layouts[node.id].child_width_sum = value;
    }
    fn set_child_height_sum(&mut self, node: Self::Item, value: f32) {
        self.layouts[node.id].child_height_sum = value;
    }
    fn set_child_width_max(&mut self, node: Self::Item, value: f32) {
        self.layouts[node.id].child_width_max = value;
    }
    fn set_child_height_max(&mut self, node: Self::Item, value: f32) {
        self.layouts[node.id].child_height_max = value;
    }

    fn horizontal_free_space(&self, node: Self::Item) -> f32 {
        self.layouts[node.id].horizontal_free_space
    }
    fn set_horizontal_free_space(&mut self, node: Self::Item, value: f32) {
        self.layouts[node.id].horizontal_free_space = value;
    }
    fn vertical_free_space(&self, node: Self::Item) -> f32 {
        self.layouts[node.id].vertical_free_space
    }
    fn set_vertical_free_space(&mut self, node: Self::Item, value: f32) {
        self.layouts[node.id].vertical_free_space = value;
    }

    fn horizontal_stretch_sum(&self, node: Self::Item) -> f32 {
        self.layouts[node.id].horizontal_stretch_sum
    }
    fn set_horizontal_stretch_sum(&mut self, node: Self::Item, value: f32) {
        self.layouts[node.id].horizontal_stretch_sum = value;
    }
    fn vertical_stretch_sum(&self, node: Self::Item) -> f32 {
        self.layouts[node.id].vertical_stretch_sum
    }
    fn set_vertical_stretch_sum(&mut self, node: Self::Item, value: f32) {
        self.layouts[node.id].vertical_stretch_sum = value;
    }

    fn set_width(&mut self, node: Self::Item, value: f32) {
        self.store.borrow().get(node.id).map(|w| {
            let mut pos = w.pos();
            pos.w = value;
            w.set_pos(pos)
        });
        println!("set posw={}", value);
        self.layouts[node.id].width = value;
    }
    fn set_height(&mut self, node: Self::Item, value: f32) {
        self.store.borrow().get(node.id).map(|w| {
            let mut pos = w.pos();
            pos.h = value;
            w.set_pos(pos)
        });
        println!("set posh={}", value);
        self.layouts[node.id].height = value;
    }
    fn set_posx(&mut self, node: Self::Item, value: f32) {
        self.store.borrow().get(node.id).map(|w| {
            let mut pos = w.pos();
            pos.x = value;
            w.set_pos(pos)
        });
        println!("set posx={}", value);
        self.layouts[node.id].posx = value;
    }
    fn set_posy(&mut self, node: Self::Item, value: f32) {
        self.store.borrow().get(node.id).map(|w| {
            let mut pos = w.pos();
            pos.y = value;
            w.set_pos(pos)
        });
        println!("set posy={}", value);
        self.layouts[node.id].posy = value;
    }

    fn set_left(&mut self, node: Self::Item, value: f32) {
        self.layouts[node.id].left = value;
    }
    fn set_right(&mut self, node: Self::Item, value: f32) {
        self.layouts[node.id].right = value;
    }
    fn set_top(&mut self, node: Self::Item, value: f32) {
        self.layouts[node.id].top = value;
    }
    fn set_bottom(&mut self, node: Self::Item, value: f32) {
        self.layouts[node.id].bottom = value;
    }

    fn set_new_width(&mut self, node: Self::Item, value: f32) {
        self.layouts[node.id].new_width = value;
    }
    fn set_new_height(&mut self, node: Self::Item, value: f32) {
        self.layouts[node.id].new_height = value;
    }

    fn stack_first_child(&self, node: Self::Item) -> bool {
        self.layouts[node.id].stack_first_child
    }
    fn set_stack_first_child(&mut self, node: Self::Item, value: bool) {
        self.layouts[node.id].stack_first_child = value;
    }
    fn stack_last_child(&self, node: Self::Item) -> bool {
        self.layouts[node.id].stack_last_child
    }
    fn set_stack_last_child(&mut self, node: Self::Item, value: bool) {
        self.layouts[node.id].stack_last_child = value;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WidgetId { id: usize }

impl Node<'_> for WidgetId {
    type Data = Rc<RefCell<WidgetStore>>;

    fn layout_type(&self, store: &'_ Self::Data) -> Option<LayoutType> {
        store.borrow().with_layout(self, |l| l.layout_type)
    }

    fn position_type(&self, store: &'_ Self::Data) -> Option<PositionType> {
        store.borrow().with_layout(self, |l| l.position_type)
    }

    fn width(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.width)
    }

    fn height(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.height)
    }

    fn min_width(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.min_width)
    }

    fn min_height(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.min_height)
    }

    fn max_width(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.max_width)
    }

    fn max_height(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.max_height)
    }

    fn left(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.left)
    }

    fn right(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.right)
    }

    fn top(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.top)
    }

    fn bottom(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.bottom)
    }

    fn min_left(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.min_left)
    }

    fn max_left(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.max_left)
    }

    fn min_right(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.min_right)
    }

    fn max_right(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.max_right)
    }

    fn min_top(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.min_top)
    }

    fn max_top(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.max_top)
    }

    fn min_bottom(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.min_bottom)
    }

    fn max_bottom(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.max_bottom)
    }

    fn child_left(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.child_left)
    }

    fn child_right(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.child_right)
    }

    fn child_top(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.child_top)
    }

    fn child_bottom(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.child_bottom)
    }

    fn row_between(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.row_between)
    }

    fn col_between(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.col_between)
    }

    fn grid_rows(&self, store: &'_ Self::Data) -> Option<Vec<Units>> {
        store.borrow().with_layout(self, |l| l.grid_rows.clone())
    }

    fn grid_cols(&self, store: &'_ Self::Data) -> Option<Vec<Units>> {
        store.borrow().with_layout(self, |l| l.grid_cols.clone())
    }

    fn row_index(&self, store: &'_ Self::Data) -> Option<usize> {
        store.borrow().with_layout(self, |l| l.row_index)
    }

    fn col_index(&self, store: &'_ Self::Data) -> Option<usize> {
        store.borrow().with_layout(self, |l| l.col_index)
    }
    fn row_span(&self, store: &'_ Self::Data) -> Option<usize> {
        store.borrow().with_layout(self, |l| l.row_span)
    }
    fn col_span(&self, store: &'_ Self::Data) -> Option<usize> {
        store.borrow().with_layout(self, |l| l.col_span)
    }
    fn border_left(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.border_left)
    }
    fn border_right(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.border_right)
    }
    fn border_top(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.border_top)
    }
    fn border_bottom(&self, store: &'_ Self::Data) -> Option<Units> {
        store.borrow().with_layout(self, |l| l.border_bottom)
    }
}

pub struct HierarchyNode {
    is_first:   bool,
    is_last:    bool,
    parent:     Option<Weak<RefCell<WidgetImpl>>>,
    id:         WidgetId,
}

pub struct WidgetStore {
    widgets:         Vec<Weak<RefCell<WidgetImpl>>>,
    hierarchy_nodes: Vec<HierarchyNode>,
}

impl WidgetStore {
    pub fn new() -> Self {
        Self {
            widgets:         vec![],
            hierarchy_nodes: vec![],
        }
    }

    pub fn for_each_widget<F: FnMut(Widget, usize)>(&self, mut f: F) {
        for (id, w) in self.widgets.iter().enumerate() {
            if let Some(w) = Widget::from_weak(w) {
                f(w, id);
            }
        }
    }

    pub fn for_each_widget_impl<F: FnMut(&mut WidgetImpl, usize)>(&self, mut f: F) {
        for (id, w) in self.widgets.iter().enumerate() {
            if let Some(w) = w.upgrade() {
                f(&mut w.borrow_mut(), id);
            }
        }
    }

    pub fn len(&self) -> usize { self.widgets.len() }

    pub fn clear(&mut self) {
        self.widgets.clear();
        self.hierarchy_nodes.clear();
    }

    pub fn add_root(&mut self, root: &Widget) {
        widget_walk(root, |wid, parent, is_first, is_last| {
            if let Some(parent) = parent {
                wid.set_parent(parent);
            }

            self.widgets.push(wid.as_weak());
            self.hierarchy_nodes.push(HierarchyNode {
                is_first,
                is_last,
                parent: parent.map(|p| p.as_weak()),
                id: WidgetId { id: self.widgets.len() - 1 },
            });
        });
    }

    pub fn get(&self, id: usize) -> Option<Widget> {
        let wid = self.widgets.get(id)?;
        Widget::from_weak(wid)
    }

    pub fn with_layout<R, F: FnOnce(&crate::widget::Layout) -> R>(&self, id: &WidgetId, f: F)
        -> Option<R>
    {
        self.get(id.id).map(|w| w.with_layout(f))
    }
}

impl<'a> Hierarchy<'a> for WidgetStore {
    type Item = WidgetId;
    type DownIter =
        std::iter::Map<
            std::slice::Iter<'a, Weak<RefCell<WidgetImpl>>>,
            fn(&std::rc::Weak<RefCell<WidgetImpl>>) -> WidgetId
        >;
    type UpIter =
        std::iter::Map<
            std::iter::Rev<std::slice::Iter<'a, Weak<RefCell<WidgetImpl>>>>,
            fn(&std::rc::Weak<RefCell<WidgetImpl>>) -> WidgetId
        >;
    type ChildIter = std::vec::IntoIter<WidgetId>;

    fn up_iter(&'a self) -> Self::UpIter {
        println!("up iter!");
        self.widgets.iter().rev().map(|w| WidgetId { id: Widget::from_weak(w).unwrap().id() })
    }

    fn down_iter(&'a self) -> Self::DownIter {
        println!("down iter!");
        self.widgets.iter().map(|w| WidgetId { id: Widget::from_weak(w).unwrap().id() })
    }

    fn child_iter(&self, node: Self::Item) -> Self::ChildIter {
        let w = self.widgets.get(node.id).unwrap();

        let mut v = vec![];
        w.upgrade().unwrap().borrow().for_each_child(|_, id, _| {
            v.push(WidgetId { id });
        });

        //d// println!("child iter {:?}!", v);

        v.into_iter()
    }

    fn parent(&self, node: Self::Item) -> Option<Self::Item> {
        let w = self.widgets.get(node.id)?;
        println!("parent of {}", node.id);
        let parent = w.upgrade()?.borrow().parent()?;
        Some(WidgetId { id: parent.id() })
    }

    fn is_first_child(&self, node: Self::Item) -> bool {
        self.hierarchy_nodes[node.id].is_first
    }

    fn is_last_child(&self, node: Self::Item) -> bool {
        self.hierarchy_nodes[node.id].is_last
    }
}

pub struct UI {
    win_w:              f32,
    win_h:              f32,
    layers:             Vec<Widget>,
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
        self.layers.push(root);
        self.on_tree_changed();
    }

    pub fn relayout(&mut self) {
        for root in &self.layers {
            let store = self.widgets.borrow();
            root.change_layout(|l| {
                l.left   = Units::Pixels(0.0);
                l.right  = Units::Pixels(0.0);
                l.width  = Units::Pixels(self.win_w);
                l.height = Units::Pixels(self.win_h);
            });
            println!("start relayout");
            morphorm::layout(&mut self.layout_cache, &*store, &self.widgets.clone());
//            root.relayout(Rect {
//                x: 0.0,
//                y: 0.0,
//                w: self.win_w,
//                h: self.win_h,
//            });
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

        for root in &self.layers {
            self.widgets.borrow_mut().add_root(root);
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
        for root in &self.layers {
            widget_draw(&root, &self.cur_redraw, painter);
        }
    }

    fn draw_frame(&mut self, painter: &mut Painter) {
        for root in &self.layers {
            widget_draw_frame(root, painter);
        }
    }

    fn set_window_size(&mut self, w: f32, h: f32) {
        self.win_w = w;
        self.win_h = h;
        self.notifier.set_layout_changed();
        self.notifier.redraw(0);
    }
}
