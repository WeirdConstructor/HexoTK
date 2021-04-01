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
            let xd_pad_for_center = ((pos.w - xd * 7.0) * 0.5).floor();
            let pos = pos.shrink(xd_pad_for_center, 0.0);

            let xoffs_w = [
                0.0 * xd,   // white C
                1.0 * xd,   // white D
                2.0 * xd,   // white E
                3.0 * xd,   // white F
                4.0 * xd,   // white G
                5.0 * xd,   // white A
                6.0 * xd,   // white B
            ];

            let xoffs_b = [
                1.0 * xd,   // black C#
                2.0 * xd,   // black D#
                4.0 * xd,   // black F#
                5.0 * xd,   // black G#
                6.0 * xd,   // black A#
            ];

            let phase =
                if let Some(phase) = ui.atoms().get_phase_value(id) {
                    phase as f64
                } else { 0.0 };

            fn draw_key(p: &mut dyn Painter, ui: &mut dyn WidgetUI,
                        id: AtomId, key: Rect, index: usize)
            {
                ui.define_active_zone(
                    ActiveZone::new_indexed_click_zone(
                        id, key, index));

                let (bg_color, line_color) =
                    if let HLStyle::None = ui.hl_style_for(id, Some(index)) {
                        (UI_GRPH_BG, UI_GRPH_LINE_CLR)
                    } else {
                        (UI_GRPH_PHASE_BG_CLR, UI_GRPH_BG)
                    };

                p.rect_fill(line_color, key.x, key.y, key.w, key.h);
                let k2 = key.shrink(1.0, 1.0);
                p.rect_fill(bg_color, k2.x, k2.y, k2.w, k2.h);
            }

            for i in 0..xoffs_w.len() {
                let key =
                    Rect {
                        x: pos.x + xoffs_w[i],
                        y: pos.y,
                        w: xd,
                        h: pos.h,
                    };

                draw_key(p, ui, id, key, i);
            }

            let black_width = xd * 0.75;

            for i in 0..xoffs_b.len() {
                let key =
                    Rect {
                        x: pos.x + xoffs_b[i] - black_width * 0.5,
                        y: pos.y,
                        w: black_width,
                        h: pos.h * 0.5,
                    };

                draw_key(p, ui, id, key, i + 7);
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
                        println!("CLICK IDX={}", index);
                        ui.queue_redraw();
                    });
                }
            },
            _ => {},
        }
    }
}
