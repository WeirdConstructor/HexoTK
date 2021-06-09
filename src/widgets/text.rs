// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use crate::constants::*;
use super::*;

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
pub struct TextSourceRef {
    text:  Rc<RefCell<(usize, String)>>,
    width: usize,
}

impl TextSourceRef {
    pub fn new(line_width: usize) -> Self {
        Self {
            text:  Rc::new(RefCell::new((0, "".to_string()))),
            width: line_width,
        }
    }

    pub fn set(&self, s: &str) {
        let s =
            if self.width > 0 {
                let mut text : String = String::new();

                let mut line_len = 0;
                for c in s.chars() {
                    text.push(c);
                    if c == '\n' {
                        line_len = 0;
                    } else {
                        line_len += 1;
                    }

                    if line_len >= self.width {
                        text.push('\n');
                        line_len = 0;
                    }
                }

                text
            } else {
                s.to_string()
            };

        let mut bor = self.text.borrow_mut();
        bor.0 += 1;
        bor.1 = s;
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
    no_padding: bool,
}

pub struct TextData {
    source:  Rc<dyn TextSource>,
    last_id: usize,
    text:    Vec<String>,
}

impl TextData {
    pub fn new(source: Rc<dyn TextSource>) -> Box<dyn std::any::Any> {
        Box::new(Self {
            source,
            last_id: 0,
            text: vec!["".to_string()]})
            // , "Test\n1 2 3 4 5\nfeofeow eow".to_string()] })
    }
}


impl Text {
    pub fn new(font_size: f64) -> Self {
        Self {
            font_size,
            no_padding: false,
        }
    }

    pub fn new_no_padding(font_size: f64) -> Self {
        Self {
            font_size,
            no_padding: true
        }
    }
}

impl WidgetType for Text {
    fn draw(&self, _ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let pos =
            if self.no_padding { pos }
            else               { pos.shrink(UI_PADDING, UI_PADDING) };

        data.with(|data: &mut TextData| {
            if let Some((id, s)) = data.source.get(data.last_id) {
                data.text[0] = s;
                data.last_id = id;
            }

            let (pos, btn_pos) =
                if data.text.len() > 1 {
                    let btn_height = UI_ELEM_TXT_H + 2.0 * UI_BORDER_WIDTH;
                    let btn_pos =
                        pos.crop_top(
                            pos.h - btn_height);
                    (pos.crop_bottom(btn_height), Some(btn_pos))
                } else {
                    (pos, None)
                };

            let xo     = pos.x;
            let mut yo = pos.y;

            let y_increment =
                p.font_height(self.font_size as f32, true) as f64;

            let mut first = true;
            for line in data.text[0].split("\n") {
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

            if let Some(pos) = btn_pos {
                p.rect_fill(
                    (0.0, 0.0, 0.0),
                    pos.x,
                    pos.y,
                    pos.w,
                    pos.h);
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
