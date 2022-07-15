// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::{Style, Widget};

use crate::style::*;

use crate::painter::*;
use crate::rect::*;

use std::cell::RefCell;
use std::rc::Rc;

pub trait GraphModel {
    fn get_generation(&self) -> u64;
    fn f(&mut self, init: bool, x: f64, x_next: f64) -> f64;
    fn vline1_pos(&self) -> Option<f64>;
    fn vline2_pos(&self) -> Option<f64>;
}

#[derive(Debug, Clone)]
pub struct StaticGraphData {
    points: Vec<f64>,
    vline1: Option<f64>,
    vline2: Option<f64>,
    generation: u64,
}

impl StaticGraphData {
    pub fn new() -> Self {
        Self {
            points: vec![],
            vline1: None,
            vline2: None,
            generation: 0,
        }
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.generation += 1;
    }

    pub fn set_vline1(&mut self, x: Option<f64>) {
        self.vline1 = x;
        self.generation += 1;
    }

    pub fn set_vline2(&mut self, x: Option<f64>) {
        self.vline2 = x;
        self.generation += 1;
    }

    pub fn set_point(&mut self, i: usize, y: f64) {
        if self.points.len() <= i {
            self.points.resize(i + 1, 0.0);
        }
        self.points[i] = y;
        self.generation += 1;
    }
}

impl GraphModel for StaticGraphData {
    fn get_generation(&self) -> u64 {
        self.generation
    }
    fn f(&mut self, _init: bool, x: f64, _x_next: f64) -> f64 {
        if self.points.is_empty() {
            return 0.0;
        }

        let x = (self.points.len() - 1) as f64 * x.clamp(0.0, 1.0);
        let i_start = (x.floor() as usize).clamp(0, self.points.len() - 1);
        let i_end = (x.ceil() as usize).clamp(0, self.points.len() - 1);
        let xr = x - x.floor();

        self.points[i_start] * (1.0 - xr) + self.points[i_end] * xr
    }
    fn vline1_pos(&self) -> Option<f64> {
        self.vline1
    }
    fn vline2_pos(&self) -> Option<f64> {
        self.vline2
    }
}

pub struct Graph {
    data: Rc<RefCell<dyn GraphModel>>,
    draw_buf: Vec<(f32, f32)>,
    live_area: Rect,
    live_draw: bool,
    samples: u16,
    vline1_pos: Option<[(f32, f32); 2]>,
    vline2_pos: Option<[(f32, f32); 2]>,
}

impl Graph {
    pub fn new(data: Rc<RefCell<dyn GraphModel>>, samples: u16, live_draw: bool) -> Self {
        Self {
            live_area: Rect::from(0.0, 0.0, 0.0, 0.0),
            draw_buf: vec![],
            vline1_pos: None,
            vline2_pos: None,
            samples,
            data,
            live_draw,
        }
    }

    pub fn get_generation(&self) -> u64 {
        self.data.borrow().get_generation()
    }

    fn draw_samples(&mut self, pos: Rect) {
        let samples = self.samples as f64;
        let mut data = self.data.borrow_mut();

        let mut x: f64 = 0.0;
        let xd = 1.0 / (samples - 1.0);
        for i in 0..(self.samples as usize) {
            let gx = x * (pos.w as f64);
            let gy = (1.0 - data.f(i == 0, x, x + xd).clamp(0.0, 1.0)) * pos.h as f64;

            self.draw_buf[i] = ((pos.x + (gx as f32)), (pos.y + (gy as f32)));

            x += xd;
        }

        if let Some(x) = data.vline1_pos() {
            let rx = ((pos.w as f64 * x) as f32).round();
            self.vline1_pos = Some([(pos.x + rx, pos.y), (pos.x + rx, pos.y + pos.h)]);
        } else {
            self.vline1_pos = None;
        }

        if let Some(x) = data.vline2_pos() {
            let rx = ((pos.w as f64 * x) as f32).round();
            self.vline2_pos = Some([(pos.x + rx, pos.y), (pos.x + rx, pos.y + pos.h)]);
        } else {
            self.vline2_pos = None;
        }
    }

    fn draw_graph(&mut self, style: &Style, p: &mut Painter) {
        let line_color = style.color;
        let mut line_w = 1.0;
        let mut line1 = 1.0;
        let mut line2 = 1.0;
        let mut line1_color = style.border_color;
        let mut line2_color = style.border_color;
        if let StyleExt::Graph {
            graph_line,
            vline1,
            vline2,
            vline1_color,
            vline2_color,
            ..
        } = style.ext
        {
            line_w = graph_line;
            line1 = vline1;
            line2 = vline2;
            line1_color = vline1_color;
            line2_color = vline2_color;
        }

        p.path_stroke(
            line_w,
            line_color,
            &mut self.draw_buf.iter().copied(),
            false,
        );

        if let Some(linepos) = &self.vline1_pos {
            p.path_stroke(line1, line1_color, &mut linepos.iter().copied(), false);
        }

        if let Some(linepos) = &self.vline2_pos {
            p.path_stroke(line2, line2_color, &mut linepos.iter().copied(), false);
        }
    }

    pub fn draw(&mut self, _w: &Widget, style: &Style, pos: Rect, real_pos: Rect, p: &mut Painter) {
        self.live_area = real_pos;

        if self.draw_buf.len() != (self.samples as usize) {
            self.draw_buf.resize(self.samples as usize, (0.0, 0.0));
        }

        if let StyleExt::Graph {
            hline, hline_color, ..
        } = style.ext
        {
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

        if self.live_draw {
            return;
        }

        self.draw_samples(pos);
        self.draw_graph(style, p);
    }

    pub fn draw_frame(&mut self, _w: &Widget, style: &Style, p: &mut Painter) {
        if !self.live_draw {
            return;
        }

        self.draw_samples(self.live_area);
        self.draw_graph(style, p);
    }
}
