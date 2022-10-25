// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use super::ModifierTracker;
use crate::{EvPayload, Event, InputEvent, MButton, Widget};

use crate::style::*;

use crate::painter::*;
use crate::rect::*;

use std::cell::RefCell;
use std::rc::Rc;

pub trait ListModel {
    fn len(&self) -> usize;
    fn fmt_item<'a>(&self, item: usize, buf: &'a mut [u8]) -> Option<usize>;
    fn selected_item(&self) -> Option<usize>;
    fn deselect(&mut self);
    fn select(&mut self, idx: usize);

    /// Should return the generation counter for the internal data.
    /// The generation counter should increase for every change on the data.
    /// This is used by the widget to determine whether they need to be redrawn.
    fn get_generation(&mut self) -> u64;
}

pub struct ListData {
    items: Vec<String>,
    selected_item: Option<usize>,
    generation: u64,
}

impl ListData {
    pub fn new() -> Self {
        Self { items: vec![], generation: 0, selected_item: None }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.selected_item = None;
        self.generation += 1;
    }

    pub fn push(&mut self, item: String) {
        self.items.push(item);
        self.generation += 1;
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }
}

impl ListModel for ListData {
    fn len(&self) -> usize {
        self.items.len()
    }
    fn selected_item(&self) -> Option<usize> {
        self.selected_item
    }

    fn deselect(&mut self) {
        self.selected_item = None;
        self.generation += 1;
    }

    fn select(&mut self, idx: usize) {
        if idx < self.items.len() {
            self.selected_item = Some(idx);
            self.generation += 1;
        }
    }

