// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use crate::constants::*;
use super::*;

#[derive(Debug)]
pub struct Container {
}

impl Container {
    pub fn new() -> Self {
        Self { }
    }
}

pub struct ContainerData {
    rows: Vec<Vec<WidgetData>>,
}

impl ContainerData {
    pub fn new() -> Self {
        Self {
            rows: vec![],
        }
    }

    pub fn new_row(&mut self) -> &mut Self {
        self.rows.push(vec![]);
        self
    }

    pub fn add(&mut self, wtype: usize, id: ParamID, pos: UIPos, data: Box<dyn std::any::Any>) -> &mut Self {
        if self.rows.len() > 0 {
            let last_idx = self.rows.len() - 1;
            self.rows[last_idx].push(WidgetData::new(wtype, id, pos, data));
        }

        self
    }
}

impl WidgetType for Container {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let cont_pos = pos;

        data.with(|data: &mut ContainerData| {
            let mut row_offs = 0;
            for cols in data.rows.iter_mut() {
                let mut col_offs = 0;

                let mut min_row_offs = 255;

                for data in cols.iter_mut() {
                    let pos = data.pos();

                    let (widget_rect, ro, co) =
                        cont_pos.calc_widget_rect(row_offs, col_offs, pos);
    //                            println!("CALC ELEM POS={:?} => row={},col={} => ro={},co={}",
    //                                    pos,
    //                                    row_offs, col_offs,
    //                                    ro, co);

                    col_offs = co;

                    if ro < min_row_offs { min_row_offs = ro; }

                    let size = ui.widget_size(data, (widget_rect.w, widget_rect.h));

                    let mut xe = widget_rect.x;
                    let mut ye = widget_rect.y;
                    let align = pos.alignment();

                    match align.0 {
                        1 => { xe += widget_rect.w - size.0; },
                        0 => { xe += ((widget_rect.w - size.0) / 2.0).round(); },
                        _ => { /* left align is a nop */ },
                    }

                    match align.1 {
                        1 => { ye += widget_rect.h - size.1; },
                        0 => { ye += ((widget_rect.h - size.1) / 2.0).round(); },
                        _ => { /* top align is a nop */ },
                    }

                    ui.draw_widget(data, p, Rect { x: xe, y: ye, w: size.0, h: size.1 });
                }

                row_offs = min_row_offs;
            }
        });
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        data.with(|data: &mut ContainerData| {
            for cols in data.rows.iter_mut() {
                for data in cols.iter_mut() {
                    ui.propagate_event(data, ev);
                }
            }
        });
    }
}
