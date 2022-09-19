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
    fn selected_item(&self) -> Option<usize> { self.selected_item }

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

pub struct List {
    item_buf: [u8; 256],
    model: Rc<RefCell<dyn ListModel>>,
    real_pos: Rect,
    modkeys: ModifierTracker,
    hover: Option<i32>,
    item_areas: Vec<(usize, Rect)>,
}

impl List {
    pub fn new(model: Rc<RefCell<dyn ListModel>>) -> Self {
        Self {
            model,
            item_buf: [0; 256],
            real_pos: Rect::from(0.0, 0.0, 0.0, 0.0),
            modkeys: ModifierTracker::new(),
            hover: None,
            item_areas: vec![],
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
                            out_events.push(w.event(
                                "select",
                                EvPayload::ItemSelect { index: zone },
                            ));
                        }
                    }

                    w.emit_redraw_required();
                    w.deactivate();
                }
            }
            InputEvent::MousePosition(x, y) => {
                if !is_hovered {
                    self.hover = None;
                    return;
                }

                let old_hover = self.hover;
                self.hover = self.mouse_zone();

                if old_hover != self.hover {
                    println!("HOVER: {:?}", self.hover);
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

        let button_width = 30.0;

        let button_pos = pos.crop_left(pos.w - button_width);
        let list_pos = pos.crop_right(button_width);

        let visible_lines = (list_pos.h / fh).floor() as usize;
        let dh = 2.0 * pad + style.border2() + list_pos.h / (visible_lines as f32);

        let mut model = self.model.borrow_mut();

        self.item_areas.clear();

        let mut selected_item = model.selected_item();

        p.clip_region(list_pos.x, list_pos.y, list_pos.w, list_pos.h);
        let mut y: f32 = 0.0;
        for row_idx in 0..visible_lines {
            let yd = y.round();
            p.stroke(
                style.border2(),
                style.color2(),
                &[(list_pos.x, yd), (list_pos.x + list_pos.w, yd)],
                false,
            );

            if let Some(len) = model.fmt_item(row_idx, &mut self.item_buf[..]) {
                let mut item_pos = list_pos;
                item_pos.y = yd;
                item_pos.h = dh;
                let item_outer = item_pos;
                let item_pos = item_pos.shrink(pad, pad);

                let mut color = style.color();
                if Some(row_idx) == selected_item {
                    color = style.selected_color();
                }
                if is_hovered {
                    if let Some(hover_idx) = self.hover {
                        if hover_idx >= 0 {
                            if hover_idx == (row_idx as i32) {
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

                self.item_areas.push((row_idx, item_outer.offs(dx, dy)));
            }

            y += dh;
        }
        p.reset_clip_region();
    }
}