    fn fmt_item<'a>(&self, index: usize, buf: &'a mut [u8]) -> Option<usize> {
        let item = self.items.get(index)?;

        use std::io::Write;
        let mut bw = std::io::BufWriter::new(buf);
        match write!(bw, "{}", item) {
            Ok(_) => Some(bw.buffer().len()),
            Err(_) => Some(0),
        }
    }
    fn get_generation(&mut self) -> u64 {
        self.generation
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListScrollMode {
    ItemCentered,
    Detached,
}

pub struct List {
    item_buf: [u8; 256],
    model: Rc<RefCell<dyn ListModel>>,
    real_pos: Rect,
    modkeys: ModifierTracker,
    hover: Option<i32>,
    item_areas: Vec<(i32, Rect)>,
    shown_item_count: usize,
    scroll_mode: ListScrollMode,
    scroll_page: usize,
}

impl List {
    pub fn new(model: Rc<RefCell<dyn ListModel>>, scroll_mode: ListScrollMode) -> Self {
        Self {
            model,
            item_buf: [0; 256],
            real_pos: Rect::from(0.0, 0.0, 0.0, 0.0),
            modkeys: ModifierTracker::new(),
            hover: None,
            item_areas: vec![],
            shown_item_count: 0,
            scroll_mode,
            scroll_page: 0,
        }
    }

    fn mouse_zone(&self) -> Option<i32> {
        self.zone(self.modkeys.mouse.x, self.modkeys.mouse.y)
    }

    fn zone(&self, x: f32, y: f32) -> Option<i32> {
        for ia in self.item_areas.iter() {
            if ia.1.is_inside(x, y) {
                return Some(ia.0 as i32);
            }
        }

        None
    }

    pub fn get_generation(&mut self) -> u64 {
        self.model.borrow_mut().get_generation()
    }

    pub fn calc_row_offs(&self, rows: usize, sel_item: i64) -> usize {
        let rows = rows as i64;
        let mut cur = sel_item;

        let margin = rows / 3;
        let margin = (margin / 2) * 2;
        let page_rows = rows - margin;

        if page_rows <= 0 {
            return cur as usize;
        }

        let mut scroll_page = 0;

        while cur >= page_rows {
            cur -= page_rows;
            scroll_page += 1;
        }

        if scroll_page > 0 {
            (scroll_page * page_rows - (margin / 2)) as usize
        } else {
            0
        }
    }

    fn handle_scroll(&mut self, w: &Widget, zone: i32, out_events: &mut Vec<(usize, Event)>) {
        let item_count = self.model.borrow_mut().len();
        let page_offs = self.shown_item_count / 2;

        match self.scroll_mode {
            ListScrollMode::ItemCentered => match zone {
                -1 => {
                    let mut cur =
                        self.model.borrow_mut().selected_item().unwrap_or(0);
                    if cur < page_offs {
                        cur = 0;
                    } else {
                        cur -= page_offs;
                    }

                    self.model.borrow_mut().select(cur);
                    out_events.push(w.event(
                        "select",
                        EvPayload::ItemSelect { index: cur as i32 },
                    ));
                }
                -2 => {
                    let mut cur =
                        self.model.borrow_mut().selected_item().unwrap_or(0);
                    if cur == 0 {
                        cur = item_count - 1;
                    } else {
                        cur = (cur - 1) % item_count;
                    }
                    self.model.borrow_mut().select(cur);
                    out_events.push(w.event(
                        "select",
                        EvPayload::ItemSelect { index: cur as i32 },
                    ));
                }
                -3 => {
                    let mut cur =
                        self.model.borrow_mut().selected_item().unwrap_or(0);
                    cur = (cur + 1) % item_count;
                    self.model.borrow_mut().select(cur);
                    out_events.push(w.event(
                        "select",
                        EvPayload::ItemSelect { index: cur as i32 },
                    ));
                }
                -4 => {
                    let mut cur =
                        self.model.borrow_mut().selected_item().unwrap_or(0);
                    cur += page_offs;
                    if item_count > 0 {
                        cur = cur.min(item_count - 1);
                    } else {
                        cur = 0;
                    }

                    self.model.borrow_mut().select(cur);
                    out_events.push(w.event(
                        "select",
                        EvPayload::ItemSelect { index: cur as i32 },
                    ));
                }
                _ => {}
            },
            ListScrollMode::Detached => match zone {
                -1 => {
                    if self.scroll_page > 0 {
                        self.scroll_page -= 1;
                    }
                }
                -4 => {
                    self.scroll_page += 1;
                }
                _ => {}
            },
        };
    }

    pub fn handle(&mut self, w: &Widget, event: &InputEvent, out_events: &mut Vec<(usize, Event)>) {
        let is_hovered = w.is_hovered();

        self.modkeys.handle(event);

        match event {
            InputEvent::MouseButtonPressed(MButton::Left) => {
                if !is_hovered {
                    return;
                }
                if self.mouse_zone().is_some() {
                    w.activate();
                    w.emit_redraw_required();
                }
            }
            InputEvent::MouseButtonReleased(MButton::Left) => {
                if w.is_active() {
                    if let Some(zone) = self.mouse_zone() {
                        if zone >= 0 {
                            self.model.borrow_mut().select(zone as usize);
                            out_events
                                .push(w.event("select", EvPayload::ItemSelect { index: zone }));
                        } else {
                            self.handle_scroll(w, zone, out_events);
                        }
                    }

                    w.emit_redraw_required();
                    w.deactivate();
                }
            }
            InputEvent::MouseWheel(y) => {
                if !is_hovered {
                    self.hover = None;
                    return;
                }
                if *y < 0.0 {
                    self.handle_scroll(w, -4, out_events);
                } else {
                    self.handle_scroll(w, -1, out_events);
                }
                w.emit_redraw_required();
            },
            InputEvent::MousePosition(_x, _y) => {
                if !is_hovered {
                    self.hover = None;
                    return;
                }

                let old_hover = self.hover;
                self.hover = self.mouse_zone();

                if old_hover != self.hover {
                    w.emit_redraw_required();
                }
            }
            _ => {}
        }
    }

    pub fn draw(
        &mut self,
        w: &Widget,
        style: &DPIStyle,
        pos: Rect,
        real_pos: Rect,
        p: &mut Painter,
    ) {
        let is_hovered = w.is_hovered();
        let is_active = w.is_active();

        let dpi_f = p.dpi_factor;
        let mut dbg = w.debug_tag();
        let (dx, dy) = (real_pos.x - pos.x, real_pos.y - pos.y);
        dbg.set_offs((dx, dy));
        self.real_pos = real_pos;

        let fh = p.font_height(style.font_size(), true);

        let pad = style.pad_item();

        let button_width = 30.0 * dpi_f;

        let buttons_pos = pos.crop_left(pos.w - button_width);
        let list_pos = pos.crop_right(button_width);

        let button_h = buttons_pos.h / 4.0;

        self.item_areas.clear();

        let mut draw_button = |index: usize| {
            let zone_idx = (index + 1) as i32 * -1;
            let mut color = style.border_color();

            if let Some(hover_idx) = self.hover {
                if hover_idx == zone_idx {
                    if is_active {
                        color = style.active_border_color();
                    } else {
                        color = style.hover_border_color();
                    }
                }
            }

            let y = button_h.floor() * (index as f32);
            let pos = Rect::from(buttons_pos.x, buttons_pos.y + y, buttons_pos.w, button_h.floor());
            p.rect_border_fill_r(style.border2(), color, style.bg_color(), pos);

            let lbl = match index {
                0 => "⇑",
                1 => "↑",
                2 => "↓",
                3 => "⇓",
                _ => "↑",
            };

            p.label(
                style.font_size() * 2.0,
                0,
                color,
                pos.x,
                pos.y,
                pos.w,
                pos.h,
                lbl,
                dbg.source("button"),
            );

            self.item_areas.push((zone_idx, pos.offs(dx, dy)));
        };

        draw_button(0);
        if self.scroll_mode != ListScrollMode::Detached {
            draw_button(1);
            draw_button(2);
        }
        draw_button(3);

        let line_height = 2.0 * pad + style.border2() + fh;
        let visible_lines = (list_pos.h / line_height).ceil() as usize;
        let dh = line_height;
        self.shown_item_count = visible_lines;

        let model = self.model.borrow_mut();
        //let item_count = model.len();
        let scroll_offs = match self.scroll_mode {
            ListScrollMode::ItemCentered => {
                self.calc_row_offs(visible_lines, model.selected_item().unwrap_or(0) as i64)
            }
            ListScrollMode::Detached => {
                let page_len = self.shown_item_count / 2;
                let max_page = model.len() / page_len;
                //d// println!("PAGE_LEN={}, max_page={}, vislines={}, items={}",
                //d//     page_len, max_page, visible_lines, item_count);
                if self.scroll_page > max_page {
                    self.scroll_page = max_page;
                }
                let offs = self.scroll_page * page_len;

                offs
            }
        };

        let selected_item = model.selected_item();

        p.clip_region(list_pos.x, list_pos.y, list_pos.w, list_pos.h);
        let mut y: f32 = 0.0;
        for row_idx in 0..visible_lines {
            let item_idx = row_idx + scroll_offs;

            let yd = y.round();
            p.stroke(
                style.border2(),
                style.color2(),
                &[(list_pos.x, list_pos.y + yd), (list_pos.x + list_pos.w, list_pos.y + yd)],
                false,
            );

            if let Some(len) = model.fmt_item(item_idx, &mut self.item_buf[..]) {
                let mut item_pos = list_pos;
                item_pos.y += yd;
                item_pos.h = dh;
                let item_outer = item_pos;
                let item_pos = item_pos.shrink(pad, pad);

                let mut color = style.color();
                if Some(item_idx) == selected_item {
                    color = style.selected_color();
                }
                if is_hovered {
                    if let Some(hover_idx) = self.hover {
                        if hover_idx >= 0 {
                            if hover_idx == (item_idx as i32) {
                                if is_active {
                                    color = style.active_color();
                                } else {
                                    color = style.hover_color();
                                }
                            }
                        }
                    }
                }

                let item_s = std::str::from_utf8(&self.item_buf[0..len]).unwrap();
                p.label(
                    style.font_size(),
                    -1,
                    color,
                    item_pos.x,
                    item_pos.y,
                    item_pos.w,
                    item_pos.h,
                    item_s,
                    dbg.source("item"),
                );

                self.item_areas.push((item_idx as i32, item_outer.offs(dx, dy)));
            }

            y += dh;
        }
        p.reset_clip_region();
    }
}
