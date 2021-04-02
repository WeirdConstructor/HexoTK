// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use crate::constants::*;
use super::*;

#[derive(Debug)]
pub struct Tabs {
}

impl Tabs {
    pub fn new() -> Self {
        Self { }
    }
}

pub struct TabsData {
    tabs:            Vec<WidgetData>,
    level:           usize,
    shrink:          (f64, f64),
}

impl TabsData {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            tabs:               vec![],
            level:              0,
            shrink:             (0.0, 0.0),
        })
    }

    pub fn level(&mut self, level: usize) -> &mut Self {
        self.level = level;
        self
    }

    pub fn shrink(&mut self, w: f64, h: f64) -> &mut Self {
        self.shrink = (w, h);
        self
    }

    pub fn add(&mut self, widget_data: WidgetData) -> &mut Self {
        if self.tabs.len() > 0 {
            let last_idx = self.tabs.len() - 1;
            self.tabs[last_idx].push(widget_data);
        }

        self
    }
}

impl WidgetType for Tabs {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {

        let id = data.id();

        data.with(|data: &mut TabsData| {
            let index = ui.atoms().get(id).map(|at| at.i()).unwrap_or(0);

            let bg_clr =
                match data.level {
                    0 => UI_BG_CLR,
                    1 => UI_BG2_CLR,
                    2 => UI_BG3_CLR,
                    _ => UI_BG3_CLR,
                };

            let pos = pos.shrink(data.shrink.0, data.shrink.1);

            // 0. draw outer border so that it is under the underlined
            //    tab.
            // 1. dark BG tab titles
            // 2. text color and underline the active tab
            // 3a. define active zones for each tab Click with index
            // 3b. detect hovered tab
            // 3c. hover highlight the hovered tab
            // 3d. Handle the click in the event callback and set the
            //     atom setting to the index.
            // 4. draw the selected child widget

            if let Some(tab) = data.tabs.get_mut(index as usize) {
                tab.draw(ui, p, pos);
            }
        });
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        data.with(|data: &mut TabsData| {
            let index = ui.atoms().get(id).map(|at| at.i()).unwrap_or(0);

            if let Some(tab) = data.tabs.get_mut(index as usize) {
                tab.event(ui, ev);
            }
        });
    }
}
