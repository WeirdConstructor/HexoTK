// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::Widget;

use crate::style::*;

use crate::painter::*;
use crate::rect::*;

use std::cell::RefCell;
use std::rc::Rc;

pub const SCOPE_SAMPLES: usize = 512;

pub trait ScopeModel {
    fn signal_count(&self) -> usize;
    fn signal_len(&self) -> usize;
    fn get(&self, sig: usize, idx: usize) -> f32;
    fn is_active(&self, sig: usize) -> bool;
}

#[derive(Debug, Clone)]
pub struct StaticScopeData {
    samples: Vec<f32>,
}

impl StaticScopeData {
    pub fn new() -> Self {
        Self { samples: vec![0.0; SCOPE_SAMPLES] }
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }

    pub fn set_sample(&mut self, i: usize, y: f32) {
        if self.samples.len() <= i {
            self.samples.resize(i + 1, 0.0);
        }
        self.samples[i] = y;
    }
}

impl ScopeModel for StaticScopeData {
    fn signal_count(&self) -> usize {
        1
    }
    fn signal_len(&self) -> usize {
        self.samples.len()
    }
    fn get(&self, _sig: usize, idx: usize) -> f32 {
        self.samples[idx]
    }
    fn is_active(&self, sig: usize) -> bool {
        sig == 0
    }
}

pub struct Scope {
    draw_buf: Vec<[(f32, f32); SCOPE_SAMPLES]>,
    data: Rc<RefCell<dyn ScopeModel>>,
    live_area: Rect,
}

impl Scope {
    pub fn new(data: Rc<RefCell<dyn ScopeModel>>) -> Self {
        Self { data, draw_buf: vec![], live_area: Rect::from(0.0, 0.0, 0.0, 0.0) }
    }

    pub fn get_generation(&self) -> u64 {
        0
    }

    fn draw_samples(&mut self, pos: Rect) {
        let data = self.data.borrow();

        for (sig_idx, buf) in self.draw_buf.iter_mut().enumerate() {
            if !data.is_active(sig_idx) {
                continue;
            }

            for i in 0..SCOPE_SAMPLES {
                let gx = (i as f32 * self.live_area.w) / (SCOPE_SAMPLES as f32);
                let sample = data.get(sig_idx, i);
                let gy = (1.0 - ((sample * 0.5) + 0.5).clamp(0.0, 1.0)) * pos.h;

                buf[i] = ((pos.x + (gx as f32)), (pos.y + (gy as f32)));
            }
        }
    }

    fn draw_graph(&mut self, style: &DPIStyle, p: &mut Painter) {
        let data = self.data.borrow();

        let line_color = style.color();
        let line_w = style.graph_line();
        let line1 = style.vline1();
        let line2 = style.vline2();
        let line1_color = style.vline1_color();
        let line2_color = style.vline2_color();

        for (i, buf) in self.draw_buf.iter().enumerate() {
            if !data.is_active(i) {
                continue;
            }

            let color = match i {
                1 => line1_color,
                2 => line2_color,
                _ => line_color,
            };
            let line_w = match i {
                1 => line1,
                2 => line2,
                _ => line_w,
            };
            p.path_stroke(line_w, color, &mut buf.iter().copied(), false);
        }
    }

    pub fn draw(
        &mut self,
        _w: &Widget,
        style: &DPIStyle,
        pos: Rect,
        real_pos: Rect,
        p: &mut Painter,
    ) {
        self.live_area = real_pos;

        let hline = style.hline();
        let hline_color = style.hline_color();

        if hline > 0.1 {
            p.path_stroke(
                hline,
                hline_color,
                &mut [
                    (pos.x, (pos.y + pos.h * 0.5).round()),
                    (pos.x + pos.w, (pos.y + pos.h * 0.5).round()),
                ]
                .iter()
                .copied(),
                false,
            );
        }
    }

    pub fn draw_frame(&mut self, _w: &Widget, style: &DPIStyle, p: &mut Painter) {
        let sig_cnt = self.data.borrow().signal_count();
        if sig_cnt > self.draw_buf.len() {
            self.draw_buf.resize_with(sig_cnt, || [(0.0, 0.0); SCOPE_SAMPLES]);
        }
        self.draw_samples(self.live_area);
        self.draw_graph(style, p);
    }
}
