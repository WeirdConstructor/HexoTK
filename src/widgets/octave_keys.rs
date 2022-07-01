// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::{Widget, InputEvent, Event, MButton, EvPayload, Style, Mutable};

use crate::style::*;

use crate::painter::*;
use crate::rect::*;

use std::rc::Rc;
use std::cell::RefCell;

pub const UI_GRPH_BORDER_CLR      : (f32, f32, f32) = UI_ACCENT_CLR;
pub const UI_GRPH_LINE_CLR        : (f32, f32, f32) = UI_PRIM_CLR;
pub const UI_GRPH_PHASE_CLR       : (f32, f32, f32) = UI_HLIGHT_CLR;
pub const UI_GRPH_PHASE_BG_CLR    : (f32, f32, f32) = UI_HLIGHT2_CLR;
pub const UI_GRPH_BG              : (f32, f32, f32) = UI_LBL_BG_CLR;

trait OOctaveKeysData {
    fn key_mask(&self) -> i64;
    fn phase_value(&self) -> i64;
    fn get_generation(&self) -> u64;
}

#[derive(Debug, Clone)]
pub struct OctaveKeysData {
    key_mask:   i64,
    generation: u64,
}

impl OctaveKeysData {
    pub fn new() -> Self {
        Self {
            key_mask:   0x0,
            generation: 0,
        }
    }

    pub fn clear(&mut self) {
        self.key_mask = 0;
        self.generation += 1;
    }

    pub fn set_key_mask(&mut self, mask: i64) {
        self.key_mask = mask;
        self.generation += 1;
    }
}


pub struct OctaveKeys {
    data:           Rc<RefCell<OctaveKeysData>>,
    key_areas:      Vec<(usize, Rect)>,
    hover_index:    Option<usize>,
    mouse_pos:      (f32, f32),
}

impl OctaveKeys {
    pub fn new(data: Rc<RefCell<OctaveKeysData>>) -> Self {
        Self {
            data,
            key_areas:      vec![],
            hover_index:    None,
            mouse_pos:      (0.0, 0.0),
        }
    }

    pub fn get_generation(&self) -> u64 {
        self.data.borrow().generation
    }

    fn get_key_index_at(&self, x: f32, y: f32) -> Option<usize> {
        let mut ret = None;

        for (idx, area) in &self.key_areas {
            if area.is_inside(x, y) {
                ret = Some(*idx);
            }
        }

        ret
    }

    pub fn toggle_index(&mut self, index: usize) {
        if index >= 64 { return; }
        self.data.borrow_mut().key_mask ^= 0x1 << index;
    }
}

fn draw_key(p: &mut Painter, key_mask: i64,
            key: &Rect, hover_idx: Option<usize>,
            index: usize,
            phase_index: usize)
{
    let key_is_set = key_mask & (0x1 << index) > 0;

    let mut hover_this_key = false;
    if let Some(hover_idx) = hover_idx {
        hover_this_key = hover_idx == index;
    }

    let (mut bg_color, mut line_color) =
        if key_is_set {
            if hover_this_key {
                (UI_GRPH_LINE_CLR, UI_GRPH_BG)
            } else {
                (UI_GRPH_PHASE_BG_CLR, UI_GRPH_BG)
            }
        } else if hover_this_key {
            (UI_GRPH_PHASE_BG_CLR, UI_GRPH_BG)
        } else {
            (UI_GRPH_BG, UI_GRPH_LINE_CLR)
        };

    if phase_index == index {
        if key_is_set {
            bg_color = UI_GRPH_BORDER_CLR;
        } else {
            bg_color = UI_GRPH_PHASE_CLR;
        }

        line_color = UI_GRPH_BG;
    }

    p.rect_fill(line_color, key.x, key.y, key.w, key.h);
    let k2 = key.shrink(1.0, 1.0);
    p.rect_fill(bg_color, k2.x, k2.y, k2.w, k2.h);
}

