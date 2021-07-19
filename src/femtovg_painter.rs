// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use femtovg::{
    renderer::OpenGl,
    Canvas,
    FontId,
    Color,
};

use crate::Painter;
use crate::constants::*;

pub struct FemtovgPainter<'a> {
    pub canvas:     &'a mut Canvas<OpenGl>,
    pub font:       FontId,
    pub font_mono:  FontId,
//    pub images:     Vec<Option<ImageId>>,
    pub scale:      f32,
    pub cur_scale:  f32,

    pub test_debug: Option<Box<dyn FnMut(crate::AtomId, usize, &str)>>,
    pub cur_id_stk: Vec<crate::AtomId>,
    pub dbg_count:  usize,
}

fn color_paint(color: (f64, f64, f64)) -> femtovg::Paint {
    femtovg::Paint::color(
        Color::rgbf(
            color.0 as f32,
            color.1 as f32,
            color.2 as f32))
}

impl<'a> FemtovgPainter<'a> {
    fn label_with_font(&mut self, size: f64, align: i8, rot: f64, color: (f64, f64, f64), x: f64, y: f64, xoi: f64, yoi: f64, w: f64, h: f64, text: &str, font: FontId) {
        #[cfg(debug_assertions)]
        if let Some(f) = &mut self.test_debug {
            f(self.cur_id_stk.last().copied().unwrap(), self.dbg_count, text);
            self.dbg_count += 1;
        }

        let mut paint = color_paint(color);
        paint.set_font(&[font]);
        paint.set_font_size(size as f32);
        paint.set_text_baseline(femtovg::Baseline::Middle);
        let x = x.round();

        let (x, y) =
            if rot > 0.0 {
                self.canvas.save();
                let x = x as f32;
                let y = y as f32;
                let wh = (w / 2.0) as f32;
                let hh = (h / 2.0) as f32;

                let rot = rot.to_radians() as f32;

                self.canvas.translate(x + wh, y + hh);
                self.canvas.rotate(rot);
                self.canvas.translate(xoi as f32, yoi as f32);

                (-wh as f64, -hh as f64)
            } else {
                (x, y)
            };

//        let mut p = femtovg::Path::new();
//        p.rect(x as f32, y as f32, w as f32, h as f32);
//        self.canvas.stroke_path(&mut p, paint);
        match align {
            -1 => {
                paint.set_text_align(femtovg::Align::Left);
                let _ =
                    self.canvas.fill_text(
                        x as f32,
                        (y + h / 2.0).round() as f32,
                        text, paint);
            },
            0  => {
                paint.set_text_align(femtovg::Align::Center);
                let _ =
                    self.canvas.fill_text(
                        (x + (w / 2.0)) as f32,
                        (y + h / 2.0).round() as f32,
                        text, paint);
            },
            _  => {
                paint.set_text_align(femtovg::Align::Right);
                let _ =
                    self.canvas.fill_text(
                        (x + w) as f32,
                        (y + h / 2.0).round() as f32,
                        text, paint);
            },
        }

//        let mut p = femtovg::Path::new();
//        let mut paint2 = color_paint((1.0, 1.0, 1.0));
//        p.rect((x - 1.0) as f32, (y - 1.0) as f32, 2.0, 2.0);
//        p.rect(((x + 0.5 * w) - 1.0) as f32, ((y + 0.5 * h) - 1.0) as f32, 2.0, 2.0);
//        self.canvas.stroke_path(&mut p, paint2);

        if rot > 0.0 {
//            self.canvas.translate(-(0.5 * w) as f32, 0.0);
            self.canvas.restore();
        }
    }
}

impl<'a> Painter for FemtovgPainter<'a> {
    fn clip_region(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.canvas.save();
        self.canvas.scissor(x as f32, y as f32, w as f32, h as f32);
    }

    fn move_and_scale(&mut self, x: f64, y: f64, x2: f64, y2: f64, factor: f64) {
        self.canvas.save();
        self.cur_scale = factor as f32;
        let factor = self.cur_scale;
//        self.canvas.translate(x as f32, y as f32);
        self.canvas.translate(x as f32, y as f32);
        self.canvas.scale(factor, factor);
        self.canvas.translate(x2 as f32, y2 as f32);
//        self.canvas.translate(-x as f32 / factor, -y as f32 / factor);
    }

    fn reset_scale(&mut self) {
        self.cur_scale = 1.0;
        self.canvas.restore();
    }

    fn reset_clip_region(&mut self) {
        self.canvas.reset_scissor();
        self.canvas.restore();
    }

