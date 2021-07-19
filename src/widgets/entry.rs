// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use crate::constants::*;
use super::*;

const BLINK_COUNT : usize = 30;

#[derive(Debug)]
pub struct Entry {
    font_size:  f64,
    rect:       Rect,
    txt_width:  usize,
    editable:   bool,
}

#[derive(Debug)]
pub struct EntryData {
    label:     String,
    count:     usize,
    curs_vis:  bool,
}

impl EntryData {
    pub fn new(label: &str) -> Box<dyn std::any::Any> {
        Box::new(Self {
            label: label.to_string(),
            count: 0,
            curs_vis: true,
        })
    }
}

impl Entry {
    pub fn new(width: f64, font_size: f64, txt_width: usize) -> Self {
        Self::new_ed(width, font_size, txt_width, true)
    }

    pub fn new_not_editable(width: f64, font_size: f64, txt_width: usize) -> Self {
        Self::new_ed(width, font_size, txt_width, false)
    }

    pub fn new_ed(width: f64, font_size: f64, txt_width: usize, editable: bool) -> Self {
        Self {
            font_size,
            txt_width,
            editable,
            rect: Rect::from(
                0.0, 0.0,
                width         + 2.0 * UI_PADDING + 2.0 * UI_BORDER_WIDTH,
                UI_ELEM_TXT_H + 2.0 * UI_PADDING + 2.0 * UI_BORDER_WIDTH + UI_ELEM_TXT_H)
        }
    }
}

impl WidgetType for Entry {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let pos       = self.rect.offs(pos.x, pos.y);
        let id        = data.id();
        let highlight = ui.hl_style_for(id, None);

        if self.editable {
            ui.define_active_zone(
                ActiveZone::new_input_zone(id, pos));
        }

        let border_color =
            match highlight {
                HLStyle::Hover(_) => UI_BTN_TXT_HOVER_CLR,
                _                 => UI_LIST_BORDER_CLR,
            };

        p.rect_fill(border_color, pos.x, pos.y, pos.w, pos.h);
        let pos = pos.shrink(UI_BORDER_WIDTH, UI_BORDER_WIDTH);
        p.rect_fill(UI_TAB_BG_CLR, pos.x, pos.y, pos.w, pos.h);

        let pos = pos.shrink(UI_SAFETY_PAD, UI_SAFETY_PAD);

        data.with(|data: &mut EntryData| {
            p.label(self.font_size, -1, UI_LBL_TXT_CLR,
                pos.x, pos.y, pos.w, UI_ELEM_TXT_H, &data.label,
                DBGID_ENTRY_LBL);

            p.path_stroke(
                1.0,
                UI_LBL_TXT_CLR,
                &mut [
                    (pos.x         - UI_SAFETY_PAD, pos.y + UI_ELEM_TXT_H + 0.5),
                    (pos.x + pos.w + UI_SAFETY_PAD, pos.y + UI_ELEM_TXT_H + 0.5),
                ].iter().copied(),
                false);

            data.count += 1;
            if data.count > BLINK_COUNT {
                data.count = 0;
                data.curs_vis = !data.curs_vis;
            };

            if !self.editable {
                data.curs_vis = false;
            }

            let txt_width = self.txt_width;

            if let Some(atom) = ui.atoms().get(id) {
                if let Some(s) = atom.str_ref() {
                    use std::io::Write;
                    let pos = pos.offs(0.0, UI_ELEM_TXT_H);
                    let mut buf : [u8; 128] = [0_u8; 128];
                    let mut bw = std::io::BufWriter::new(&mut buf[..]);

                    let len = s.chars().count();
                    let skip =
                        if len > txt_width { len - txt_width }
                        else { 0 };

                    if self.editable {
                        for c in s.chars().skip(skip) {
                            write!(bw, "{}", c).expect("write ok");
                        }
                    } else {
                        for c in s.chars().take(txt_width) {
                            write!(bw, "{}", c).expect("write ok");
                        }
                    }

                    if ui.is_input_value_for(id) && data.curs_vis {
                        write!(bw, "|").expect("write ok");
                    }

                    p.label_mono(self.font_size, -1, UI_LIST_TXT_CLR,
                        pos.x, pos.y + pos.h - 2.0 * UI_ELEM_TXT_H, pos.w, UI_ELEM_TXT_H,
                        &std::str::from_utf8(bw.buffer()).unwrap(),
                        DBGID_ENTRY_VAL);
                }
            }
        });
    }

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _avail: (f64, f64)) -> (f64, f64) {
        (self.rect.w, self.rect.h)
    }

    fn event(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, ev: &UIEvent) {
        match ev {
            _ => {},
        }
    }
}
