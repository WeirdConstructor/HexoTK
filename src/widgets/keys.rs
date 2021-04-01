// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use crate::constants::*;
use super::*;
use super::util::*;

#[derive(Debug)]
pub struct Keys {
    width:      f64,
    height:     f64,
    font_size:  f64,
}

#[derive(Debug)]
pub struct KeysData {
    name:           String,
}

impl KeysData {
    pub fn new(name: &str) -> Box<dyn std::any::Any> {
        Box::new(Self {
            name:       String::from(name),
        })
    }
}


impl Keys {
    pub fn new(width: f64, height: f64, font_size: f64) -> Self {
        Self {
            width,
            height,
            font_size,
        }
    }
}

impl WidgetType for Keys {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData,
            p: &mut dyn Painter, pos: Rect)
    {
        let id        = data.id();
        let highlight = ui.hl_style_for(id, None);

        let border_color =
            match highlight {
                HLStyle::Hover(_) => UI_GRPH_BORDER_HOVER_CLR,
                _                 => UI_GRPH_BORDER_CLR,
            };

        let pos =
            rect_border(p, UI_GRPH_BORDER, border_color, UI_GRPH_BG, pos);

        data.with(|data: &mut KeysData| {
            let lbl_y = pos.y + pos.h - UI_ELEM_TXT_H;
            p.label(
                self.font_size,
                0,
                UI_GRPH_TEXT_CLR,
                pos.x,
                lbl_y,
                pos.w,
                UI_ELEM_TXT_H,
                &data.name);

            let pos = pos.crop_bottom(UI_ELEM_TXT_H);

            let xd = (pos.w / 7.0).floor();

            let xoffs = [
                0.0 * xd,   // white C
                0.5 * xd,   // black C#
                1.0 * xd,   // white D
                1.5 * xd,   // black D#
                2.0 * xd,   // white E
                3.0 * xd,   // white F
                3.5 * xd,   // black F#
                4.0 * xd,   // white G
                4.5 * xd,   // black G#
                5.0 * xd,   // white A
                5.5 * xd,   // black A#
                6.0 * xd,   // white B
            ];

            for i in 0..12 {
                let key =
                    Rect {
                        x: pos.x + xoffs[i],
                        y: pos.y,
                        w: xd,
                        h: pos.h,
                    };

                ui.define_active_zone(
                    ActiveZone::new_indexed_click_zone(
                        id, key, i));

                let hover_key =
                    if let HLStyle::None = ui.hl_style_for(id, Some(i)) {
                        false
                    } else { true }

                p.rect_fill(color, key.x, key.y, key.w, key.h);
            }
        });
    }

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _avail: (f64, f64)) -> (f64, f64) {
        (self.width,
         self.height + UI_GRPH_BORDER * 2.0 + UI_ELEM_TXT_H)
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        match ev {
            UIEvent::Click { id, x, y, index, .. } => {
                if *id == data.id() {
                    data.with(|data: &mut KeysData| {
                        data.set_cv_binary(ui, *id, *index, *x, *y, self.samples);
                        ui.queue_redraw();
                    });
                }
            },
            _ => {},
        }
    }
}
