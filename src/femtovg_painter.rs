// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use femtovg::{
    renderer::OpenGl,
    Canvas,
    FontId,
    ImageId,
    Color,
};

use crate::Painter;
use crate::Rect;
use crate::constants::*;
use crate::ImageRef;

use std::rc::Rc;
use std::cell::RefCell;

pub struct ImageStore {
    // TODO: FREE THESE IMAGES OR WE HAVE A MEMORY LEAK!
    freed_images: Vec<ImageId>,
}

pub struct FemtoImgRef {
    store:      Rc<RefCell<ImageStore>>,
    image_id:   ImageId,
    w:          f64,
    h:          f64,
}

impl ImageRef for FemtoImgRef {
    fn as_femto(&self) -> Option<&FemtoImgRef> {
        Some(self)
    }
}

impl Drop for FemtoImgRef {
    fn drop(&mut self) {
        self.store.borrow_mut().freed_images.push(self.image_id);
    }
}

pub struct PersistFemtovgData {
    store:  Rc<RefCell<ImageStore>>,
    offs_stk: Vec<(f64, f64)>,
}

impl PersistFemtovgData {
    pub fn new() -> Self {
        Self {
            offs_stk: vec![(0.0, 0.0)],
            store: Rc::new(RefCell::new(ImageStore {
                freed_images: vec![],
            })),
        }
    }
}

pub struct FemtovgPainter<'a, 'b> {
    pub canvas:     &'a mut Canvas<OpenGl>,
    pub font:       FontId,
    pub font_mono:  FontId,
    pub data:       &'b mut PersistFemtovgData,
    pub scale:      f32,
    pub cur_scale:  f32,

    pub test_debug: Option<Box<dyn FnMut(crate::AtomId, usize, &str, Rect)>>,
    pub cur_id_stk: Vec<crate::AtomId>,
}

fn color_paint(color: (f64, f64, f64)) -> femtovg::Paint {
    femtovg::Paint::color(
        Color::rgbf(
            color.0 as f32,
            color.1 as f32,
            color.2 as f32))
}

impl<'a, 'b> FemtovgPainter<'a, 'b> {
    #[allow(unused_variables)] // because if lblid if no driver is enabled
    fn label_with_font(
        &mut self, size: f64, align: i8, rot: f64, color: (f64, f64, f64),
        x: f64, y: f64, xoi: f64, yoi: f64, w: f64, h: f64,
        text: &str, font: FontId, lblid: usize)
    {
        let mut paint = color_paint(color);
        paint.set_font(&[font]);
        paint.set_font_size(size as f32);
        paint.set_text_baseline(femtovg::Baseline::Middle);
        let x = x.round();

        #[cfg(feature = "driver")]
        if let Some(f) = &mut self.test_debug {
            if let Some(id) = self.cur_id_stk.last().copied() {
                f(id, lblid, text, Rect { x, y, w, h });
            }
        }

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

impl<'a, 'b> Painter for FemtovgPainter<'a, 'b> {
    fn new_image(&mut self, w: f64, h: f64) -> Box<dyn ImageRef> {
        let image_id =
            self.canvas.create_image_empty(
                w as usize, h as usize,
                femtovg::PixelFormat::Rgba8,
                femtovg::ImageFlags::FLIP_Y)
                .expect("making image buffer");

        Box::new(FemtoImgRef {
            store:  self.data.store.clone(),
            w, h, image_id,
        })
    }

    fn start_image(&mut self, image: &dyn ImageRef, screen_x: f64, screen_y: f64) {
        self.data.offs_stk.push((screen_x, screen_y));
        self.canvas.set_render_target(
            femtovg::RenderTarget::Image(image.as_femto().unwrap().image_id));
    }

    fn finish_image(&mut self) {
        self.data.offs_stk.pop();
        self.canvas.set_render_target(femtovg::RenderTarget::Screen);
    }

    fn draw_image(&mut self, image: &dyn ImageRef, screen_x: f64, screen_y: f64) {
        self.canvas.set_render_target(femtovg::RenderTarget::Screen);

        let img_paint =
            femtovg::Paint::image(
                image.as_femto().unwrap().image_id,
                0.0, 0.0,
                image.as_femto().unwrap().w as f32,
                image.as_femto().unwrap().h as f32,
                0.0, 1.0);

        let mut path = femtovg::Path::new();
        path.rect(
            screen_x as f32, screen_y as f32,
            image.as_femto().unwrap().w as f32,
            image.as_femto().unwrap().h as f32);

        self.canvas.set_render_target(femtovg::RenderTarget::Screen);
        self.canvas.fill_path(&mut path, img_paint);
    }

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
        let mut pth = femtovg::Path::new();
        pth.rect(x as f32, y as f32, w as f32, h as f32);
        self.canvas.fill_path(&mut pth, color_paint(color));
    }

    fn rect_stroke(&mut self, width: f64, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64) {
        let mut pth = femtovg::Path::new();
        pth.rect(x as f32, y as f32, w as f32, h as f32);
        let mut paint = color_paint(color);
        paint.set_line_width(width as f32);
        self.canvas.stroke_path(&mut pth, paint);
    }

    fn label(&mut self, size: f64, align: i8, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64, text: &str, lblid: usize) {
        self.label_with_font(size, align, 0.0, color, x, y, 0.0, 0.0, w, h, text, self.font, lblid);
    }

    fn label_rot(&mut self, size: f64, align: i8, rot: f64, color: (f64, f64, f64), x: f64, y: f64, xo: f64, yo: f64, w: f64, h: f64, text: &str, lblid: usize) {
        self.label_with_font(size, align, rot, color, x, y, xo, yo, w, h, text, self.font, lblid);
    }

    fn label_mono(&mut self, size: f64, align: i8, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64, text: &str, lblid: usize) {
        self.label_with_font(size, align, 0.0, color, x, y, 0.0, 0.0, w, h, text, self.font_mono, lblid);
    }

    fn text_width(&mut self, size: f32, mono: bool, text: &str) -> f32 {
        let mut paint = color_paint(UI_PRIM_CLR);
        if mono {
            paint.set_font(&[self.font_mono]);
        } else {
            paint.set_font(&[self.font]);
        }
        paint.set_font_size(size);
        if let Ok(metr) = self.canvas.measure_text(0.0, 0.0, text, paint) {
            metr.width() // / (self.scale * self.cur_scale)
        } else {
            20.0
        }
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

    #[cfg(feature = "driver")]
    fn start_widget(&mut self, id: crate::AtomId) {
        self.cur_id_stk.push(id);
    }

    #[cfg(feature = "driver")]
    fn end_widget(&mut self, _id: crate::AtomId) {
        self.cur_id_stk.pop();
    }
}
