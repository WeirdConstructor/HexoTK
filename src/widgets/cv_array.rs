// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use crate::constants::*;
use super::*;
use super::util::*;

#[derive(Debug)]
pub struct CvArray {
    width:      f64,
    height:     f64,
    font_size:  f64,
    samples:    usize,
    null_v:     Vec<f32>,
}

#[derive(Debug)]
pub struct CvArrayData {
    name:           String,
    x_delta:        f64,
    active_area:    Rect,
}

impl CvArrayData {
    pub fn new(name: &str) -> Box<dyn std::any::Any> {
        Box::new(Self {
            name:       String::from(name),
            x_delta:    10.0,
            active_area: Rect::from(0.0, 0.0, 10.0, 10.0),
        })
    }
}


impl CvArray {
    pub fn new(samples: usize, width: f64, height: f64, font_size: f64) -> Self {
        Self {
            null_v: vec![0.0; samples],
            width,
            height,
            font_size,
            samples,
        }
    }
}

impl WidgetType for CvArray {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let id = data.id();
        let highlight = ui.hl_style_for(id, None);

        let border_color =
            match highlight {
                HLStyle::Hover(_) => UI_GRPH_BORDER_HOVER_CLR,
                _                 => UI_GRPH_BORDER_CLR,
            };

        let pos =
            rect_border(p, UI_GRPH_BORDER, border_color, UI_GRPH_BG, pos);

        let w = self.width;
        let h = self.height;

        ui.define_active_zone(ActiveZone::new_indexed_drag_zone(id, pos, 4));

        let mut label_color = UI_BTN_TXT_CLR;

        data.with(|data: &mut CvArrayData| {
            let zone_rect = Rect::from_tpl((0.0, 0.0, w, h));

            let xd = pos.w / (self.samples as f64);

            data.active_area = pos;
            data.x_delta     = xd.max(1.0);

            for i in 0..self.samples {
            }

            if let Some(data) = ui.atoms().get(id).unwrap().v_ref() {
                for i in 0..self.samples {
                    let v = (data[i] as f64).clamp(0.0, 1.0);
                    let h = pos.h * v;
                    //d// println!("[{:2}] V={:8.5} H={:8.5}", i, v, h);
                    if h > 0.0 {
                        p.rect_fill(
                            UI_GRPH_LINE_CLR,
                            pos.x + i as f64 * xd,
                            pos.y + pos.h * (1.0 - v),
                            xd,
                            h);
                    }
                }
            }
        });
    }

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _avail: (f64, f64)) -> (f64, f64) {
        (self.width  + UI_GRPH_BORDER * 2.0 + UI_SAFETY_PAD,
         self.height + UI_GRPH_BORDER * 2.0 + UI_ELEM_TXT_H + UI_SAFETY_PAD)
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        match ev {
            UIEvent::Drag { id, index, x, y, start_x, start_y, .. } => {
                if *id == data.id() {
                    // TODO: Set position!
                    let delta = y - start_y;
                    data.with(|data: &mut CvArrayData| {
                        let xoffs = x / data.x_delta;
                        println!("**** XOFFS={:6.2}", xoffs);
                    });
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
