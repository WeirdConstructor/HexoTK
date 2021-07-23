// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use crate::constants::*;
use super::*;
use super::util::*;

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
    source:     Rc<dyn TextSource>,
    last_id:    usize,
    text:       Vec<String>,
    pages:      usize,
    page_idx:   usize,
}

#[allow(clippy::new_ret_no_self)]
impl TextData {
    pub fn new(source: Rc<dyn TextSource>) -> Box<dyn std::any::Any> {
        Box::new(Self {
            source,
            last_id:    0,
            page_idx:   0,
            pages:      1,
            text: vec!["".to_string()]})
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
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let pos =
            if self.no_padding { pos }
            else               { pos.shrink(UI_PADDING, UI_PADDING) };

        let id = data.id();

        let btn_height = UI_ELEM_TXT_H * 2.0;

        data.with(|data: &mut TextData| {
            if let Some((id, s)) = data.source.get(data.last_id) {
                data.text.clear();
                data.pages = 0;

                for part in s.split("\n---page---\n") {
                    data.text.push(part.to_string());
                    data.pages += 1;
                }

                data.last_id = id;
                data.page_idx = 0;
            }

            let (pos, btn_pos) =
                if data.text.len() > 1 {
                    let btn_height = btn_height + 2.0 * UI_BTN_BORDER2_WIDTH;
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
            for (i, line) in data.text[data.page_idx].split("\n").enumerate() {
                if first {
                    p.label_mono((self.font_size * 1.5).round(), 0,
                        UI_HELP_TXT_CLR,
                        xo, yo.floor(), pos.w, y_increment, line,
                        dbgid_pack(
                            DBGID_TEXT_HEADER,
                            data.page_idx as u16,
                            i as u16));
                    yo += y_increment;

                } else {
                    p.label_mono(self.font_size, -1,
                        UI_HELP_TXT_CLR,
                        xo, yo.floor(), pos.w, y_increment, line,
                        dbgid_pack(
                            DBGID_TEXT_HEADER,
                            data.page_idx as u16,
                            i as u16));
                }

                yo += y_increment;

                first = false;
            }

            if let Some(pos) = btn_pos {
                let btn_right_pos = pos.crop_left((pos.w - UI_BTN_WIDTH).max((pos.w / 2.0).floor()));
                let btn_left_pos  = pos.crop_right((pos.w - UI_BTN_WIDTH).max((pos.w / 2.0).floor()));
                let btn_left_pos =
                    rect_border(p,
                        UI_BTN_BORDER2_WIDTH,
                        UI_BTN_BORDER2_CLR,
                        UI_BTN_BG_CLR,
                        btn_left_pos);
                let btn_right_pos =
                    rect_border(p,
                        UI_BTN_BORDER2_WIDTH,
                        UI_BTN_BORDER2_CLR,
                        UI_BTN_BG_CLR,
                        btn_right_pos);

                ui.define_active_zone(
                    ActiveZone::new_indexed_click_zone(id, btn_left_pos, 0)
                    .dbgid(dbgid_pack(DBGID_TEXT_PGBTN, 0, 0)));
                ui.define_active_zone(
                    ActiveZone::new_indexed_click_zone(id, btn_right_pos, 1)
                    .dbgid(dbgid_pack(DBGID_TEXT_PGBTN, 1, 0)));

                let hlt_left  = ui.hl_style_for(id, Some(0));
                let hlt_right = ui.hl_style_for(id, Some(1));
                let btn_txt_left_clr =
                    match hlt_left {
                        HLStyle::Hover(_) => UI_BTN_TXT_HOVER_CLR,
                        _                 => UI_BTN_TXT_CLR,
                    };
                let btn_txt_right_clr =
                    match hlt_right {
                        HLStyle::Hover(_) => UI_BTN_TXT_HOVER_CLR,
                        _                 => UI_BTN_TXT_CLR,
                    };

                p.label(
                    self.font_size, 0,
                    btn_txt_left_clr,
                    btn_left_pos.x,
                    btn_left_pos.y.floor(),
                    btn_left_pos.w,
                    btn_height, "<",
                    dbgid_pack(DBGID_TEXT_PGBTN, data.page_idx as u16, 0));
                p.label(
                    self.font_size, 0,
                    btn_txt_right_clr,
                    btn_right_pos.x,
                    btn_right_pos.y.floor(),
                    btn_right_pos.w,
                    btn_height, ">",
                    dbgid_pack(DBGID_TEXT_PGBTN, data.page_idx as u16, 0));

                if pos.w > 3.0 * UI_BTN_WIDTH {
                    use std::io::Write;

                    let mut buf : [u8; 128] = [0_u8; 128];
                    let mut bw = std::io::BufWriter::new(&mut buf[..]);
                    let _ = write!(bw, "Page {}/{}", data.page_idx + 1, data.pages);

                    let lbl_left  = pos.x + ((pos.w / 2.0) - (UI_BTN_WIDTH / 2.0));
                    let lbl_pos = Rect::from(lbl_left, pos.y, UI_BTN_WIDTH, btn_height);
                    p.label(
                        self.font_size, 0,
                        UI_HELP_TXT_CLR,
                        lbl_pos.x,
                        lbl_pos.y.floor(),
                        lbl_pos.w,
                        btn_height,
                        &std::str::from_utf8(bw.buffer()).unwrap(),
                        DBGID_TEXT_PG);
                }
            }
        });
    }

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, avail: (f64, f64)) -> (f64, f64) {
        avail
    }

    fn event(&self, _ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        match ev {
            UIEvent::Click { id, index, .. } => {
                if data.id() == *id {
                    data.with(|data: &mut TextData| {
                        if *index == 0 {
                            data.page_idx = (data.page_idx + 1 + data.pages) % data.pages;
                        } else if *index == 1 {
                            data.page_idx = (data.page_idx + 1) % data.pages;
                        }
                    });
                }
            },
            _ => {}
        }
    }
}
