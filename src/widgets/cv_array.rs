// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use crate::constants::*;
use super::*;

#[derive(Debug)]
pub struct CvArray {
    width:      f64,
    height:     f64,
    font_size:  f64,
    samples:    usize,
}

#[derive(Debug)]
pub struct CvArrayData {
    name:    String,
}

impl CvArrayData {
    pub fn new(name: &str) -> Box<dyn std::any::Any> {
        Box::new(Self {
            name: String::from(name),
        })
    }
}


impl CvArray {
    pub fn new(samples: usize, width: f64, height: f64, font_size: f64) -> Self {
        Self {
            width,
            height,
            font_size,
            samples,
        }
    }
}

impl WidgetType for CvArray {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let (x, y) = (pos.x, pos.y);

        let w = self.width;
        let h = self.height;

        let id = data.id();
        let highlight = ui.hl_style_for(id, None);

        ui.define_active_zone(ActiveZone::new_indexed_drag_zone(id, pos, 4));

        let mut label_color = UI_BTN_TXT_CLR;

        let (mut color, border_color, mut bg_color) =
            match highlight {
                HLStyle::Hover(_) => {
                    (UI_BTN_TXT_HOVER_CLR, UI_BTN_TXT_HOVER_CLR, UI_BTN_BG_CLR)
                },
                HLStyle::HoverModTarget => {
                    (UI_BTN_TXT_HLHOVR_CLR, UI_BTN_TXT_HLHOVR_CLR, UI_BTN_BG_CLR)
                },
                HLStyle::AtomClick => {
                    label_color = UI_BTN_BG_CLR;
                    (UI_BTN_BG_CLR, UI_BTN_BORDER2_CLR, UI_BTN_TXT_CLR)
                },
                HLStyle::ModTarget => {
                    (UI_BTN_TXT_HLIGHT_CLR, UI_BTN_TXT_HLIGHT_CLR, UI_BTN_BG_CLR)
                },
                HLStyle::Inactive => {
                    (UI_INACTIVE2_CLR, UI_INACTIVE2_CLR, UI_BTN_BG_CLR)
                },
                _ => (UI_BTN_TXT_CLR, UI_BTN_BORDER2_CLR, UI_BTN_BG_CLR)
            };

        data.with(|data: &mut CvArrayData| {
            let zone_rect = Rect::from_tpl((0.0, 0.0, w, h));
        });
    }

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _avail: (f64, f64)) -> (f64, f64) {
        (self.width  + UI_GRPH_BORDER * 2.0 + UI_SAFETY_PAD,
         self.height + UI_GRPH_BORDER * 2.0 + UI_ELEM_TXT_H + UI_SAFETY_PAD)
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        match ev {
            UIEvent::Drag { id, index, x, y, .. } => {
                if *id == data.id() {
                    // TODO: Set position!
                    println!("DRAG! {:?}", ev);
                }
            },
            UIEvent::Click { id, button, index, x, y, .. } => {
                if *id == data.id() {
                    data.with(|data: &mut CvArrayData| {
                        // TODO: Set position!
                    });

                    ui.queue_redraw();
                }
            },
            _ => {},
        }
    }
}
