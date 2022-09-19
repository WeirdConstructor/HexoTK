// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::{EvPayload, Event, InputEvent, MButton, Widget};
use super::ModifierTracker;

use crate::style::*;

use crate::painter::*;
use crate::rect::*;

use std::cell::RefCell;
use std::rc::Rc;

pub trait ListModel {
    fn len(&self) -> usize;
    fn fmt_item<'a>(&self, item: usize, buf: &'a mut [u8]) -> Option<usize>;

    /// Should return the generation counter for the internal data.
    /// The generation counter should increase for every change on the data.
    /// This is used by the widget to determine whether they need to be redrawn.
    fn get_generation(&mut self) -> u64;
}

pub struct ListData {
    items: Vec<String>,
    generation: u64,
}

impl ListData {
    pub fn new() -> Self {
        Self { items: vec![], generation: 0 }
    }

    pub fn clear(&mut self) {
        self.items.clear();
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

    pub fn get_generation(&mut self) -> u64 {
        self.model.borrow_mut().get_generation()
    }

    pub fn handle(&mut self, w: &Widget, event: &InputEvent, out_events: &mut Vec<(usize, Event)>) {
        let is_hovered = w.is_hovered();

        self.modkeys.handle(event);

        match event {
            InputEvent::MouseButtonPressed(MButton::Middle) => {
                if !is_hovered {
                    return;
                }
                w.activate();
            }
            InputEvent::MouseButtonReleased(MButton::Middle) => {
                if w.is_active() {
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

                for ia in self.item_areas.iter() {
                    if ia.1.is_inside(self.modkeys.mouse.x, self.modkeys.mouse.y) {
                        self.hover = Some(ia.0 as i32);
                    }
                }

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

        let list_pos = pos;

        let visible_lines = (list_pos.h / fh).floor() as usize;
        let dh = 2.0 * pad + style.border2() + list_pos.h / (visible_lines as f32);

        let mut model = self.model.borrow_mut();

        self.item_areas.clear();

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
                if is_hovered {
                    if let Some(hover_idx) = self.hover {
                        if hover_idx >= 0 {
                            if hover_idx == (row_idx as i32) {
                                println!("HOVER CLOLOR {}", row_idx);
                                color = style.hover_color();
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
