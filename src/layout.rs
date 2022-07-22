// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::widget_store::WidgetStore;
use crate::Rect;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use morphorm::{Cache, GeometryChanged, LayoutType, Node, PositionType, Units};

#[derive(Debug, Default, Clone, Copy)]
pub struct CachedLayout {
    geometry_changed: GeometryChanged,

    width: f32,
    height: f32,
    posx: f32,
    posy: f32,

    left: f32,
    right: f32,
    top: f32,
    bottom: f32,

    new_width: f32,
    new_height: f32,

    child_width_max: f32,
    child_width_sum: f32,
    child_height_max: f32,
    child_height_sum: f32,
    grid_row_max: f32,
    grid_col_max: f32,

    horizontal_free_space: f32,
    vertical_free_space: f32,

    vertical_stretch_sum: f32,
    horizontal_stretch_sum: f32,

    stack_first_child: bool,
    stack_last_child: bool,
}

pub struct LayoutCache {
    layouts: HashMap<usize, CachedLayout>,
    store: Rc<RefCell<WidgetStore>>,
}

impl LayoutCache {
    pub fn new(store: Rc<RefCell<WidgetStore>>) -> Self {
        Self { layouts: HashMap::new(), store }
    }

    pub fn get_widget_rect_by_id(&self, w_id: &WidgetId) -> Rect {
        if let Some(layout) = self.layouts.get(&w_id.unique_id()) {
            Rect { x: layout.posx, y: layout.posy, w: layout.width, h: layout.height }
        } else {
            Rect::from(0.0, 0.0, 0.0, 0.0)
        }
    }

    pub fn layout_mut<R: Default, F: FnOnce(&mut CachedLayout) -> R>(
        &mut self,
        w_id: &WidgetId,
        f: F,
    ) -> R {
        if let Some(layout) = self.layouts.get_mut(&w_id.unique_id()) {
            f(layout)
        } else {
            R::default()
        }
    }

    pub fn layout<R: Default, F: FnOnce(&CachedLayout) -> R>(&self, w_id: &WidgetId, f: F) -> R {
        if let Some(layout) = self.layouts.get(&w_id.unique_id()) {
            f(layout)
        } else {
            R::default()
        }
    }

    pub fn clear(&mut self) {
        self.layouts.clear();
        self.store.borrow_mut().for_each_widget(|wid| {
            self.layouts.insert(wid.unique_id(), CachedLayout::default());
        });
    }
}

impl Cache for LayoutCache {
    type Item = WidgetId;

    fn geometry_changed(&self, node: Self::Item) -> GeometryChanged {
        self.layout(&node, |l| l.geometry_changed)
    }

    fn visible(&self, node: Self::Item) -> bool {
        self.store
            .borrow()
            .get(node.unique_id())
            .map(|w| w.with_layout(|l| l.visible))
            .unwrap_or(false)
    }

    fn width(&self, node: Self::Item) -> f32 {
        self.layout(&node, |l| l.width)
    }

    fn height(&self, node: Self::Item) -> f32 {
        self.layout(&node, |l| l.height)
    }

    fn posx(&self, node: Self::Item) -> f32 {
        self.layout(&node, |l| l.posx)
    }

    fn posy(&self, node: Self::Item) -> f32 {
        self.layout(&node, |l| l.posy)
    }

    fn left(&self, node: Self::Item) -> f32 {
        self.layout(&node, |l| l.left)
    }
    fn right(&self, node: Self::Item) -> f32 {
        self.layout(&node, |l| l.right)
    }
    fn top(&self, node: Self::Item) -> f32 {
        self.layout(&node, |l| l.top)
    }
    fn bottom(&self, node: Self::Item) -> f32 {
        self.layout(&node, |l| l.bottom)
    }

    fn new_width(&self, node: Self::Item) -> f32 {
        self.layout(&node, |l| l.new_width)
    }
    fn new_height(&self, node: Self::Item) -> f32 {
        self.layout(&node, |l| l.new_height)
    }

    fn child_width_max(&self, node: Self::Item) -> f32 {
        self.layout(&node, |l| l.child_width_max)
    }

    fn child_width_sum(&self, node: Self::Item) -> f32 {
        self.layout(&node, |l| l.child_width_sum)
    }

    fn child_height_max(&self, node: Self::Item) -> f32 {
        self.layout(&node, |l| l.child_height_max)
    }

    fn child_height_sum(&self, node: Self::Item) -> f32 {
        self.layout(&node, |l| l.child_height_sum)
    }

    fn grid_row_max(&self, node: Self::Item) -> f32 {
        self.layout(&node, |l| l.grid_row_max)
    }

