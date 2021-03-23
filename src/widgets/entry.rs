// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use crate::constants::*;
use super::*;

#[derive(Debug)]
pub struct Entry {
    font_size:  f64,
    rect:       Rect,
}

#[derive(Debug)]
pub struct EntryData {
    label:     String,
    txt_width: usize,
}

impl EntryData {
    pub fn new(label: &str, txt_width: usize) -> Box<dyn std::any::Any> {
        Box::new(Self {
            label: label.to_string(),
            txt_width,
        })
    }
}

impl Entry {
    pub fn new(width: f64, font_size: f64) -> Self {
        Self {
            font_size,
            rect: Rect::from(
                0.0, 0.0,
                width         + 2.0 * UI_PADDING,
                UI_ELEM_TXT_H + 2.0 * UI_PADDING + UI_ELEM_TXT_H)
        }
    }

//    fn draw_border(&self, p: &mut dyn Painter, width: f64, clr: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64, fill: bool) {
//        let path = &[
//            (x,                      y + UI_BTN_BEVEL),
//            (x + UI_BTN_BEVEL,       y),
//            (x + (w - UI_BTN_BEVEL), y),
//            (x + w,                  y + UI_BTN_BEVEL),
//            (x + w,                  y + (h - UI_BTN_BEVEL)),
//            (x + (w - UI_BTN_BEVEL), y + h),
//            (x + UI_BTN_BEVEL,       y + h),
//            (x,                      y + (h - UI_BTN_BEVEL)),
//        ];
//
//        if fill {
//            p.path_fill(clr, &mut path.iter().copied(), true);
//        } else {
//            p.path_stroke(width, clr, &mut path.iter().copied(), true);
//        }
//    }
//
//    fn draw_divider(&self, p: &mut dyn Painter, _width: f64, color: (f64, f64, f64), x: f64, y: f64) {
//        let (x, y) = (
//            x + (UI_BTN_BORDER_WIDTH / 2.0).round(),
//            y + (UI_BTN_BORDER_WIDTH / 2.0).round(),
//        );
//
//        let w = self.width;
//        let h = UI_ELEM_TXT_H * 2.0 + UI_BTN_BORDER_WIDTH;
//
//        // divider
//        p.path_stroke(
//            UI_BTN_BORDER2_WIDTH,
//            color,
//            &mut [
//                (x,     y + (h / 2.0).round()),
//                (x + w, y + (h / 2.0).round()),
//            ].iter().copied(),
//            false);
//    }

}

impl WidgetType for Entry {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let pos       = self.rect.offs(pos.x, pos.y);
        let id        = data.id();
        let highlight = ui.hl_style_for(id);

        ui.define_active_zone(
            ActiveZone::new_input_zone(id, pos));

        let border_color =
            match highlight {
                HLStyle::Hover(_) => { UI_BTN_TXT_HOVER_CLR },
                _ => UI_ENTRY_BORDER_CLR,
            };

        p.rect_fill(border_color, pos.x, pos.y, pos.w, pos.h);
        let pos = pos.shrink(UI_BORDER_WIDTH, UI_BORDER_WIDTH);
        p.rect_fill(UI_TAB_BG_CLR, pos.x, pos.y, pos.w, pos.h);

        let pos = pos.shrink(UI_SAFETY_PAD, UI_SAFETY_PAD);

        data.with(|data: &mut EntryData| {
            p.label(self.font_size, -1, UI_LBL_TXT_CLR,
                pos.x, pos.y, pos.w, UI_ELEM_TXT_H, &data.label);

            if let Some(atom) = ui.atoms().get(id) {
                if let Some(s) = atom.str_ref() {
                    use std::io::Write;
                    let pos = pos.offs(0.0, UI_ELEM_TXT_H);
                    let mut buf : [u8; 128] = [0_u8; 128];
                    let mut bw = std::io::BufWriter::new(&mut buf[..]);
                    if ui.is_input_value_for(id) {
                        write!(bw, "{}|", s).expect("write ok");
                    } else {
                        write!(bw, "{}", s).expect("write ok");
                    }

                    p.label(self.font_size, -1, UI_LBL_TXT_CLR,
                        pos.x, pos.y, pos.w, UI_ELEM_TXT_H,
                        &std::str::from_utf8(bw.buffer()).unwrap());
                }
            }
            // TODO:
            // - make sure to draw a hover highlight border!
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
