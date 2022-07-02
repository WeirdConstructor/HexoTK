// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::{Widget, InputEvent, Event, MButton, EvPayload, Style, Mutable};

use crate::style::*;

use crate::painter::*;
use crate::rect::*;

use std::rc::Rc;
use std::cell::RefCell;

pub trait GraphModel {
    fn get_generation(&self) -> u64;
    fn f(&self, init: bool, x: f64, x_next: f64) -> f64;
    fn vline1_pos(&self) -> Option<f64>;
    fn vline2_pos(&self) -> Option<f64>;
}

#[derive(Debug, Clone)]
pub struct StaticGraphData {
    points:     Vec<f64>,
    vline1:     Option<f64>,
    vline2:     Option<f64>,
    generation: u64,
}

impl StaticGraphData {
    pub fn new() -> Self {
        Self {
            points:     vec![],
            vline1:     None,
            vline2:     None,
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
    fn get_generation(&self) -> u64 { self.generation }
    fn f(&self, init: bool, x: f64, x_next: f64) -> f64 {
        if self.points.is_empty() { return 0.0; }

        let x       = (self.points.len() - 1) as f64 * x.clamp(0.0, 1.0);
        let i_start = (x.floor() as usize).clamp(0, self.points.len() - 1);
        let i_end   = (x.ceil() as usize).clamp(0, self.points.len() - 1);
        let xr      = x - x.floor();

          self.points[i_start] * (1.0 - xr)
        + self.points[i_end] * xr
    }
    fn vline1_pos(&self) -> Option<f64> { self.vline1 }
    fn vline2_pos(&self) -> Option<f64> { self.vline2 }
}

pub struct Graph {
    data:            Rc<RefCell<dyn GraphModel>>,
    draw_buf:        Vec<(f32, f32)>,
    live_area:       Rect,
    live_draw:       bool,
    samples:         f64,
    sampling_factor: f32,
    vline1_pos:      Option<[(f32, f32); 2]>,
    vline2_pos:      Option<[(f32, f32); 2]>,
}

impl Graph {
    pub fn new(data: Rc<RefCell<dyn GraphModel>>, sampling_factor: f32, live_draw: bool) -> Self {
        Self {
            live_area: Rect::from(0.0, 0.0, 0.0, 0.0),
            draw_buf:  vec![],
            samples:   0.0,
            vline1_pos: None,
            vline2_pos: None,
            sampling_factor,
            data,
            live_draw,
        }
    }

    pub fn get_generation(&self) -> u64 {
        self.data.borrow().get_generation()
    }

    fn draw_samples(&mut self, pos: Rect) {
        let samples = self.samples;
        let data = self.data.borrow();

        let mut x : f64 = 0.0;
        let xd = 1.0 / (samples - 1.0);
        for i in 0..(samples as usize) {
            let gx = x * (pos.w as f64);
            let gy =
                (1.0 - data.f(i == 0, x, x + xd).clamp(0.0, 1.0))
                * pos.h as f64;

            self.draw_buf[i] = (
                (pos.x + (gx as f32)),
                (pos.y + (gy as f32))
            );

            x += xd;
        }

        if let Some(x) = data.vline1_pos() {
            let rx = ((pos.w as f64 * x) as f32).round();
            self.vline1_pos = Some([
                (pos.x + rx, pos.y),
                (pos.x + rx, pos.y + pos.h),
            ]);
        } else {
            self.vline1_pos = None;
        }

        if let Some(x) = data.vline2_pos() {
            let rx = ((pos.w as f64 * x) as f32).round();
            self.vline2_pos = Some([
                (pos.x + rx, pos.y),
                (pos.x + rx, pos.y + pos.h),
            ]);
        } else {
            self.vline2_pos = None;
        }
    }

    fn draw_graph(&mut self, style: &Style, p: &mut Painter) {
        let line_color = style.color;
        let mut line_w      = 1.0;
        let mut line1       = 1.0;
        let mut line2       = 1.0;
        let mut line1_color = style.border_color;
        let mut line2_color = style.border_color;
        if let StyleExt::Graph {
            graph_line, vline1, vline2, vline1_color, vline2_color
        } = style.ext {
            line_w      = graph_line;
            line1       = vline1;
            line2       = vline2;
            line1_color = vline1_color;
            line2_color = vline2_color;
        }

        p.path_stroke(
            line_w,
            line_color,
            &mut self.draw_buf.iter().copied(),
            false);

        if let Some(linepos) = &self.vline1_pos {
            p.path_stroke(
                line1,
                line1_color,
                &mut linepos.iter().copied(),
                false);
        }

        if let Some(linepos) = &self.vline2_pos {
            p.path_stroke(
                line2,
                line2_color,
                &mut linepos.iter().copied(),
                false);
        }
    }

    pub fn draw(
        &mut self, w: &Widget, style: &Style, pos: Rect,
        real_pos: Rect, p: &mut Painter)
    {
        self.live_area = real_pos;

        let samples = (pos.w * self.sampling_factor).floor() as usize;
        if self.draw_buf.len() != samples {
            self.draw_buf.resize(samples, (0.0, 0.0));
        }

        self.samples = samples as f64;

        if self.live_draw { return; }

        self.draw_samples(pos);
        self.draw_graph(style, p);
    }

    pub fn draw_frame(&mut self, w: &Widget, style: &Style, p: &mut Painter) {
        if !self.live_draw { return; }

        self.draw_samples(self.live_area);
        self.draw_graph(style, p);
    }
}

//use crate::constants::*;
//use super::*;
//
//#[derive(Debug)]
//pub struct Graph {
//    width:      f64,
//    height:     f64,
//}
//
//pub struct GraphData {
//    samples:   usize,
//    func:      Box<dyn FnMut(&dyn WidgetUI, bool, f64, f64) -> f64>,
//    buf:       Vec<(f64, f64)>,
//}
//
//#[allow(clippy::new_ret_no_self)]
//impl GraphData {
//    pub fn new(
//        samples: usize,
//        func: Box<dyn FnMut(&dyn WidgetUI, bool, f64, f64) -> f64>
//    ) -> Box<dyn std::any::Any>
//    {
//        let mut buf = vec![];
//        buf.resize(samples, (0.0, 0.0));
//
//        Box::new(Self {
//            samples,
//            func,
//            buf,
//        })
//    }
//}
//
//impl Graph {
//    pub fn new(width: f64, height: f64) -> Self {
//        Self {
//            width,
//            height
//        }
//    }
//}

//impl WidgetType for Graph {
//    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
//        let out_pos = Rect::from(pos.x, pos.y, self.width, self.height);
//        let in_pos  = pos.shrink(UI_GRPH_BORDER, UI_GRPH_BORDER);
//
//        p.rect_fill(UI_GRPH_BORDER_CLR, out_pos.x, out_pos.y, out_pos.w, out_pos.h);
//        p.rect_fill(UI_GRPH_BG,         in_pos.x,  in_pos.y,  in_pos.w,  in_pos.h);
//
//        data.with(|data: &mut GraphData| {
//            let xd = 1.0 / (data.samples - 1) as f64;
//            let mut x = 0.0;
//
//            for i in 0..data.samples {
//                let gx = x * in_pos.w;
//                let gy =
//                    (1.0 - (*data.func)(ui, i == 0, x, x + xd).clamp(0.0, 1.0))
//                    * in_pos.h;
//
//                data.buf[i] = (
//                    (in_pos.x + gx),
//                    (in_pos.y + gy)
//                );
//                x += xd;
//            }
//
//            p.path_stroke(
//                1.0,
//                UI_GRPH_LINE_CLR,
//                &mut data.buf.iter().copied(),
//                false);
//        });
//    }
//
//    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _avail: (f64, f64)) -> (f64, f64) {
//        (self.width, self.height)
//    }
//
//    fn event(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _ev: &UIEvent) {
//    }
//}
