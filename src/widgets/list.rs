// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::{EvPayload, Event, InputEvent, MButton, Widget};

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
        Self {
            items: vec![],
            generation: 0,
        }
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
    fn len(&self) -> usize { self.items.len() }
    fn fmt_item<'a>(&self, index: usize, buf: &'a mut [u8]) -> Option<usize> {
        let item = self.items.get(index)?;

        use std::io::Write;
        let mut bw = std::io::BufWriter::new(buf);
        match write!(bw, "{}", item) {
            Ok(_) => Some(bw.buffer().len()),
            Err(_) => Some(0),
        }
    }
    fn get_generation(&mut self) -> u64 { self.generation }
}

pub struct List {
    item_buf: [u8; 256],
    model: Rc<RefCell<dyn ListModel>>,
    real_pos: Rect,
}

impl List {
    pub fn new(model: Rc<RefCell<dyn ListModel>>) -> Self {
        Self {
            model,
            item_buf: [0; 256],
            real_pos: Rect::from(0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn get_generation(&mut self) -> u64 {
        self.model.borrow_mut().get_generation()
    }

    pub fn handle(&mut self, w: &Widget, event: &InputEvent, out_events: &mut Vec<(usize, Event)>) {
    }

    pub fn draw(
        &mut self,
        w: &Widget,
        style: &DPIStyle,
        pos: Rect,
        real_pos: Rect,
        p: &mut Painter,
    ) {
        let dpi_f = p.dpi_factor;
        let mut dbg = w.debug_tag();
        dbg.set_offs((real_pos.x - pos.x, real_pos.y - pos.y));
        self.real_pos = real_pos;

        p.rect_fill(style.bg_color(), pos.x, pos.y, pos.w, pos.h);
        let p2 = pos.shrink(10.0, 10.0);
        p.rect_fill(style.color(), p2.x, p2.y, p2.w, p2.h);
    }
}
