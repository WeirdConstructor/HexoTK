// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use crate::constants::*;
use super::*;

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct TextEntryRef {
    text:  Rc<RefCell<(usize, String)>>,
}

#[derive(Debug)]
pub struct Entry {
    width:      f64,
    font_size:  f64,
}

#[derive(Debug)]
pub struct EntryData {
    label:     String,
    txt_ref:   TextEntryRef,
    txt_width: usize,
}

impl EntryData {
    pub fn new(label: &str, txt_width: usize, txt_ref: TextEntryRef) -> Box<dyn std::any::Any> {
        Box::new(Self {
            label: label.to_string(),
            txt_width,
            txt_ref,
        })
    }
}


impl Entry {
    pub fn new(width: f64, font_size: f64) -> Self {
        Self {
            width,
            font_size,
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
        let (x, y) = (pos.x, pos.y);

        let (xo, yo) = (
            x + UI_PADDING,
            y + UI_PADDING,
        );

        let w = self.width;
        let h = UI_ELEM_TXT_H * 2.0 + UI_ENTRY_BORDER_WIDTH;

        let id = data.id();
        let highlight = ui.hl_style_for(id);

        let color =
            match highlight {
                HLStyle::Hover(_) => {
//                    self.draw_border(
//                        p, UI_BTN_BORDER2_WIDTH, UI_BTN_TXT_HOVER_CLR,
//                        xo - (UI_BTN_BORDER2_WIDTH * 0.5).round(),
//                        yo - (UI_BTN_BORDER2_WIDTH * 0.5).round(),
//                        w + UI_BTN_BORDER2_WIDTH,
//                        h + UI_BTN_BORDER2_WIDTH, false);
                    UI_BTN_TXT_HOVER_CLR
                },
                _ => UI_BTN_TXT_CLR,
            };

        data.with(|data: &mut ButtonData| {
            // Draw rect with LBL_BG
            // draw label with ":"
            // on new line draw the entered text (width is provided by the
            //    TextData!)
            // draw a line benath it.
            // Place an active zone for text entering


            // TODO:
            // - make sure to draw a hover highlight border!
        });
    }

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _avail: (f64, f64)) -> (f64, f64) {
        (self.width    + 2.0 * UI_PADDING,
         UI_ELEM_TXT_H + 2.0 * UI_PADDING + UI_ELEM_TXT_H)
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        match ev {
            UIEvent::TextChanged { id, val } => {
                if *id == data.id() {
                    println!("ENTERED TEXT!");
                    ui.queue_redraw();
                }
            },
            _ => {},
        }
    }
}