impl OctaveKeys {
    pub fn handle(
        &mut self, w: &Widget, event: &InputEvent,
        out_events: &mut Vec<(usize, Event)>)
    {
        match event {
            InputEvent::MouseButtonPressed(MButton::Left) => {
                if !w.is_hovered() {
                    return;
                }

                w.activate();
                w.emit_redraw_required();
            },
            InputEvent::MouseButtonReleased(MButton::Left) => {
                if !w.is_active() {
                    return;
                }

                let (x, y) = self.mouse_pos;

                if let Some(key_idx) = self.get_key_index_at(x, y) {
                    self.toggle_index(key_idx);

                    out_events.push((w.id(), Event {
                        name: "change".to_string(),
                        data: EvPayload::KeyMask(self.data.borrow().key_mask)
                    }));
                }

                w.deactivate();
                w.emit_redraw_required();
            }
            InputEvent::MousePosition(x, y) => {
                self.mouse_pos = (*x, *y);

                let old_hover = self.hover_index;
                self.hover_index = self.get_key_index_at(*x, *y);

                if old_hover != self.hover_index {
                    w.emit_redraw_required();
                }
            }
            _ => {}
        }
    }

    pub fn draw(
        &mut self, w: &Widget, style: &Style, pos: Rect,
        real_pos: Rect, p: &mut Painter)
    {
        let mut dbg = LblDebugTag::from_id(w.id());
        let rp_offset = (real_pos.x - pos.x, real_pos.y - pos.y);
        dbg.set_offs(rp_offset);

        // let border_color =
        //     if state.hovered == entity { UI_GRPH_BORDER_HOVER_CLR }
        //     else { UI_GRPH_BORDER_CLR };

        let xd = (pos.w / 7.0).floor();
        let xd_pad_for_center = ((pos.w - xd * 7.0) * 0.5).floor();
        let pos = pos.shrink(xd_pad_for_center, 0.0);

        let xoffs_w = [
            (0, 0.0 * xd),   // white C
            (2, 1.0 * xd),   // white D
            (4, 2.0 * xd),   // white E
            (5, 3.0 * xd),   // white F
            (7, 4.0 * xd),   // white G
            (9, 5.0 * xd),   // white A
            (11, 6.0 * xd),  // white B
        ];

        let xoffs_b = [
            (1, 1.0 * xd),   // black C#
            (3, 2.0 * xd),   // black D#
            (6, 4.0 * xd),   // black F#
            (8, 5.0 * xd),   // black G#
            (10, 6.0 * xd),  // black A#
        ];

        self.key_areas.clear();
        for xw in xoffs_w.iter() {
            let key =
                Rect {
                    x: pos.x + (*xw).1,
                    y: pos.y,
                    w: xd,
                    h: pos.h,
                };

//            draw_key(p, key_mask, key, hover_idx, (*xw).0, phase_index);
            self.key_areas.push(
                ((*xw).0, key.offs(rp_offset.0, rp_offset.1)));
        }

        let black_width = xd * 0.75;

        for xb in xoffs_b.iter() {
            let key =
                Rect {
                    x: pos.x + (*xb).1 - black_width * 0.5,
                    y: pos.y,
                    w: black_width,
                    h: pos.h * 0.5,
                };

//            draw_key(p, key_mask, key, hover_idx, (*xb).0, phase_index);
            self.key_areas.push(
                ((*xb).0, key.offs(rp_offset.0, rp_offset.1)));
        }
    }

    pub fn draw_frame(&mut self, w: &Widget, style: &Style, painter: &mut Painter) {
        let phase = 0.0_f64;
//            if let Some(phase) = ui.atoms().get_phase_value(id) {
//                phase as f64
//            } else { 0.0 };
        let phase_index = (phase * 12.0).floor() as usize;

        let mut hover_idx = self.hover_index;
        if !w.is_hovered() {
            hover_idx = None;
        }

        let key_mask = self.data.borrow().key_mask;

        for (index, key) in self.key_areas.iter() {
            draw_key(painter, key_mask, key, hover_idx, *index, phase_index);
        }
    }
}
