// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use femtovg::{
    renderer::OpenGl,
    Canvas,
    FontId,
    ImageId,
    Color,
};

use crate::Painter;
use crate::constants::*;

pub struct FemtovgPainter<'a> {
    pub canvas:     &'a mut Canvas<OpenGl>,
    pub font:       FontId,
    pub font_mono:  FontId,
    pub scale:      f32,
}

fn color_paint(color: (f64, f64, f64)) -> femtovg::Paint {
    femtovg::Paint::color(
        Color::rgbf(
            color.0 as f32,
            color.1 as f32,
            color.2 as f32))
}

impl<'a> FemtovgPainter<'a> {
    fn label_with_font(&mut self, size: f64, align: i8, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64, text: &str, font: FontId) {
        let mut paint = color_paint(color);
        paint.set_font(&[font]);
        paint.set_font_size(size as f32);
        paint.set_text_baseline(femtovg::Baseline::Middle);
        let x = x.round();
        match align {
            -1 => {
                paint.set_text_align(femtovg::Align::Left);
                self.canvas.fill_text(x as f32, (y + h / 2.0).round() as f32, text, paint);
            },
            0  => {
                paint.set_text_align(femtovg::Align::Center);
                self.canvas.fill_text((x + (w / 2.0)) as f32, (y + h / 2.0).round() as f32, text, paint);
            },
            _  => {
                paint.set_text_align(femtovg::Align::Right);
                self.canvas.fill_text((x + w) as f32, (y + h / 2.0).round() as f32, text, paint);
            },
        }
    }
}

impl<'a> Painter for FemtovgPainter<'a> {
    fn path_fill(&mut self, color: (f64, f64, f64), segments: &mut dyn std::iter::Iterator<Item = (f64, f64)>, closed: bool) {
        let mut p = femtovg::Path::new();
        let mut paint = color_paint(color);

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

    fn path_stroke(&mut self, width: f64, color: (f64, f64, f64), segments: &mut dyn std::iter::Iterator<Item = (f64, f64)>, closed: bool) {
        let mut p = femtovg::Path::new();
        let mut paint = color_paint(color);
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
        self.label_with_font(size, align, color, x, y, w, h, text, self.font);
    }

    fn label_mono(&mut self, size: f64, align: i8, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64, text: &str) {
        self.label_with_font(size, align, color, x, y, w, h, text, self.font_mono);
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
            metr.height() / self.scale
        } else {
            UI_ELEM_TXT_H as f32
        }
    }
}
