// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::{EvPayload, Event, InputEvent, MButton, Widget};

use crate::style::*;

use crate::painter::*;
use crate::rect::*;

use std::cell::RefCell;
use std::rc::Rc;

pub trait BlockView {
    fn rows(&self) -> usize;
    fn contains(&self, idx: usize) -> Option<usize>;
    fn expanded(&self) -> bool;
    fn label(&self, buf: &mut [u8]) -> usize;
    fn has_input(&self, idx: usize) -> bool;
    fn has_output(&self, idx: usize) -> bool;
    fn input_label(&self, idx: usize, buf: &mut [u8]) -> usize;
    fn output_label(&self, idx: usize, buf: &mut [u8]) -> usize;
    fn custom_color(&self) -> Option<usize>;
}

pub trait BlockCodeView {
    fn area_header(&self, id: usize) -> Option<&str>;
    fn area_size(&self, id: usize) -> (usize, usize);
    fn block_at(&self, id: usize, x: i64, y: i64) -> Option<&dyn BlockView>;
    fn origin_at(&self, id: usize, x: i64, y: i64) -> Option<(i64, i64)>;
    fn generation(&self) -> u64;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockPos {
    Block { id: usize, x: i64, y: i64, row: usize, col: usize, rows: usize },
    Cell  { id: usize, x: i64, y: i64 },
}

#[allow(dead_code)]
impl BlockPos {
    pub fn area_id(&self) -> usize {
        match self {
            BlockPos::Block { id, .. } => *id,
            BlockPos::Cell  { id, .. } => *id,
        }
    }

    pub fn x(&self) -> i64 {
        match self {
            BlockPos::Block { x, .. } => *x,
            BlockPos::Cell  { x, .. } => *x,
        }
    }

    pub fn y(&self) -> i64 {
        match self {
            BlockPos::Block { y, .. } => *y,
            BlockPos::Cell  { y, .. } => *y,
        }
    }

    pub fn row_info(&self) -> (usize, usize) {
        match self {
            BlockPos::Block { rows, row, .. } => (*rows, *row),
            BlockPos::Cell  { .. }            => (1, 0),
        }
    }

    pub fn pos(&self) -> (usize, i64, i64) {
        match self {
            BlockPos::Block { id, x, y, .. } => (*id, *x, *y),
            BlockPos::Cell  { id, x, y, .. } => (*id, *x, *y),
        }
    }
}



pub struct BlockCode {
    code:           Rc<RefCell<dyn BlockCodeView>>,

    block_size:     f32,

    areas:          Vec<Vec<(usize, Rect)>>,
    hover:          Option<(usize, i64, i64, usize)>,

    m_down:         Option<BlockPos>,

    shift_offs:     (f32, f32),
    tmp_shift_offs: Option<(f32, f32)>,

    mouse_pos:      (f32, f32),
}

impl BlockCode {
    pub fn new(code: Rc<RefCell<dyn BlockCodeView>>) -> Self {
        Self {
            code,

            block_size:     30.0,

            areas:          vec![],
            hover:          None,
            m_down:         None,

            shift_offs:     (0.0, 0.0),
            tmp_shift_offs: None,

            mouse_pos: (0.0, 0.0),
        }
    }

    pub fn reset_areas(&mut self) {
        for a in self.areas.iter_mut() {
            a.clear();
        }
    }

    pub fn store_area_pos(&mut self, area_id: usize, level: usize, pos: Rect) {
        if level >= self.areas.len() {
            self.areas.resize_with(level + 1, || vec![]);
        }

        self.areas[level].push((area_id, pos));
    }

