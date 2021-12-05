use crate::widget_store::WidgetStore;
use crate::Rect;

use std::rc::Rc;
use std::cell::RefCell;

use morphorm::{
    Node, GeometryChanged, Cache,
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

    pub fn get_widget_rect_by_id(&self, id: &WidgetId) -> Rect {
        Rect {
            x: self.layouts[id.id()].posx,
            y: self.layouts[id.id()].posy,
            w: self.layouts[id.id()].width,
            h: self.layouts[id.id()].height,
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

    fn set_visible(&mut self, _node: Self::Item, _value: bool) {
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
        self.layouts[node.id].width = value;
    }
    fn set_height(&mut self, node: Self::Item, value: f32) {
        self.layouts[node.id].height = value;
    }
    fn set_posx(&mut self, node: Self::Item, value: f32) {
        self.layouts[node.id].posx = value;
    }
    fn set_posy(&mut self, node: Self::Item, value: f32) {
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

impl WidgetId {
    pub fn from(id: usize) -> Self {
        Self { id }
    }

    pub fn id(&self) -> usize { self.id }
}

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
        let w = store.borrow().get(self.id)?;
        Some(Units::Pixels(w.style().border))
    }
    fn border_right(&self, store: &'_ Self::Data) -> Option<Units> {
        let w = store.borrow().get(self.id)?;
        Some(Units::Pixels(w.style().border))
    }
    fn border_top(&self, store: &'_ Self::Data) -> Option<Units> {
        let w = store.borrow().get(self.id)?;
        Some(Units::Pixels(w.style().border))
    }
    fn border_bottom(&self, store: &'_ Self::Data) -> Option<Units> {
        let w = store.borrow().get(self.id)?;
        Some(Units::Pixels(w.style().border))
    }
}

