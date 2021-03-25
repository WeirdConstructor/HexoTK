// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use crate::constants::*;
use super::*;
use super::util::*;

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
pub struct List {
    font_size:  f64,
    lines:      usize,
    rect:       Rect,
}

#[derive(Debug, Clone, Copy)]
pub enum ListOutput {
    ByString,
    BySetting,
}

#[derive(Debug, Clone)]
pub struct ListItems {
    items: Rc<RefCell<Vec<(i64, String, String)>>>,
    width: usize,
}

impl ListItems {
    pub fn new(width: usize) -> Self {
        Self {
            items: Rc::new(RefCell::new(vec![])),
            width,
        }
    }

    pub fn push(&self, setting: i64, s: String) {
        let s_short =
            if s.len() > self.width {
                let s_short : String =
                    s.chars().take(self.width).collect();
                s_short
            } else {
                s.to_string()
            };
        self.items.borrow_mut().push((setting, s, s_short));
    }
}

#[derive(Debug, Clone)]
pub struct ListData {
    label:     String,
    items:     ListItems,
    out_mode:  ListOutput,
    offs:      usize,
}

impl ListData {
    pub fn new(label: &str, out_mode: ListOutput, items: ListItems) -> Box<dyn std::any::Any> {
        Box::new(Self {
            label: label.to_string(),
            items,
            out_mode,
            offs: 0,
        })
    }
}

impl List {
    pub fn new(width: f64, font_size: f64, lines: usize) -> Self {
        Self {
            lines,
            font_size,
            rect: Rect::from(
                0.0, 0.0,
                width
                + 2.0 * UI_PADDING + 2.0 * UI_BORDER_WIDTH,
                (lines as f64) * UI_ELEM_TXT_H
                + 2.0 * UI_PADDING + 2.0 * UI_BORDER_WIDTH + UI_ELEM_TXT_H)
        }
    }
}

impl WidgetType for List {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let pos       = self.rect.offs(pos.x, pos.y);
        let id        = data.id();
        let highlight = ui.hl_style_for(id, None);

        let border_color =
            match highlight {
                HLStyle::Hover(_) => UI_BTN_TXT_HOVER_CLR,
                _                 => UI_ENTRY_BORDER_CLR,
            };

        let pos =
            rect_border(p, UI_BORDER_WIDTH, border_color, UI_TAB_BG_CLR, pos);

        let pos = pos.shrink(UI_SAFETY_PAD, UI_SAFETY_PAD);

        data.with(|data: &mut ListData| {
            p.label(self.font_size, -1, UI_LBL_TXT_CLR,
                pos.x, pos.y, pos.w, UI_ELEM_TXT_H, &data.label);

            p.path_stroke(
                1.0,
                UI_LBL_TXT_CLR,
                &mut [
                    (pos.x         - UI_SAFETY_PAD, pos.y + UI_ELEM_TXT_H + 0.5),
                    (pos.x + pos.w + UI_SAFETY_PAD, pos.y + UI_ELEM_TXT_H + 0.5),
                ].iter().copied(),
                false);

            let pos = pos.crop_top(UI_ELEM_TXT_H);

            let mut yo = 0.0;
            for (idx, item) in
                data.items.items.borrow()
                    .iter()
                    .skip(data.offs)
                    .take(self.lines)
                    .enumerate()
            {
                let highlight = ui.hl_style_for(id, Some(idx + 2));
                let txt_color =
                    match highlight {
                        HLStyle::Hover(_) => UI_LIST_TXT_HOVER_CLR,
                        _                 => UI_LIST_TXT_CLR,
                    };

                let lpos =
                    Rect::from(pos.x, pos.y + yo, pos.w, UI_ELEM_TXT_H)
                    .crop_right(UI_LIST_BTN_WIDTH);
                p.label_mono(self.font_size, -1, txt_color,
                    lpos.x, lpos.y, lpos.w, lpos.h, &item.2);

                ui.define_active_zone(
                    ActiveZone::new_indexed_click_zone(id, lpos, 2 + idx));

                p.path_stroke(
                    1.0,
                    UI_LIST_SEP_CLR,
                    &mut [
                        (lpos.x         - UI_SAFETY_PAD, lpos.y + lpos.h + 0.5),
                        (lpos.x + pos.w + UI_SAFETY_PAD, lpos.y + lpos.h + 0.5),
                    ].iter().copied(),
                    false);

                yo += UI_ELEM_TXT_H;
            }

            let highlight_up   = ui.hl_style_for(id, Some(0));
            let highlight_down = ui.hl_style_for(id, Some(1));
            let txt_color_up =
                match highlight_up {
                    HLStyle::Hover(_) => UI_LIST_TXT_HOVER_CLR,
                    _                 => UI_LIST_SEP_CLR,
                };
            let txt_color_down =
                match highlight_down {
                    HLStyle::Hover(_) => UI_LIST_TXT_HOVER_CLR,
                    _                 => UI_LIST_SEP_CLR,
                };

            let pos = pos.offs(0.0, 1.0);

            let btn_up_pos =
                pos.crop_left(pos.w - UI_LIST_BTN_WIDTH)
                   .crop_bottom(pos.h * 0.5)
                   .offs(UI_SAFETY_PAD, 0.0);
            let btn_down_pos =
                pos.crop_left(pos.w - UI_LIST_BTN_WIDTH)
                   .crop_bottom(pos.h * 0.5)
                   .offs(UI_SAFETY_PAD, btn_up_pos.h);

            let btn_up_pos =
                rect_border(p,
                    UI_LIST_BTN_BORDER_WIDTH,
                    UI_LIST_SEP_CLR,
                    UI_TAB_BG_CLR,
                    btn_up_pos);
            if data.offs > 0 {
                ui.define_active_zone(
                    ActiveZone::new_indexed_click_zone(id, btn_up_pos, 0));
                draw_pointer(
                    p,
                    true,
                    UI_LIST_BTN_POINTER_SIZE,
                    txt_color_up,
                    btn_up_pos.center());
            }

            let btn_down_pos =
                rect_border(p,
                    UI_LIST_BTN_BORDER_WIDTH,
                    UI_LIST_SEP_CLR,
                    UI_TAB_BG_CLR,
                    btn_down_pos);
            ui.define_active_zone(
                ActiveZone::new_indexed_click_zone(id, btn_down_pos, 1));
            draw_pointer(
                p,
                false,
                UI_LIST_BTN_POINTER_SIZE,
                txt_color_down,
                btn_down_pos.center());
        });
    }

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _avail: (f64, f64)) -> (f64, f64) {
        (self.rect.w, self.rect.h)
    }

    fn event(&self, _ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        match ev {
            UIEvent::Click { id, index, .. } => {
                if data.id() == *id {
                    println!("CLICK LIST: {:?}", ev);
                }
            },
            _ => {},
        }
    }
}