    fn path_fill_rot(&mut self, color: (f64, f64, f64),
                     rot: f64, x: f64, y: f64, xo: f64, yo: f64,
                     segments: &mut dyn std::iter::Iterator<Item = (f64, f64)>,
                     closed: bool) {

        self.canvas.save();
        let rot = rot.to_radians();

        self.canvas.translate(x as f32, y as f32);
        self.canvas.rotate(rot as f32);
        self.canvas.translate(xo as f32, yo as f32);

        self.path_fill(color, segments, closed);

        self.canvas.restore();
    }

    fn path_stroke_rot(&mut self, width: f64, color: (f64, f64, f64),
                       rot: f64, x: f64, y: f64, xo: f64, yo: f64,
                       segments: &mut dyn std::iter::Iterator<Item = (f64, f64)>,
                       closed: bool) {

        self.canvas.save();
        let rot = rot.to_radians();

        self.canvas.translate(x as f32, y as f32);
        self.canvas.rotate(rot as f32);
        self.canvas.translate(xo as f32, yo as f32);

        self.path_stroke(width, color, segments, closed);

        self.canvas.restore();
    }

    fn path_fill(&mut self, color: (f64, f64, f64), segments: &mut dyn std::iter::Iterator<Item = (f64, f64)>, closed: bool) {
        let mut p = femtovg::Path::new();
        let paint = color_paint(color);

        let mut first = true;
        for s in segments {
            if first {
                p.move_to(s.0 as f32, s.1 as f32);
                first = false;
            } else {
                p.line_to(s.0 as f32, s.1 as f32);
            }
        }

        if closed { p.close(); }

        self.canvas.fill_path(&mut p, paint);
    }

    fn path_stroke(&mut self, width: f64, color: (f64, f64, f64),
                   segments: &mut dyn std::iter::Iterator<Item = (f64, f64)>,
                   closed: bool)
    {
        let mut p = femtovg::Path::new();
        let mut paint = color_paint(color);
        paint.set_line_join(femtovg::LineJoin::Round);
        paint.set_line_width(width as f32);

        let mut first = true;
        for s in segments {
            if first {
                p.move_to(s.0 as f32, s.1 as f32);
                first = false;
            } else {
                p.line_to(s.0 as f32, s.1 as f32);
            }
        }

        if closed { p.close(); }

        self.canvas.stroke_path(&mut p, paint);
    }

    fn arc_stroke(&mut self, width: f64, color: (f64, f64, f64), radius: f64, from_rad: f64, to_rad: f64, x: f64, y: f64) {
        let mut p = femtovg::Path::new();
        let mut paint = color_paint(color);
        paint.set_line_width(width as f32);
        p.arc(x as f32, y as f32, radius as f32, from_rad as f32, to_rad as f32, femtovg::Solidity::Hole);
        self.canvas.stroke_path(&mut p, paint);
    }

    fn rect_fill(&mut self, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64) {
        let mut p = femtovg::Path::new();
        p.rect(x as f32, y as f32, w as f32, h as f32);
        self.canvas.fill_path(&mut p, color_paint(color));
    }

    fn rect_stroke(&mut self, width: f64, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64) {
        let mut p = femtovg::Path::new();
        p.rect(x as f32, y as f32, w as f32, h as f32);
        let mut paint = color_paint(color);
        paint.set_line_width(width as f32);
        self.canvas.stroke_path(&mut p, paint);
    }

    fn label(&mut self, size: f64, align: i8, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64, text: &str) {
        self.label_with_font(size, align, 0.0, color, x, y, 0.0, 0.0, w, h, text, self.font);
    }

    fn label_rot(&mut self, size: f64, align: i8, rot: f64, color: (f64, f64, f64), x: f64, y: f64, xo: f64, yo: f64, w: f64, h: f64, text: &str) {
        self.label_with_font(size, align, rot, color, x, y, xo, yo, w, h, text, self.font);
    }

    fn label_mono(&mut self, size: f64, align: i8, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64, text: &str) {
        self.label_with_font(size, align, 0.0, color, x, y, 0.0, 0.0, w, h, text, self.font_mono);
    }

    fn font_height(&mut self, size: f32, mono: bool) -> f32 {
        let mut paint = color_paint(UI_PRIM_CLR);
        if mono {
            paint.set_font(&[self.font_mono]);
        } else {
            paint.set_font(&[self.font]);
        }
        paint.set_font_size(size);
        if let Ok(metr) = self.canvas.measure_font(paint) {
            metr.height() / (self.scale * self.cur_scale)
        } else {
            UI_ELEM_TXT_H as f32
        }
    }

    #[cfg(debug_assertions)]
    fn start_widget(&mut self, id: crate::AtomId) {
        self.cur_id_stk.push(id);
        self.dbg_count = 0;
    }

    #[cfg(debug_assertions)]
    fn end_widget(&mut self, _id: crate::AtomId) {
        self.cur_id_stk.pop();
    }
}
