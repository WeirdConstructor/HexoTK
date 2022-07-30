// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::{EvPayload, Event, InputEvent, MButton, Widget};

use crate::style::*;

use crate::painter::*;
use crate::rect::*;

use std::cell::RefCell;
use std::rc::Rc;

use hexodsp::blocklang::{BlockView, BlockCodeView};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockPos {
    Block { id: usize, x: i64, y: i64, row: usize, col: usize, rows: usize },
    Cell { id: usize, x: i64, y: i64 },
}

#[allow(dead_code)]
impl BlockPos {
    pub fn area_id(&self) -> usize {
        match self {
            BlockPos::Block { id, .. } => *id,
            BlockPos::Cell { id, .. } => *id,
        }
    }

    pub fn x(&self) -> i64 {
        match self {
            BlockPos::Block { x, .. } => *x,
            BlockPos::Cell { x, .. } => *x,
        }
    }

    pub fn y(&self) -> i64 {
        match self {
            BlockPos::Block { y, .. } => *y,
            BlockPos::Cell { y, .. } => *y,
        }
    }

    pub fn row_info(&self) -> (usize, usize) {
        match self {
            BlockPos::Block { rows, row, .. } => (*rows, *row),
            BlockPos::Cell { .. } => (1, 0),
        }
    }

    pub fn pos(&self) -> (usize, i64, i64) {
        match self {
            BlockPos::Block { id, x, y, .. } => (*id, *x, *y),
            BlockPos::Cell { id, x, y, .. } => (*id, *x, *y),
        }
    }
}

pub struct BlockCode {
    code: Rc<RefCell<dyn BlockCodeView>>,

    block_size: f32,

    areas: Vec<Vec<(usize, Rect)>>,
    hover: Option<(usize, i64, i64, usize)>,

    m_down: Option<BlockPos>,
    pan_on: Option<(f32, f32)>,

    shift_offs: (f32, f32),
    tmp_shift_offs: Option<(f32, f32)>,

    mouse_pos: (f32, f32),
    real_pos: Rect,
}

impl BlockCode {
    pub fn new(code: Rc<RefCell<dyn BlockCodeView>>) -> Self {
        Self {
            code,

            block_size: 30.0,

            areas: vec![],
            hover: None,
            m_down: None,
            pan_on: None,

            shift_offs: (0.0, 0.0),
            tmp_shift_offs: None,

            mouse_pos: (0.0, 0.0),
            real_pos: Rect::from(0.0, 0.0, 0.0, 0.0),
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

                let sub_col = if (xo - xi as f32 * block_w) > (block_w * 0.5) { 1 } else { 0 };

                return Some((a.0, xi, yi, sub_col));
            }
        }