    fn find_area_at_mouse(&self, x: f32, y: f32) -> Option<(usize, i64, i64, usize)> {
        let shift_x = (self.shift_offs.0 + self.tmp_shift_offs.map(|o| o.0).unwrap_or(0.0)).round();
        let shift_y = (self.shift_offs.1 + self.tmp_shift_offs.map(|o| o.1).unwrap_or(0.0)).round();

        let x = x - shift_x;
        let y = y - shift_y;

        let block_h = self.block_size;
        let block_w = block_h * 2.0;

        for lvl in self.areas.iter().rev() {
            for a in lvl.iter() {
                let (id, pos) = *a;

                if id > 0 && !pos.is_inside(x, y) {
                    continue;
                }

                let xo = x - pos.x;
                let yo = y - pos.y;
                let xi = (xo / block_w).floor() as i64;
                let yi = (yo / block_h).floor() as i64;

                let sub_col =
                    if (xo - xi as f32 * block_w) > (block_w * 0.5) {
                        1
                    } else {
                        0
                    };

                return Some((a.0, xi, yi, sub_col));
            }
        }

        None
    }

    fn find_pos_at_mouse(&self, x: f32, y: f32) -> Option<BlockPos> {
        if let Some((area, x, y, subcol)) = self.find_area_at_mouse(x, y) {
            if let Some((ox, oy)) =
                self.code.borrow().origin_at(area, x, y)
            {
                let rows =
                    self.code.borrow()
                        .block_at(area, ox, oy)
                        .map(|b| b.rows())
                        .unwrap_or(1);

                let row = (y - oy).max(0) as usize;
                Some(BlockPos::Block { id: area, x, y, col: subcol, row, rows })
            } else {
                Some(BlockPos::Cell { id: area, x, y })
            }
        } else {
            None
        }
    }
}

impl BlockCode {
    pub fn handle(&mut self, w: &Widget, event: &InputEvent, out_events: &mut Vec<(usize, Event)>) {
        match event {
            InputEvent::MouseButtonPressed(MButton::Left) => {
                if !w.is_hovered() {
                    return;
                }

                let (x, y) = self.mouse_pos;

                w.activate();
                w.emit_redraw_required();
            }
            InputEvent::MouseButtonReleased(MButton::Left) => {
                if !w.is_active() {
                    return;
                }

                w.deactivate();
                w.emit_redraw_required();
            }
            InputEvent::MousePosition(x, y) => {
                self.mouse_pos = (*x, *y);

//                    w.emit_redraw_required();
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
        // FIXME: The usage of dpi_f * 1.0 is suspicious, but I currently don't know
        //        why the offsets are there where they are and don't have the time to investigate.
        let dpi_f = p.dpi_factor;

        let mut dbg = w.debug_tag();
        dbg.set_offs((real_pos.x - pos.x, real_pos.y - pos.y));

        let code = self.code.borrow();
    }

    pub fn get_generation(&self) -> u64 {
        self.code.borrow().generation()
    }
}

fn draw_markers(p: &mut Painter, clr: (f32, f32, f32), x: f32, y: f32, block_w: f32, block_h: f32, marker_px: f32) {
    p.path_stroke(
        1.0,
        clr,
        &mut ([
            (x,             y + marker_px),
            (x,             y),
            (x + marker_px, y),
        ].iter().copied()
         .map(|p| (p.0.floor() + 0.5, p.1.floor() + 0.5))), false);

    p.path_stroke(
        1.0,
        clr,
        &mut ([
            (block_w + x - marker_px, y),
            (block_w + x,             y),
            (block_w + x,             y + marker_px),
        ].iter().copied()
         .map(|p| (p.0.floor() - 0.5, p.1.floor() + 0.5))), false);

    p.path_stroke(
        1.0,
        clr,
        &mut ([
            (x,             block_h + y - marker_px),
            (x,             block_h + y),
            (x + marker_px, block_h + y),
        ].iter().copied()
         .map(|p| (p.0.floor() + 0.5, p.1.floor() - 0.5))), false);

    p.path_stroke(
        1.0,
        clr,
        &mut ([
            (block_w + x - marker_px, block_h + y),
            (block_w + x,             block_h + y),
            (block_w + x,             block_h + y - marker_px),
        ].iter().copied()
         .map(|p| (p.0.floor() - 0.5, p.1.floor() - 0.5))), false);
}

