// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

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
    binary:     bool,
}

#[derive(Debug)]
pub struct CvArrayData {
    name:           String,
    x_delta:        f64,
    active_area:    Rect,
}

#[allow(clippy::new_ret_no_self)]
impl CvArrayData {
    pub fn new(name: &str) -> Box<dyn std::any::Any> {
        Box::new(Self {
            name:       String::from(name),
            x_delta:    10.0,
            active_area: Rect::from(0.0, 0.0, 10.0, 10.0),
        })
    }

    pub fn set_cv_binary(
        &self, ui: &mut dyn WidgetUI, id: AtomId, index: usize)
    {
        if index >= 64 { return; }

        let mask : i64 =
            ui.atoms().get(id).map(|atom| {
                let mut i = atom.i();
                i ^= 0x1 << index;
                i
            }).unwrap_or_else(|| 0x1 << index);
        ui.atoms_mut().set(id, Atom::setting(mask));
    }

    pub fn set_cv(&self, ui: &mut dyn WidgetUI, id: AtomId,
                  x: f64, y: f64, samples: usize)
    {
        let delta = (self.active_area.h - y) / self.active_area.h;
        let xoffs = (x / self.x_delta).max(0.0);
        let idx   = xoffs.floor().min(samples as f64 - 1.0) as usize;

        if let Some(Some(new)) =
            ui.atoms()
              .get(id)
              .map(|atom|
                  atom.set_v_idx_micro(idx, delta.clamp(0.0, 1.0) as f32))
        {
            ui.atoms_mut().set(id, new);
        }
    }
}


impl CvArray {
    pub fn new(samples: usize, width: f64, height: f64, font_size: f64, binary: bool) -> Self {
        let inner_w = width - 2.0 * UI_GRPH_BORDER;
        let xd      = (inner_w / (samples as f64)).floor();
        let width   = xd * (samples as f64) + 2.0 * UI_GRPH_BORDER;

        Self {
            null_v: vec![0.0; samples],
            binary,
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

        if !self.binary {
            ui.define_active_zone(
                ActiveZone::new_indexed_drag_zone(id, pos, 4)
                .dbgid(DBGID_CVARRAY_DRAG));
        }

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
                &data.name, DBGID_CVARRAY_NAME);

            let pos = pos.crop_bottom(UI_ELEM_TXT_H);

            let xd = pos.w / (self.samples as f64);
            let xd = xd.floor();

            data.active_area = pos;
            data.x_delta     = xd.max(1.0);

            let phase =
                if let Some(phase) = ui.atoms().get_phase_value(id) {
                    phase as f64
                } else { 0.0 };

            let mut x = 0.0;
            let phase_delta = 1.0 / (self.samples as f64);
            let mut xphase = 0.0;

            for i in 0..self.samples {
                let v = {
                    let data = ui.atoms().get(id);
                    if self.binary {
                        data.map(|atom| {
                                if atom.i() & (0x1 << i) > 0 { 1.0 }
                                else                         { 0.0 }
                            }).unwrap_or(0.0)
                    } else if let Some(Some(data)) = data.map(|atom| atom.v_ref()) {
                        (data[i] as f64).clamp(0.0, 1.0)
                    } else { 0.0 }
                };

                let hover_highlight =
                    if self.binary {
                        let zone_pos = Rect {
                            x: pos.x + x,
                            y: pos.y,
                            w: xd.ceil(),
                            h: pos.h,
                        };

                        ui.define_active_zone(
                            ActiveZone::new_indexed_click_zone(
                                id, zone_pos, i)
                            .dbgid(DBGID_CVARRAY_CLICK));

                        !matches!(ui.hl_style_for(id, Some(i)), HLStyle::None)
                    } else {
                        false
                    };

                let h = pos.h * (1.0 - v);

                let (color, mut phase_bg_color) =
                    if phase >= xphase && phase < (xphase + phase_delta) {
                        (UI_GRPH_PHASE_CLR, Some(UI_GRPH_PHASE_BG_CLR))
                    } else {
                        (UI_GRPH_LINE_CLR, None)
                    };

                if hover_highlight {
                    phase_bg_color = Some(UI_GRPH_PHASE_BG_CLR);
                }

                xphase += phase_delta;

                //d// println!("h={:6.2} pos.h={:6.2}", h, pos.h);

                // draw the last a little bit wider to prevent the gap
                let w =
                    if i == (self.samples - 1) {
                        xd + 0.5
                    } else {
                        xd
                    };

                if let Some(bg_color) = phase_bg_color {
                    p.rect_fill(
                        bg_color,
                        (pos.x + x).ceil() - 0.5,
                        pos.y - 0.5,
                        w,
                        pos.h);
                }

                if pos.h - h > 0.5 {
                    p.rect_fill(
                        color,
                        (pos.x + x).ceil() - 0.5,
                        (pos.y + h) - 0.5,
                        w,
                        pos.h - h + 1.5);
                }

                x += xd;
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
            for _i in 0..(self.samples - 1) {
                p.path_stroke(
                    1.0,
                    UI_GRPH_LINE_CLR,
                    &mut [
                        ((pos.x + x).floor() - 0.5, pos.y.floor()),
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
            UIEvent::Drag { id, x, y, .. } => {
                if *id == data.id() {
                    data.with(|data: &mut CvArrayData| {
                        data.set_cv(ui, *id, *x, *y, self.samples);
                        ui.queue_redraw();
                    });
                }
            },
            UIEvent::Click { id, index, .. } => {
                if *id == data.id() {
                    data.with(|data: &mut CvArrayData| {
                        data.set_cv_binary(ui, *id, *index);
                        ui.queue_redraw();
                    });
                }
            },
            _ => {},
        }
    }
}