        None
    }

    fn find_pos_at_mouse(&self, x: f32, y: f32) -> Option<BlockPos> {
        if let Some((area, x, y, subcol)) = self.find_area_at_mouse(x, y) {
            if let Some((ox, oy)) = self.code.borrow().origin_at(area, x, y) {
                let rows = self.code.borrow().block_at(area, ox, oy).map(|b| b.rows()).unwrap_or(1);

                let row = (y - oy).max(0) as usize;
                Some(BlockPos::Block { id: area, x, y, col: subcol, row, rows })
            } else {
                Some(BlockPos::Cell { id: area, x, y })
            }
        } else {
            None
        }
    }

    pub fn draw_area(
        &mut self,
        style: &DPIStyle,
        p: &mut Painter,
        area_id: usize,
        pos: Rect,
        level: usize,
        mut dbg: LblDebugTag,
    ) {
        let dpi_f = p.dpi_factor;

        p.clip_region(pos.x, pos.y, pos.w, pos.h);

        let block_h = dpi_f * self.block_size;
        let block_w = block_h * 2.0;

        if let Some(s) = self.code.borrow().area_header(area_id) {
            p.label(
                self.block_size * 0.4,
                -1,
                style.border_color(),
                pos.x,
                pos.y,
                pos.w,
                block_h,
                s,
                dbg.source("area"),
            );
        }

        let cols = (pos.w / block_w).ceil() as usize;
        let rows = (pos.h / block_h).ceil() as usize;

        p.rect_fill(style.bg_color(), pos.x, pos.y, pos.w, pos.h);

        let shift_x = (self.shift_offs.0 + self.tmp_shift_offs.map(|o| o.0).unwrap_or(0.0)).round();
        let shift_y = (self.shift_offs.1 + self.tmp_shift_offs.map(|o| o.1).unwrap_or(0.0)).round();

        let draw_col_offs = if level == 0 { -((shift_x / block_w).round() as i64) } else { 0 };
        let draw_row_offs = if level == 0 { -((shift_y / block_h).round() as i64) } else { 0 };

        if level == 0 {
            p.translate(shift_x, shift_y);
        }

        for row in draw_row_offs..(rows as i64 + draw_row_offs) {
            for col in draw_col_offs..(cols as i64 + draw_col_offs) {
                let x = col as f32 * block_w;
                let y = row as f32 * block_h;

                let marker_px = (block_h * 0.2).floor();

                if style.with_markers() {
                    draw_markers(
                        p,
                        style.grid_marker_color(),
                        pos.x + x,
                        pos.y + y,
                        block_w,
                        block_h,
                        marker_px,
                    );
                }
            }
        }

        let mut next_areas = vec![];

        let mut lbl_buf: [u8; 20] = [0; 20];

        let area_border_px = dpi_f * 4.0;

        for row in (-10 + draw_row_offs)..((rows as i64) + draw_row_offs) {
            for col in (-5 + draw_col_offs)..((cols as i64) + draw_col_offs + 1) {
                let col = col as i64;
                let row = row as i64;

                let x = col as f32 * block_w;
                let y = row as f32 * block_h;

                let mut hover_here = if let Some(hover) = self.hover {
                    area_id == hover.0 && col == hover.1 && row == hover.2
                } else {
                    false
                };

                let mut hover_row = -1;
                let mut hover_col = -1;

                if let Some((area, x, y, subcol)) = self.hover {
                    if area == area_id {
                        if let Some((bx, by)) = self.code.borrow().origin_at(area, x, y) {
                            hover_row = (y - by) as i32;
                            hover_col = subcol as i32;
                            hover_here = bx == col && by == row;
                        }
                    }
                }

                if let Some(block) = self.code.borrow().block_at(area_id, col, row) {
                    let bg_color = if hover_here {
                        style.block_bg_hover_color()
                    } else {
                        style.block_bg_color()
                    };
                    let border_color = if hover_here {
                        style.hover_border_color()
                    } else {
                        block
                            .custom_color()
                            .map(|cidx| style.color_by_idx(cidx))
                            .unwrap_or(style.border_color())
                    };

                    let w = block_w;
                    let h = block.rows() as f32 * block_h;

                    p.rect_fill(bg_color, pos.x + x, pos.y + y, w, h);

                    p.rect_stroke(
                        dpi_f * 2.0,
                        border_color,
                        pos.x + x + dpi_f * 1.0,
                        pos.y + y + dpi_f * 2.0,
                        w - dpi_f * 2.0,
                        h - dpi_f * 4.0,
                    );

                    let hole_px = (0.6 * block_h).ceil();

                    let len = block.label(&mut lbl_buf[..]);
                    let val_s = std::str::from_utf8(&lbl_buf[0..len]).unwrap();
                    p.label(
                        dpi_f * self.block_size * 0.5,
                        0,
                        style.border_color(),
                        pos.x + x,
                        pos.y + y,
                        w,
                        h,
                        val_s,
                        dbg.source("block"),
                    );

                    for i in 0..block.rows() {
                        if block.has_input(i) {
                            let row = i as f32 * block_h;
                            p.rect_fill(
                                bg_color,
                                pos.x + x,
                                pos.y + y + row + ((block_h - hole_px) * 0.5).floor(),
                                dpi_f * 3.0,
                                hole_px,
                            );

                            let len = block.input_label(i, &mut lbl_buf[..]);
                            let val_s = std::str::from_utf8(&lbl_buf[0..len]).unwrap();
                            p.label(
                                dpi_f * self.block_size * 0.4,
                                -1,
                                style.border_color(),
                                pos.x + x + dpi_f * 1.0,
                                pos.y + row + y - dpi_f * 1.0,
                                (block_w * 0.5).floor(),
                                block_h,
                                val_s,
                                dbg.source("input"),
                            );

                            if hover_here && hover_col == 0 && hover_row == (i as i32) {
                                let sel_block_w = (block_w * 0.5 * 0.8).floor();
                                let sel_block_h = (block_h * 0.8).floor();

                                p.rect_stroke(
                                    4.0,
                                    style.port_select_color(),
                                    (pos.x + x + ((block_w * 0.5 - sel_block_w) * 0.5)).floor(),
                                    (pos.y + row + y + ((block_h - sel_block_h) * 0.5)).floor(),
                                    sel_block_w,
                                    sel_block_h,
                                );
                            }
                        }

                        if block.has_output(i) {
                            let row = i as f32 * block_h;
                            p.rect_fill(
                                bg_color,
                                pos.x + x + w - dpi_f * 3.0,
                                pos.y + y + row + ((block_h - hole_px) * 0.5).floor(),
                                dpi_f * 3.0,
                                hole_px,
                            );

                            let len = block.output_label(i, &mut lbl_buf[..]);
                            let val_s = std::str::from_utf8(&lbl_buf[0..len]).unwrap();
                            p.label(
                                dpi_f * self.block_size * 0.4,
                                1,
                                style.border_color(),
                                (pos.x + x + (block_w * 0.5) - dpi_f * 1.0).floor(),
                                pos.y + row + y - dpi_f * 1.0,
                                (block_w * 0.5).floor(),
                                block_h,
                                val_s,
                                dbg.source("output"),
                            );

                            if hover_here && hover_col == 1 && hover_row == (i as i32) {
                                let sel_block_w = (block_w * 0.5 * 0.8).floor();
                                let sel_block_h = (block_h * 0.8).floor();

                                p.rect_stroke(
                                    dpi_f * 4.0,
                                    style.port_select_color(),
                                    (pos.x
                                        + x
                                        + (block_w * 0.5)
                                        + ((block_w * 0.5 - sel_block_w) * 0.5))
                                        .floor(),
                                    (pos.y + row + y + ((block_h - sel_block_h) * 0.5)).floor(),
                                    sel_block_w,
                                    sel_block_h,
                                );
                            }
                        }
                    }

                    if let Some(cont_id) = block.contains(0) {
                        let (area_w, area_h) = self.code.borrow().area_size(cont_id);
                        let bpos = Rect {
                            x: pos.x + x + area_border_px,  // + border
                            y: pos.y + y + h - dpi_f * 1.0, // -1.0 for the border offs
                            w: (area_w as f32 * block_w + block_w * 0.3).floor(),
                            h: (area_h as f32 * block_h + block_w * 0.3).floor(),
                        };

                        next_areas.push((cont_id, bpos, border_color, bg_color));

                        if let Some(cont_id) = block.contains(1) {
                            let (area_w, area_h) = self.code.borrow().area_size(cont_id);
                            let bpos = Rect {
                                x: bpos.x,
                                y: bpos.y + 2.0 * area_border_px + bpos.h - dpi_f * 1.0, // -1.0 for the border offs
                                w: (area_w as f32 * block_w + block_w * 0.3).floor(),
                                h: (area_h as f32 * block_h + block_w * 0.3).floor(),
                            };

                            next_areas.push((cont_id, bpos, border_color, bg_color));
                        }
                    }
                } else if hover_here {
                    let mut y_offs = 0.0;
                    let mut h_add = 0.0;

                    if let Some(down) = self.m_down {
                        let (rows, grab_row) = down.row_info();

                        y_offs = -(grab_row as f32 * block_h);
                        h_add = (rows - 1) as f32 * block_h;
                    }

                    p.rect_stroke(
                        2.0,
                        style.hover_border_color(),
                        pos.x + x + dpi_f * 1.0,
                        pos.y + y + dpi_f * 1.0 + y_offs,
                        block_w - dpi_f * 2.0,
                        (h_add + block_h) - dpi_f * 2.0,
                    );
                }

                if let Some(down) = self.m_down {
                    if (area_id, col, row) == down.pos() {
                        let (rows, grab_row) = down.row_info();

                        p.rect_stroke(
                            dpi_f * 2.0,
                            style.port_select_color(),
                            pos.x + x + dpi_f * 1.0,
                            pos.y + y + dpi_f * 1.0 - grab_row as f32 * block_h,
                            block_w - dpi_f * 2.0,
                            (block_h * (rows as f32)) - dpi_f * 2.0,
                        );
                    }
                }
            }
        }

        for cont_area in next_areas.into_iter() {
            let (cont_id, pos, border_color, bg_color) = cont_area;

            let apos =
                Rect { x: pos.x + area_border_px, y: pos.y + area_border_px, w: pos.w, h: pos.h };
            p.rect_fill(
                border_color,
                apos.x - area_border_px,
                apos.y - area_border_px,
                apos.w + dpi_f * 8.0,
                apos.h + dpi_f * 8.0,
            );
            let h_area_border_px = (area_border_px * 0.5).floor();
            p.rect_fill(
                style.bg_color(),
                apos.x - h_area_border_px,
                apos.y - h_area_border_px,
                apos.w + area_border_px,
                apos.h + area_border_px,
            );
            p.rect_fill(
                bg_color,
                (pos.x + block_w * 0.25).floor(),
                pos.y - h_area_border_px,
                (block_w * 0.5).floor(),
                dpi_f * 8.0,
            );

            self.store_area_pos(
                cont_id,
                level + 1,
                Rect {
                    x: apos.x - area_border_px,
                    y: apos.y - area_border_px,
                    w: apos.w + dpi_f * 8.0,
                    h: apos.h + dpi_f * 8.0,
                },
            );

            self.draw_area(style, p, cont_id, apos, level + 1, dbg);
        }

        if level == 0 {
            p.restore();
        }

        p.reset_clip_region();
    }
}

