use crate::layout::{LayoutCache, WidgetId};
use crate::widget::{widget_walk, Widget, WidgetImpl};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use morphorm::Hierarchy;

pub struct WidgetStore {
    widgets: HashMap<usize, Weak<RefCell<WidgetImpl>>>,
}

impl WidgetStore {
    pub fn new() -> Self {
        Self { widgets: HashMap::new() }
    }

    pub fn for_each_widget<F: FnMut(Widget)>(&self, mut f: F) {
        for (_id, w) in self.widgets.iter() {
            if let Some(w) = Widget::from_weak(w) {
                f(w);
            }
        }
    }

    pub fn for_each_widget_impl<F: FnMut(&mut WidgetImpl)>(&self, mut f: F) {
        for (_id, w) in self.widgets.iter() {
            if let Some(w) = w.upgrade() {
                f(&mut w.borrow_mut());
            }
        }
    }

    pub fn clear(&mut self) {
        self.widgets.clear();
    }

    pub fn add_root(&mut self, root: &Widget, layer_idx: usize) {
        widget_walk(root, |wid, parent, _is_first, _is_last, depth| {
            if let Some(parent) = parent {
                wid.set_parent(parent);
            }

            wid.set_tree_pos(layer_idx, depth);
            self.widgets.insert(wid.unique_id(), wid.as_weak());
        });
    }

    pub fn get(&self, unique_id: usize) -> Option<Widget> {
        let wid = self.widgets.get(&unique_id)?;
        Widget::from_weak(wid)
    }

    pub fn with_layout<R, F: FnOnce(&crate::widget::Layout) -> Option<R>>(
        &self,
        w_id: &WidgetId,
        f: F,
    ) -> Option<R> {
        self.get(w_id.unique_id()).map(|w| w.with_layout(f)).flatten()
    }
}

pub struct WidgetTree {
    store: Rc<RefCell<WidgetStore>>,
    widgets: HashMap<usize, (Option<WidgetId>, bool, bool)>,
    widget_dfs_vec: Vec<WidgetId>,
}

impl WidgetTree {
    pub fn from_root(store: Rc<RefCell<WidgetStore>>, root: &Widget) -> Self {
        let mut widgets = HashMap::new();
        let mut widget_dfs_vec = vec![];

        widget_walk(root, |wid, _parent, is_first, is_last, _depth| {
            widget_dfs_vec.push(WidgetId::from(wid.unique_id()));
            widgets.insert(
                wid.unique_id(),
                (wid.parent().map(|w| WidgetId::from(w.unique_id())), is_first, is_last),
            );
        });

        Self { store, widgets, widget_dfs_vec }
    }

    pub fn apply_layout_to_widgets(&self, cache: &LayoutCache) {
        for w_id in &self.widget_dfs_vec {
            let pos = cache.get_widget_rect_by_id(w_id).round();
            //d// println!("apply_layout[{}] = {:?}", w_id.unique_id(), pos);

            if let Some(widget) = self.store.borrow().get(w_id.unique_id()) {
                widget.set_pos(pos);
            }
        }
    }
}

impl<'a> Hierarchy<'a> for WidgetTree {
    type Item = WidgetId;

    fn up_iter<F: FnMut(Self::Item)>(&'a self, mut f: F) {
        for w_id in self.widget_dfs_vec.iter().rev() {
            (f)(*w_id);
        }
    }

    fn down_iter<F: FnMut(Self::Item)>(&'a self, mut f: F) {
        for w_id in self.widget_dfs_vec.iter() {
            (f)(*w_id);
        }
    }

    fn child_iter<F: FnMut(Self::Item)>(&'a self, node: Self::Item, mut f: F) {
        let w = self.store.borrow().get(node.unique_id()).unwrap();

        w.for_each_child(|w, _, _| {
            (f)(WidgetId::from(w.unique_id()));
        });
    }

    fn parent(&self, node: Self::Item) -> Option<Self::Item> {
        self.widgets.get(&node.unique_id()).map(|(w, _, _)| *w)?
    }

    fn is_first_child(&self, node: Self::Item) -> bool {
        self.widgets.get(&node.unique_id()).map(|(_, is_first, _)| *is_first).unwrap_or(false)
    }

    fn is_last_child(&self, node: Self::Item) -> bool {
        self.widgets.get(&node.unique_id()).map(|(_, _, is_last)| *is_last).unwrap_or(false)
    }
}
