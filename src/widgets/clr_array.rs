// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::constants::*;
use super::*;
use super::util::*;

const UNUSED_COLOR_IDX : i64 = 99999;

#[derive(Debug)]
pub struct ClrArray {
}

#[derive(Debug)]
pub struct ClrArrayData {
    with_unset_option:  bool,
}

#[allow(clippy::new_ret_no_self)]
impl ClrArrayData {
    pub fn new(with_unset_option: bool) -> Box<dyn std::any::Any> {
        Box::new(Self {
            with_unset_option,
        })
    }
}

impl ClrArray {
    pub fn new() -> Self {
        Self { }
    }
}

impl WidgetType for ClrArray {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData,
            p: &mut dyn Painter, pos: Rect)
    {
        let id = data.id();

        data.with(|data: &mut ClrArrayData| {
            let xd = (pos.w / 9.0).floor();
            let yd = (pos.h / 2.0).floor();

            let rest_w = pos.w - xd * 9.0;
            let rest_h = pos.h - yd * 2.0;
            let xo = (rest_w / 2.0).round();
            let yo = (rest_h / 2.0).round();

            // XXX: Offset by 1, to wrap the magenta at index 0 to the
            //      end of the selector!
            let mut clr : u8 = 1;

            for row in 0..2 {
                for i in 0..9 {
                    let pos = Rect {
                        x: (xo + pos.x + (i   as f64) * xd).round(),
                        y: (yo + pos.y + (row as f64) * yd).round(),
                        w: xd.round(),
                        h: yd.round(),
                    };

                    let highlight = ui.hl_style_for(id, Some(clr as usize));

                    let selected =
                        ui.atoms().get(id)
                          .cloned()
                          .unwrap_or_else(|| Atom::setting(UNUSED_COLOR_IDX));

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

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, avail: (f64, f64)) -> (f64, f64) {
        avail
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        match ev {
            UIEvent::Click { id, index, .. } => {
                if *id == data.id() {
                    data.with(|data: &mut ClrArrayData| {
                        println!("SELECT THE COLOR: {}", *index);

                        if data.with_unset_option {
                            let mut idx_out = *index as i64;

                            if let Some(a) = ui.atoms().get(*id) {
                                if a.i() == *index as i64 {
                                    idx_out = UNUSED_COLOR_IDX;
                                }
                            }

                            ui.atoms_mut().set(*id, Atom::setting(idx_out));
                        } else {
                            ui.atoms_mut().set(*id, Atom::setting(*index as i64));
                        }
                        ui.queue_redraw();
                    });
                }
            },
            _ => {},
        }
    }
}
