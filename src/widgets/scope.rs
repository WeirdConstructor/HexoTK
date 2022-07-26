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
    fn fmt_val(&self, sig: usize, buf: &mut [u8]) -> usize;
}

#[derive(Debug, Clone)]
pub struct StaticScopeData {
    samples: Vec<Vec<f32>>,
}

impl StaticScopeData {
    pub fn new() -> Self {
        let mut v = vec![];
        v.push(vec![0.0; SCOPE_SAMPLES]);
        v.push(vec![0.0; SCOPE_SAMPLES]);
        v.push(vec![0.0; SCOPE_SAMPLES]);
        Self { samples: v }
    }

    pub fn clear(&mut self) {
        self.samples[0].clear();
        self.samples[1].clear();
        self.samples[2].clear();
    }

    pub fn set_sample(&mut self, sig: usize, i: usize, y: f32) {
        if self.samples[sig].len() <= i {
            self.samples[sig].resize(i + 1, 0.0);
        }
        self.samples[sig][i] = y;
    }
}

impl ScopeModel for StaticScopeData {
    fn signal_count(&self) -> usize {
        3
    }
    fn signal_len(&self) -> usize {
        self.samples.len()
    }
    fn get(&self, sig: usize, idx: usize) -> f32 {
        self.samples[sig][idx]
    }
    fn is_active(&self, _sig: usize) -> bool {
        true
    }
    fn fmt_val(&self, sig: usize, buf: &mut [u8]) -> usize {
        use std::io::Write;
        let max_len = buf.len();
        let mut bw = std::io::BufWriter::new(buf);
        match write!(
            bw,
            "{} min: {:6.3} max: {:6.3} rng: {:6.3}",
            //                   self.min, self.max, self.avg)
            sig,
            -0.1212,
            0.992343,
            0.3432
        ) {
            Ok(_) => {
                if bw.buffer().len() > max_len {
                    max_len
                } else {
                    bw.buffer().len()
                }
            }
            Err(_) => 0,
        }
    }
}

pub struct Scope {
    draw_buf: Vec<[(f32, f32); SCOPE_SAMPLES]>,
    data: Rc<RefCell<dyn ScopeModel>>,
    live_area: Rect,
    lbl_buf: [u8; 50],
    txt_h: f32,
}

impl Scope {
    pub fn new(data: Rc<RefCell<dyn ScopeModel>>) -> Self {
        Self {
            data,
            draw_buf: vec![],
            live_area: Rect::from(0.0, 0.0, 0.0, 0.0),
            lbl_buf: [0; 50],
            txt_h: 0.0,
        }
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

        for (i, buf) in self.draw_buf.iter().enumerate().rev() {
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

        self.txt_h = p.font_height(style.font_size() as f32, true);

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

    pub fn draw_frame(&mut self, w: &Widget, style: &DPIStyle, p: &mut Painter) {
        let mut dbg = w.debug_tag();

        let sig_cnt = self.data.borrow().signal_count();
        if sig_cnt > self.draw_buf.len() {
            self.draw_buf.resize_with(sig_cnt, || [(0.0, 0.0); SCOPE_SAMPLES]);
        }
        self.draw_samples(self.live_area.shrink(0.0, self.txt_h * 2.0));
        self.draw_graph(style, p);

        let line_color = style.color();
        let line1_color = style.vline1_color();
        let line2_color = style.vline2_color();

        let data = self.data.borrow();
        for i in 0..sig_cnt {
            if !data.is_active(i) {
                continue;
            }

            let color = match i {
                1 => line1_color,
                2 => line2_color,
                _ => line_color,
            };

            let len = data.fmt_val(i, &mut self.lbl_buf[..]);
            let val_s = std::str::from_utf8(&self.lbl_buf[0..len]).unwrap();
            dbg.set_logic_pos(i as i32, 0);

            let y = if i == 2 {
                self.live_area.y + self.live_area.h - self.txt_h
            } else {
                self.live_area.y + self.txt_h * (i as f32)
            };
            p.label(
                style.font_size(),
                0,
                color,
                self.live_area.x,
                y,
                self.live_area.w,
                self.txt_h,
                val_s,
                dbg.source("scope_label"),
            );
        }
    }
}
