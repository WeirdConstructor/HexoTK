// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use crate::constants::*;
use super::*;

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
pub struct TextSourceRef {
    text: Rc<RefCell<(usize, String)>>,
}

impl TextSourceRef {
    pub fn new() -> Self {
        Self {
            text: Rc::new(RefCell::new((0, "".to_string()))),
        }
    }

    pub fn set(&self, s: &str) {
        let mut bor = self.text.borrow_mut();
        bor.0 += 1;
        bor.1 = s.to_string();
    }
}

pub trait TextSource {
    fn get(&self, last_id: usize) -> Option<(usize, String)>;
}

impl TextSource for TextSourceRef {
    fn get(&self, last_id: usize) -> Option<(usize, String)> {
        let bor = self.text.borrow();
        if bor.0 > last_id {
            Some(bor.clone())
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Text {
    font_size:  f64,
}

pub struct TextData {
    source:  Rc<dyn TextSource>,
    last_id: usize,
    text:    String,
}

impl TextData {
    pub fn new(source: Rc<dyn TextSource>) -> Box<dyn std::any::Any> {
        Box::new(Self { source, last_id: 0, text: "".to_string() })
    }
}


impl Text {
    pub fn new(font_size: f64) -> Self {
        Self {
            font_size,
        }
    }
}

impl WidgetType for Text {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let pos = pos.shrink(UI_PADDING, UI_PADDING);

        data.with(|data: &mut TextData| {
            if let Some((id, s)) = data.source.get(data.last_id) {
                data.text    = s;
                data.last_id = id;
            }

            let xo     = pos.x;
            let mut yo = pos.y;

            p.rect_fill(UI_BG_CLR, pos.x, pos.y, pos.w, pos.h);

            let y_increment =
                p.font_height(self.font_size as f32, true) as f64;

            let mut first = true;
            for line in data.text.split("\n") {
                if first {
                    p.label_mono((self.font_size * 1.5).round(), 0,
                        UI_HELP_TXT_CLR,
                        xo, yo.floor(), pos.w, y_increment, line);
                    yo += y_increment;

                } else {
                    p.label_mono(self.font_size, -1,
                        UI_HELP_TXT_CLR,
                        xo, yo.floor(), pos.w, y_increment, line);
                }

                yo += y_increment;

                first = false;
            }
        });
    }

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, avail: (f64, f64)) -> (f64, f64) {
        avail
    }

    fn event(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _ev: &UIEvent) {
//        match ev {
//            _ => {},
//        }
    }
}
