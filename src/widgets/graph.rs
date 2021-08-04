// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::constants::*;
use super::*;

#[derive(Debug)]
pub struct Graph {
    width:      f64,
    height:     f64,
}

pub struct GraphData {
    samples:   usize,
    func:      Box<dyn FnMut(&dyn WidgetUI, bool, f64, f64) -> f64>,
    buf:       Vec<(f64, f64)>,
}

#[allow(clippy::new_ret_no_self)]
impl GraphData {
    pub fn new(
        samples: usize,
        func: Box<dyn FnMut(&dyn WidgetUI, bool, f64, f64) -> f64>
    ) -> Box<dyn std::any::Any>
    {
        let mut buf = vec![];
        buf.resize(samples, (0.0, 0.0));

        Box::new(Self {
            samples,
            func,
            buf,
        })
    }
}

impl Graph {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height
        }
    }
}

impl WidgetType for Graph {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let out_pos = Rect::from(pos.x, pos.y, self.width, self.height);
        let in_pos  = pos.shrink(UI_GRPH_BORDER, UI_GRPH_BORDER);

        p.rect_fill(UI_GRPH_BORDER_CLR, out_pos.x, out_pos.y, out_pos.w, out_pos.h);
        p.rect_fill(UI_GRPH_BG,         in_pos.x,  in_pos.y,  in_pos.w,  in_pos.h);

        data.with(|data: &mut GraphData| {
            let xd = 1.0 / (data.samples - 1) as f64;
            let mut x = 0.0;

            for i in 0..data.samples {
                let gx = x * in_pos.w;
                let gy =
                    (1.0 - (*data.func)(ui, i == 0, x, x + xd).clamp(0.0, 1.0))
                    * in_pos.h;

                data.buf[i] = (
                    (in_pos.x + gx),
                    (in_pos.y + gy)
                );
                x += xd;
            }

            p.path_stroke(
                1.0,
                UI_GRPH_LINE_CLR,
                &mut data.buf.iter().copied(),
                false);
        });
    }

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _avail: (f64, f64)) -> (f64, f64) {
        (self.width, self.height)
    }

    fn event(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _ev: &UIEvent) {
    }
}
