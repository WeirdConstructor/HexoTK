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
        let inner_w = width - 2.0 * UI_GRPH_BORDER;
        let xd      = (inner_w / (samples as f64)).floor();
        let width   = xd * (samples as f64) + 2.0 * UI_GRPH_BORDER;

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
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData,
            p: &mut dyn Painter, pos: Rect)
    {
        let id = data.id();
        let highlight = ui.hl_style_for(id, None);

        let border_color =
            match highlight {
                HLStyle::Hover(_) => UI_GRPH_BORDER_HOVER_CLR,
                _                 => UI_GRPH_BORDER_CLR,
            };

        let pos =
            rect_border(p, UI_GRPH_BORDER, border_color, UI_GRPH_BG, pos);

        ui.define_active_zone(ActiveZone::new_indexed_drag_zone(id, pos, 4));

        let mut label_color = UI_BTN_TXT_CLR;

        data.with(|data: &mut CvArrayData| {
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

            let xd = pos.w / (self.samples as f64);
            let xd = xd.floor();

            data.active_area = pos;
            data.x_delta     = xd.max(1.0);

            if let Some(data) = ui.atoms().get(id).unwrap().v_ref() {
                let mut x = 0.0;

                for i in 0..self.samples {
                    let v = (data[i] as f64).clamp(0.0, 1.0);
                    let h = pos.h * (1.0 - v);

                    println!("h={:6.2} pos.h={:6.2}", h, pos.h);

                    // draw the last a little bit wider to prevent the gap
                    let w =
                        if i == (self.samples - 1) {
                            (xd + 0.5).floor()
                        } else {
                            xd.ceil()
                        };

                    if pos.h - h > 0.5 {
                        p.rect_fill(
                            UI_GRPH_LINE_CLR,
                            (pos.x + x).ceil() - 0.5,
                            (pos.y + h) - 0.5,
                            w,
                            pos.h - h + 1.5);
                    }

                    x += xd;
                }
            }

            p.path_stroke(
                1.0,
//                UI_GRPH_LINE_CLR,
                UI_ACCENT_CLR,
                &mut [
                    (pos.x,         lbl_y.floor() + 0.5),
                    (pos.x + pos.w, lbl_y.floor() + 0.5),
                ].iter().copied(),
                false);

            let mut x = xd;
            for i in 0..(self.samples - 1) {
                p.path_stroke(
                    1.0,
                    UI_GRPH_LINE_CLR,
                    &mut [
                        ((pos.x + x).floor() - 0.5, pos.y),
                        ((pos.x + x).floor() - 0.5, pos.y + pos.h),
                    ].iter().copied(),
                    false);

                x += xd;
            }
        });
    }

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _avail: (f64, f64)) -> (f64, f64) {
        (self.width,
         self.height + UI_GRPH_BORDER * 2.0 + UI_ELEM_TXT_H)
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        match ev {
            UIEvent::Drag { id, index, x, y, start_x, start_y, .. } => {
                if *id == data.id() {
                    // TODO: Set position!
                    data.with(|data: &mut CvArrayData| {
                        let delta = (data.active_area.h - y) / data.active_area.h;
                        let xoffs = (x / data.x_delta).max(0.0);
                        let idx   = xoffs.floor().min(self.samples as f64 - 1.0) as usize;

                        if let Some(new) =
                            ui.atoms()
                              .get(*id)
                              .unwrap()
                              .set_v_idx_micro(idx, delta.clamp(0.0, 1.0) as f32)
                        {
                            ui.atoms_mut().set(*id, new);
                        }
                    });
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
