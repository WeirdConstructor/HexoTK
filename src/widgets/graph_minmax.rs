// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use crate::constants::*;
use super::*;

#[derive(Debug)]
pub struct GraphMinMax {
    width:      f64,
    height:     f64,
}

pub struct GraphMinMaxData {
    func:                 Box<dyn FnMut(&dyn WidgetUI, usize) -> (f64, f64)>,
    buf:                  Vec<(f64, f64)>,
    minmax_sample_count:  usize,
}

impl GraphMinMaxData {
    pub fn new(minmax_sample_count: usize, func: Box<dyn FnMut(&dyn WidgetUI, usize) -> (f64, f64)>) -> Box<dyn std::any::Any> {
        let mut buf = vec![];
        buf.resize(2 * minmax_sample_count, (0.0, 0.0));

        Box::new(Self {
            func,
            buf,
            minmax_sample_count,
        })
    }
}

impl GraphMinMax {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height
        }
    }
}

impl WidgetType for GraphMinMax {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let out_pos = Rect::from(pos.x, pos.y, self.width, self.height);
        let in_pos  = pos.shrink(UI_GRPH_BORDER, UI_GRPH_BORDER);

        p.rect_fill(UI_GRPH_BORDER_CLR, out_pos.x, out_pos.y, out_pos.w, out_pos.h);
        p.rect_fill(UI_GRPH_BG,         in_pos.x,  in_pos.y,  in_pos.w,  in_pos.h);

        data.with(|data: &mut GraphMinMaxData| {
            let xd = 1.0 / (data.minmax_sample_count - 1) as f64;
            let mut x = 0.0;

            for i in 0..data.minmax_sample_count {

                let (min, max) = (*data.func)(ui, i);
                let min = (min.clamp(-1.0, 1.0) + 1.0) * 0.5;
                let max = (max.clamp(-1.0, 1.0) + 1.0) * 0.5;

                let gx = x * in_pos.w;
                let gy1 = (1.0 - min) * in_pos.h;
                let gy2 = (1.0 - max) * in_pos.h;

                data.buf[i * 2] = (
                    (in_pos.x + gx),
                    (in_pos.y + gy1)
                );
                data.buf[i * 2 + 1] = (
                    (in_pos.x + gx),
                    (in_pos.y + gy2)
                );

                x += xd;
            }

            p.path_stroke(
                0.75,
                UI_GRPH_LINE_CLR,
                &mut data.buf.iter().copied(),
                false);

            p.path_stroke(
                0.5,
                UI_GRPH_BORDER_CLR,
                &mut ([
                    (in_pos.x           , in_pos.y + in_pos.h * 0.5),
                    (in_pos.x + in_pos.w, in_pos.y + in_pos.h * 0.5),
                ].iter().copied().map(|p| (p.0.floor(), p.1.floor()))), false);
        });
    }

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _avail: (f64, f64)) -> (f64, f64) {
        (self.width, self.height)
    }

    fn event(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, ev: &UIEvent) {
        match ev {
            _ => {},
        }
    }
}