    fn grid_col_max(&self, node: Self::Item) -> f32 {
        self.layout(&node, |l| l.grid_col_max)
    }

    fn set_grid_col_max(&mut self, node: Self::Item, value: f32) {
        self.layout_mut(&node, |l| l.grid_col_max = value)
    }

    fn set_grid_row_max(&mut self, node: Self::Item, value: f32) {
        self.layout_mut(&node, |l| l.grid_row_max = value)
    }

    fn set_visible(&mut self, _node: Self::Item, _value: bool) {
        // nop
    }

    fn set_geo_changed(&mut self, node: Self::Item, flag: GeometryChanged, value: bool) {
        self.layout_mut(&node, |l| l.geometry_changed.set(flag, value))
    }

    fn set_child_width_sum(&mut self, node: Self::Item, value: f32) {
        self.layout_mut(&node, |l| l.child_width_sum = value)
    }
    fn set_child_height_sum(&mut self, node: Self::Item, value: f32) {
        self.layout_mut(&node, |l| l.child_height_sum = value)
    }
    fn set_child_width_max(&mut self, node: Self::Item, value: f32) {
        self.layout_mut(&node, |l| l.child_width_max = value)
    }
    fn set_child_height_max(&mut self, node: Self::Item, value: f32) {
        self.layout_mut(&node, |l| l.child_height_max = value)
    }

    fn horizontal_free_space(&self, node: Self::Item) -> f32 {
        self.layout(&node, |l| l.horizontal_free_space)
    }
    fn set_horizontal_free_space(&mut self, node: Self::Item, value: f32) {
        self.layout_mut(&node, |l| l.horizontal_free_space = value)
    }
    fn vertical_free_space(&self, node: Self::Item) -> f32 {
        self.layout(&node, |l| l.vertical_free_space)
    }
    fn set_vertical_free_space(&mut self, node: Self::Item, value: f32) {
        self.layout_mut(&node, |l| l.vertical_free_space = value)
    }

    fn horizontal_stretch_sum(&self, node: Self::Item) -> f32 {
        self.layout(&node, |l| l.horizontal_stretch_sum)
    }
    fn set_horizontal_stretch_sum(&mut self, node: Self::Item, value: f32) {
        self.layout_mut(&node, |l| l.horizontal_stretch_sum = value)
    }
    fn vertical_stretch_sum(&self, node: Self::Item) -> f32 {
        self.layout(&node, |l| l.vertical_stretch_sum)
    }
    fn set_vertical_stretch_sum(&mut self, node: Self::Item, value: f32) {
        self.layout_mut(&node, |l| l.vertical_stretch_sum = value)
    }

    fn set_width(&mut self, node: Self::Item, value: f32) {
        self.layout_mut(&node, |l| l.width = value)
    }
    fn set_height(&mut self, node: Self::Item, value: f32) {
        self.layout_mut(&node, |l| l.height = value)
    }
    fn set_posx(&mut self, node: Self::Item, value: f32) {
        self.layout_mut(&node, |l| l.posx = value)
    }
    fn set_posy(&mut self, node: Self::Item, value: f32) {
        self.layout_mut(&node, |l| l.posy = value)
    }

    fn set_left(&mut self, node: Self::Item, value: f32) {
        self.layout_mut(&node, |l| l.left = value)
    }
    fn set_right(&mut self, node: Self::Item, value: f32) {
        self.layout_mut(&node, |l| l.right = value)
    }
    fn set_top(&mut self, node: Self::Item, value: f32) {
        self.layout_mut(&node, |l| l.top = value)
    }
    fn set_bottom(&mut self, node: Self::Item, value: f32) {
        self.layout_mut(&node, |l| l.bottom = value)
    }

    fn set_new_width(&mut self, node: Self::Item, value: f32) {
        self.layout_mut(&node, |l| l.new_width = value)
    }
    fn set_new_height(&mut self, node: Self::Item, value: f32) {
        self.layout_mut(&node, |l| l.new_height = value)
    }

