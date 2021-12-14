use crate::layout::{WidgetId, LayoutCache};
use crate::widget::{Widget, WidgetImpl, widget_walk};
use std::rc::{Weak, Rc};
use std::cell::RefCell;
use std::collections::HashMap;

use morphorm::Hierarchy;

pub struct WidgetStore {
    widgets:         Vec<Weak<RefCell<WidgetImpl>>>,
}

impl WidgetStore {
    pub fn new() -> Self {
        Self {
            widgets: vec![],
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
    }

    pub fn add_root(&mut self, root: &Widget) {
        widget_walk(root, |wid, parent, _is_first, _is_last| {
            if let Some(parent) = parent {
                wid.set_parent(parent);
            }

            self.widgets.push(wid.as_weak());
        });
    }

    pub fn get(&self, id: usize) -> Option<Widget> {
        let wid = self.widgets.get(id)?;
        Widget::from_weak(wid)
    }

    pub fn with_layout<R, F: FnOnce(&crate::widget::Layout) -> R>(&self, id: &WidgetId, f: F)
        -> Option<R>
    {
        self.get(id.id()).map(|w| w.with_layout(f))
    }
}

pub struct WidgetTree {
    store:  Rc<RefCell<WidgetStore>>,
    widgets: HashMap<usize, (Option<WidgetId>, bool, bool)>,
    widget_dfs_vec: Vec<WidgetId>,
}

impl WidgetTree {
    pub fn from_root(store: Rc<RefCell<WidgetStore>>, root: &Widget) -> Self {
        let mut widgets = HashMap::new();
        let mut widget_dfs_vec = vec![];

        widget_walk(root, |wid, _parent, is_first, is_last| {
            widget_dfs_vec.push(WidgetId::from(wid.id()));
            println!("Walk {}", wid.id());
            widgets.insert(wid.id(), (
                wid.parent().map(|w| WidgetId::from(w.id())),
                is_first,
                is_last,
            ));
        });

        Self {
            store,
            widgets,
            widget_dfs_vec,
        }
    }

    pub fn apply_layout_to_widgets(&self, cache: &LayoutCache) {
        for id in &self.widget_dfs_vec {
            let pos = cache.get_widget_rect_by_id(id).round();
            println!("apply_layout[{}] = {:?}", id.id(), pos);

            if let Some(widget) = self.store.borrow().get(id.id()) {
                widget.set_pos(pos);
            }
        }
    }
}

impl<'a> Hierarchy<'a> for WidgetTree {
    type Item = WidgetId;

    fn up_iter<F: FnMut(Self::Item)>(&'a self, mut f: F) {
        for id in self.widget_dfs_vec.iter().rev() {
            (f)(*id);
        }
    }

    fn down_iter<F: FnMut(Self::Item)>(&'a self, mut f: F) {
        for id in self.widget_dfs_vec.iter() {
            (f)(*id);
        }
    }

    fn child_iter<F: FnMut(Self::Item)>(&'a self, node: Self::Item, mut f: F) {
        let w = self.store.borrow().get(node.id()).unwrap();

        w.for_each_child(|w, _, _| {
            (f)(WidgetId::from(w.id()));
        });
    }

    fn parent(&self, node: Self::Item) -> Option<Self::Item> {
        self.widgets.get(&node.id()).map(|(w, _, _)| *w)?
    }

    fn is_first_child(&self, node: Self::Item) -> bool {
        self.widgets.get(&node.id())
            .map(|(_, is_first, _)| *is_first)
            .unwrap_or(false)
    }

    fn is_last_child(&self, node: Self::Item) -> bool {
        self.widgets.get(&node.id())
            .map(|(_, _, is_last)| *is_last)
            .unwrap_or(false)
    }
}