impl BlockCode {
    pub fn handle(&mut self, w: &Widget, event: &InputEvent, out_events: &mut Vec<(usize, Event)>) {
        match event {
            InputEvent::MouseButtonPressed(btn) => {
                if !w.is_hovered() {
                    return;
                }

                if *btn != MButton::Middle {
                    let (x, y) = self.mouse_pos;
                    self.m_down = self.find_pos_at_mouse(x, y);

                    w.activate();
                } else {
                    self.pan_on = Some(self.mouse_pos);
                }
            }
            InputEvent::MouseButtonReleased(btn) => {
                if self.pan_on.is_none() && !w.is_active() {
                    return;
                }

                let (x, y) = self.mouse_pos;

                if *btn == MButton::Middle {
                    if let Some(tmp_shift_offs) = self.tmp_shift_offs.take() {
                        self.shift_offs.0 += tmp_shift_offs.0;
                        self.shift_offs.1 += tmp_shift_offs.1;
                    }

                } else {
                    let m_up = self.find_pos_at_mouse(x, y);

                    if m_up == self.m_down {
                        if let Some(pos) = m_up {
                            out_events.push(w.event(
                                "click",
                                EvPayload::BlockPos { button: *btn, at: pos, to: None },
                            ));
                        }
                    } else {
                        if let (Some(down_pos), Some(up_pos)) = (self.m_down, m_up) {
                            out_events.push(w.event(
                                "drag",
                                EvPayload::BlockPos {
                                    button: *btn,
                                    at: down_pos,
                                    to: Some(up_pos),
                                },
                            ));
                        }
                    }

                    w.emit_redraw_required();
                }

                self.m_down = None;
                self.pan_on = None;

                w.deactivate();
                w.emit_redraw_required();
            }
            InputEvent::MousePosition(x, y) => {
                self.mouse_pos = (*x, *y);

                if let Some((pan_x, pan_y)) = self.pan_on {
                    self.tmp_shift_offs = Some((*x - pan_x, *y - pan_y));

                    w.emit_redraw_required();
                } else {
                    if !w.is_hovered() {
                        return;
                    }

                    let old_hover = self.hover;
                    self.hover = self.find_area_at_mouse(*x, *y);

                    if old_hover != self.hover {
                        w.emit_redraw_required();
                    }
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
        // FIXME: The usage of dpi_f * 1.0 is suspicious, but I currently don't know
        //        why the offsets are there where they are and don't have the time to investigate.
        self.real_pos = real_pos;

        let mut dbg = w.debug_tag();
        dbg.set_offs((self.real_pos.x - pos.x, self.real_pos.y - pos.y));

        self.reset_areas();

        let pos = pos.floor();
        self.store_area_pos(0, 0, pos);
        self.draw_area(style, p, 0, pos, 0, dbg);
    }

    pub fn get_generation(&self) -> u64 {
        self.code.borrow().generation()
    }
}

fn draw_markers(
    p: &mut Painter,
    clr: (f32, f32, f32),
    x: f32,
    y: f32,
    block_w: f32,
    block_h: f32,
    marker_px: f32,
) {
    p.path_stroke(
        1.0,
        clr,
        &mut ([(x, y + marker_px), (x, y), (x + marker_px, y)]
            .iter()
            .copied()
            .map(|p| (p.0.floor() + 0.5, p.1.floor() + 0.5))),
        false,
    );

    p.path_stroke(
        1.0,
        clr,
        &mut ([(block_w + x - marker_px, y), (block_w + x, y), (block_w + x, y + marker_px)]
            .iter()
            .copied()
            .map(|p| (p.0.floor() - 0.5, p.1.floor() + 0.5))),
        false,
    );

    p.path_stroke(
        1.0,
        clr,
        &mut ([(x, block_h + y - marker_px), (x, block_h + y), (x + marker_px, block_h + y)]
            .iter()
            .copied()
            .map(|p| (p.0.floor() + 0.5, p.1.floor() - 0.5))),
        false,
    );

    p.path_stroke(
        1.0,
        clr,
        &mut ([
            (block_w + x - marker_px, block_h + y),
            (block_w + x, block_h + y),
            (block_w + x, block_h + y - marker_px),
        ]
        .iter()
        .copied()
        .map(|p| (p.0.floor() - 0.5, p.1.floor() - 0.5))),
        false,
    );
}
