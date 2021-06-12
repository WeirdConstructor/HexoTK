// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use crate::constants::*;
use super::*;
use super::util::*;

use std::rc::Rc;

#[derive(Debug)]
pub struct Tabs {
}

impl Tabs {
    pub fn new_ref() -> Rc<Self> {
        Rc::new(Self { })
    }
}

pub struct TabsData {
    tabs:            Vec<WidgetData>,
    labels:          Vec<String>,
    level:           usize,
    shrink:          (f64, f64),
}

impl TabsData {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            tabs:               vec![],
            labels:             vec![],
            level:              0,
            shrink:             (2.0, 2.0),
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

    pub fn add(&mut self, label: &str, widget_data: WidgetData) -> &mut Self {
        self.tabs.push(widget_data);
        self.labels.push(label.to_string());

        self
    }
}

impl WidgetType for Tabs {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {

        let id = data.id();

        data.with(|data: &mut TabsData| {
            let index =
                ui.atoms().get(id).map(|at| at.i()).unwrap_or(0) as usize;

            let bg_clr =
                match data.level {
                    0 => UI_BG_CLR,
                    1 => UI_BG2_CLR,
                    2 => UI_BG3_CLR,
                    _ => UI_BG3_CLR,
                };

            let pos = pos.shrink(data.shrink.0, data.shrink.1);

            let mut indexed_tab_pos = None;

            for (i, label) in data.labels.iter().enumerate() {
                let x = i as f64 * (UI_TAB_WIDTH + UI_TAB_PAD_WIDTH);
                let tab_pos = Rect {
                    x: pos.x + x,
                    y: pos.y,
                    w: UI_TAB_WIDTH,
                    h: UI_TAB_HEIGHT,
                };

                if index == i {
                    indexed_tab_pos = Some(tab_pos);
                }

                ui.define_active_zone(
                    ActiveZone::new_indexed_click_zone(id, tab_pos, i));

                let highlight = ui.hl_style_for(id, Some(i));

                let (text_color, border_color) =
                    match highlight {
                        HLStyle::Hover(_) =>
                            (UI_TAB_TXT_HOVER_CLR, UI_TAB_TXT_HOVER_CLR),
                        _ =>
                            (if i == index { UI_TAB_TXT_CLR }
                             else          { UI_TAB_TXT2_CLR },
                             UI_TAB_BORDER_CLR),
                    };

                let tab_pos =
                    rect_border(
                        p, UI_TAB_BORDER_WIDTH, border_color,
                        UI_TAB_BG_CLR, tab_pos);

                p.rect_fill(
                    UI_TAB_BG_CLR, tab_pos.x, tab_pos.y, tab_pos.w, tab_pos.h);
                p.label(UI_TAB_FONT_SIZE, 0, text_color,
                    tab_pos.x, tab_pos.y, tab_pos.w, tab_pos.h,
                    &label);
            }

            let pos = pos.crop_top(UI_TAB_HEIGHT - UI_TAB_BORDER_WIDTH);

            let pos =
                rect_border(p, UI_BORDER_WIDTH, UI_TAB_BORDER_CLR, bg_clr, pos);

            if let Some(tab_pos) = indexed_tab_pos {
                p.path_stroke(
                    UI_BORDER_WIDTH,
                    UI_TAB_DIV_CLR,
                    &mut [
                        (tab_pos.x + UI_TAB_BORDER_WIDTH,
                         tab_pos.y + tab_pos.h),
                        (tab_pos.x + tab_pos.w - UI_TAB_BORDER_WIDTH * 2.0,
                         tab_pos.y + tab_pos.h),
                    ].iter().copied(),
                    false);
            }

            if let Some(tab) = data.tabs.get_mut(index as usize) {
                tab.draw(ui, p, pos);
            }
        });
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        let my_id = data.id();

        data.with(|data: &mut TabsData| {
            match ev {
                UIEvent::Click { id, index, .. } => {
                    if my_id == *id {
                        ui.atoms_mut().set(
                            my_id, Atom::setting(*index as i64));
                    }
                },
                _ => {}
            }

            let index = ui.atoms().get(my_id).map(|at| at.i()).unwrap_or(0);

            if let Some(tab) = data.tabs.get_mut(index as usize) {
                tab.event(ui, ev);
            }
        });
    }
}
