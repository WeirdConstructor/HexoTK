// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::{Style, Widget};

use crate::style::*;

use crate::painter::*;
use crate::rect::*;

use std::cell::RefCell;
use std::rc::Rc;

const WAVEFORM_SCALE_FACTOR: f32 = 0.9;

pub trait GraphMinMaxModel {
    fn read(&mut self, dst: &mut [(f32, f32)]);
    fn fmt_val(&mut self, buf: &mut [u8]) -> usize;
    fn get_generation(&self) -> u64;
}

#[derive(Debug, Clone)]
pub struct StaticGraphMinMaxData {
    minmax: Vec<(f32, f32)>,
    generation: u64,
}

impl StaticGraphMinMaxData {
    pub fn new() -> Self {
        Self { minmax: vec![], generation: 0 }
    }

    pub fn clear(&mut self) {
        self.minmax.clear();
        self.generation += 1;
    }

    pub fn set_point(&mut self, i: usize, v: (f32, f32)) {
        if self.minmax.len() <= i {
            self.minmax.resize(i + 1, (0.0, 0.0));
        }
        self.minmax[i] = v;
        self.generation += 1;
    }
}

impl GraphMinMaxModel for StaticGraphMinMaxData {
    fn get_generation(&self) -> u64 {
        self.generation
    }
    fn read(&mut self, dst: &mut [(f32, f32)]) {
        for i in 0..dst.len() {
            if i < self.minmax.len() {
                dst[i] = self.minmax[i];
            } else {
                dst[i] = (0.0, 0.0);
            }
        }
    }
    fn fmt_val(&mut self, buf: &mut [u8]) -> usize {
        use std::io::Write;
        let max_len = buf.len();
        let mut bw = std::io::BufWriter::new(buf);
        match write!(
            bw,
            "{:6.3} | {:6.3} | {:6.3}",
            //                   self.min, self.max, self.avg)
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

//---------------------------------------------------------------------------

pub struct GraphMinMax {
    live_area: Rect,
    data: Rc<RefCell<dyn GraphMinMaxModel>>,
    buf: Vec<(f32, f32)>,
    minmax_buf: Vec<(f32, f32)>,
    minmax_sample_count: usize,
    lbl_buf: [u8; 50],
}

impl GraphMinMax {
    pub fn new(data: Rc<RefCell<dyn GraphMinMaxModel>>, minmax_sample_count: usize) -> Self {
        let mut buf = vec![];
        buf.resize(2 * minmax_sample_count, (0.0, 0.0));

        let mut minmax_buf = vec![];
        minmax_buf.resize(minmax_sample_count, (0.0, 0.0));

        Self {
            live_area: Rect::from(0.0, 0.0, 0.0, 0.0),
            data,
            buf,
            minmax_buf,
            minmax_sample_count,
            lbl_buf: [0; 50],
        }
    }

    pub fn get_generation(&self) -> u64 {
        self.data.borrow().get_generation()
    }

    //    fn draw_graph(&mut self, style: &Style, p: &mut Painter) {
    //        let line_color = style.color;
    //        let mut line_w      = 1.0;
    //        let mut line1       = 1.0;
    //        let mut line2       = 1.0;
    //        let mut line1_color = style.border_color;
    //        let mut line2_color = style.border_color;
    //        if let StyleExt::Graph {
    //            graph_line, vline1, vline2, vline1_color, vline2_color, ..
    //        } = style.ext {
    //            line_w      = graph_line;
    //            line1       = vline1;
    //            line2       = vline2;
    //            line1_color = vline1_color;
    //            line2_color = vline2_color;
    //        }
    //    }

    pub fn draw(
        &mut self,
        _w: &Widget,
        _style: &Style,
        _pos: Rect,
        real_pos: Rect,
        _p: &mut Painter,
    ) {
        self.live_area = real_pos;
    }

    pub fn draw_frame(&mut self, w: &Widget, style: &Style, p: &mut Painter) {
        let mut dbg = w.debug_tag();

        let line_color = style.color;
        let mut line_w = 0.9;
        let mut line_c = 0.7;
        let mut line_c_color = (style.color.0 * 0.5, style.color.1 * 0.5, style.color.2 * 0.5);
        if let StyleExt::Graph { graph_line, vline1, vline1_color, .. } = style.ext {
            line_w = graph_line;
            line_c = vline1;
            line_c_color = vline1_color;
        }

        let pos = self.live_area;
        let mut data = self.data.borrow_mut();

        let txt_h = p.font_height(style.font_size as f32, true);
        let val_pos = pos.resize(pos.w, txt_h);
        let grph_pos = pos.crop_top(txt_h);

        data.read(&mut self.minmax_buf[..]);

        let xd = 1.0 / (self.minmax_sample_count - 1) as f32;
        let mut x = 0.0;

        let mut last_minmax = (-1.0, 1.0);

        for i in 0..self.minmax_sample_count {
            let (min, max) = self.minmax_buf[i];

            let min = min.clamp(-1.0, 1.0) * WAVEFORM_SCALE_FACTOR;
            let max = max.clamp(-1.0, 1.0) * WAVEFORM_SCALE_FACTOR;
            let min = (min + 1.0) * 0.5;
            let max = (max + 1.0) * 0.5;

            // - 1.0 for preventing bleeding into the border.
            let gx = x * (grph_pos.w - 1.0) + 0.5;
            let gy1 = (1.0 - min) * grph_pos.h;
            let gy2 = (1.0 - max) * grph_pos.h;

            self.buf[i * 2] = ((grph_pos.x + gx), (grph_pos.y + gy1));

            if (last_minmax.1 - 0.00001) <= max {
                // (probably) Rising edge
                self.buf[i * 2 + 1] = ((grph_pos.x + gx + 0.5), (grph_pos.y + gy2));
            } else {
                // (probably) Falling edge
                self.buf[i * 2 + 1] = ((grph_pos.x + gx - 0.5), (grph_pos.y + gy2));
            }

            last_minmax = (min, max);

            x += xd;
        }

        p.path_stroke(
            line_c,
            line_c_color,
            &mut ([
                (grph_pos.x, grph_pos.y + grph_pos.h * 0.5),
                (grph_pos.x + grph_pos.w, grph_pos.y + grph_pos.h * 0.5),
            ]
            .iter()
            .copied()
            .map(|p| (p.0.floor(), p.1.floor() + 0.5))),
            false,
        );

        p.path_stroke(
            line_w,
            line_color,
            &mut self.buf.iter().copied().map(|p| (p.0, p.1 + 0.5)),
            false,
        );

        let len = data.fmt_val(&mut self.lbl_buf[..]);
        let val_s = std::str::from_utf8(&self.lbl_buf[0..len]).unwrap();
        p.label(
            style.font_size,
            0,
            style.color,
            val_pos.x,
            val_pos.y,
            val_pos.w,
            txt_h,
            val_s,
            dbg.source("graph_minmax_label"),
        );
    }
}