    fn stack_first_child(&self, node: Self::Item) -> bool {
        self.layout(&node, |l| l.stack_first_child)
    }
    fn set_stack_first_child(&mut self, node: Self::Item, value: bool) {
        self.layout_mut(&node, |l| l.stack_first_child = value)
    }
    fn stack_last_child(&self, node: Self::Item) -> bool {
        self.layout(&node, |l| l.stack_last_child)
    }
    fn set_stack_last_child(&mut self, node: Self::Item, value: bool) {
        self.layout_mut(&node, |l| l.stack_last_child = value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WidgetId {
    unique_id: usize,
}

impl WidgetId {
    pub fn from(unique_id: usize) -> Self {
        Self { unique_id }
    }

    pub fn unique_id(&self) -> usize {
        self.unique_id
    }
}

#[derive(Clone)]
pub struct LayoutTree {
    pub dpi_factor: f32,
    pub store: Rc<RefCell<WidgetStore>>,
}

macro_rules! get_size {
    ($self: ident, $tree: ident, $field: ident) => {
        match $tree.store.borrow().with_layout($self, |l| l.$field) {
            Some(Units::Pixels(px)) => Some(Units::Pixels(px * $tree.dpi_factor)),
            u => u,
        }
    };
}

impl Node<'_> for WidgetId {
    type Data = LayoutTree;

    fn layout_type(&self, tree: &'_ Self::Data) -> Option<LayoutType> {
        tree.store.borrow().with_layout(self, |l| l.layout_type)
    }

    fn position_type(&self, tree: &'_ Self::Data) -> Option<PositionType> {
        tree.store.borrow().with_layout(self, |l| l.position_type)
    }

    fn width(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, width)
    }

    fn height(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, height)
    }

    fn min_width(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, min_width)
    }

    fn min_height(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, min_height)
    }

    fn max_width(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, max_width)
    }

    fn max_height(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, max_height)
    }

    fn left(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, left)
    }

    fn right(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, right)
    }

    fn top(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, top)
    }

    fn bottom(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, bottom)
    }

    fn min_left(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, min_left)
    }

    fn max_left(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, max_left)
    }

    fn min_right(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, min_right)
    }

    fn max_right(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, max_right)
    }

    fn min_top(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, min_top)
    }

    fn max_top(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, max_top)
    }

    fn min_bottom(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, min_bottom)
    }

    fn max_bottom(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, max_bottom)
    }

    fn child_left(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, child_left)
    }

    fn child_right(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, child_right)
    }

    fn child_top(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, child_top)
    }

    fn child_bottom(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, child_bottom)
    }

    fn row_between(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, row_between)
    }

    fn col_between(&self, tree: &'_ Self::Data) -> Option<Units> {
        get_size!(self, tree, col_between)
    }

    fn grid_rows(&self, tree: &'_ Self::Data) -> Option<Vec<Units>> {
        tree.store.borrow().with_layout(self, |l| l.grid_rows.clone())
    }

    fn grid_cols(&self, tree: &'_ Self::Data) -> Option<Vec<Units>> {
        tree.store.borrow().with_layout(self, |l| l.grid_cols.clone())
    }

    fn row_index(&self, tree: &'_ Self::Data) -> Option<usize> {
        tree.store.borrow().with_layout(self, |l| l.row_index)
    }

    fn col_index(&self, tree: &'_ Self::Data) -> Option<usize> {
        tree.store.borrow().with_layout(self, |l| l.col_index)
    }
    fn row_span(&self, tree: &'_ Self::Data) -> Option<usize> {
        tree.store.borrow().with_layout(self, |l| l.row_span)
    }
    fn col_span(&self, tree: &'_ Self::Data) -> Option<usize> {
        tree.store.borrow().with_layout(self, |l| l.col_span)
    }
    fn border_left(&self, tree: &'_ Self::Data) -> Option<Units> {
        let w = tree.store.borrow().get(self.unique_id)?;
        let style = w.style();
        if w.has_default_style() {
            Some(Units::Pixels(tree.dpi_factor * (style.border + style.pad_left)))
        } else {
            Some(Units::Pixels(tree.dpi_factor * style.pad_left))
        }
    }
    fn border_right(&self, tree: &'_ Self::Data) -> Option<Units> {
        let w = tree.store.borrow().get(self.unique_id)?;
        let style = w.style();
        if w.has_default_style() {
            Some(Units::Pixels(tree.dpi_factor * (style.border + style.pad_right)))
        } else {
            Some(Units::Pixels(tree.dpi_factor * style.pad_right))
        }
    }
    fn border_top(&self, tree: &'_ Self::Data) -> Option<Units> {
        let w = tree.store.borrow().get(self.unique_id)?;
        let style = w.style();
        if w.has_default_style() {
            Some(Units::Pixels(tree.dpi_factor * (style.border + style.pad_top)))
        } else {
            Some(Units::Pixels(tree.dpi_factor * style.pad_top))
        }
    }
    fn border_bottom(&self, tree: &'_ Self::Data) -> Option<Units> {
        let w = tree.store.borrow().get(self.unique_id)?;
        let style = w.style();
        if w.has_default_style() {
            Some(Units::Pixels(tree.dpi_factor * (style.border + style.pad_bottom)))
        } else {
            Some(Units::Pixels(tree.dpi_factor * style.pad_bottom))
        }
    }
}
