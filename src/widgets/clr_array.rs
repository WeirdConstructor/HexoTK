// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::constants::*;
use super::*;
use super::util::*;

#[derive(Debug)]
pub struct ClrArray {
    width:      f64,
    height:     f64,
}

#[derive(Debug)]
pub struct ClrArrayData {
}

#[allow(clippy::new_ret_no_self)]
impl ClrArrayData {
    pub fn new() -> Box<dyn std::any::Any> {
        Box::new(Self {
        })
    }

//    pub fn set_cv(&self, ui: &mut dyn WidgetUI, id: AtomId,
//                  x: f64, y: f64, samples: usize)
//    {
//        let delta = (self.active_area.h - y) / self.active_area.h;
//        let xoffs = (x / self.x_delta).max(0.0);
//        let idx   = xoffs.floor().min(samples as f64 - 1.0) as usize;
//
//        if let Some(Some(new)) =
//            ui.atoms()
//              .get(id)
//              .map(|atom|
//                  atom.set_v_idx_micro(idx, delta.clamp(0.0, 1.0) as f32))
//        {
//            ui.atoms_mut().set(id, new);
//        }
//    }
}


impl ClrArray {
    pub fn new(width: f64, height: f64) -> Self {
//        let inner_w = width - 2.0 * UI_GRPH_BORDER;
//        let xd      = (inner_w / (samples as f64)).floor();
//        let width   = xd * (samples as f64) + 2.0 * UI_GRPH_BORDER;

        Self {
            width,
            height,
        }
    }
}

impl WidgetType for ClrArray {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData,
            p: &mut dyn Painter, pos: Rect)
    {
        let id = data.id();

        data.with(|data: &mut ClrArrayData| {
            let xd = pos.w / 9.0;
            let yd = pos.h / 2.0;

            // XXX: Offset by 1, to wrap the magenta at index 0 to the
            //      end of the selector!
            let mut clr : u8 = 1;

            for row in 0..2 {
                for i in 0..9 {
                    let pos = Rect {
                        x: pos.x + (i   as f64) * xd,
                        y: pos.y + (row as f64) * yd,
                        w: xd,
                        h: yd,
                    };

                    let highlight = ui.hl_style_for(id, Some(clr as usize));

                    let selected =
                        ui.atoms().get(id)
                          .cloned()
                          // XXX: 1000 is a color that won't ever exist... very probably
                          .unwrap_or_else(|| Atom::setting(1000));

                    let border_color =
                        if clr as i64 == selected.i() {
                            UI_SELECT_CLR
                        } else {
                            match highlight {
                                HLStyle::Hover(_) => UI_HLIGHT_CLR,
                                _                 => UI_ACCENT_DARK_CLR,
                            }
                        };

                    p.rect_fill(
                        border_color,
                        pos.x, pos.y, pos.w, pos.h);

                    ui.define_active_zone(
                        ActiveZone::new_indexed_click_zone(
                            id, pos, clr as usize)
                        .dbgid(DBGID_CLRARRAY));

                    let pos = pos.shrink(2.0, 2.0);

                    p.rect_fill(
                        hex_color_idx2clr(clr),
                        pos.x, pos.y, pos.w, pos.h);

                    clr = (clr + 1) % 18;
                }
            }
        });
    }

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _avail: (f64, f64)) -> (f64, f64) {
        (self.width  + UI_GRPH_BORDER * 2.0,
         self.height + UI_GRPH_BORDER * 2.0)
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        match ev {
            UIEvent::Click { id, index, .. } => {
                if *id == data.id() {
                    data.with(|data: &mut ClrArrayData| {
                        //d// println!("SELECT COLOR: {}", *index);
                        ui.atoms_mut().set(*id, Atom::setting(*index as i64));
                        ui.queue_redraw();
                    });
                }
            },
            _ => {},
        }
    }
}
