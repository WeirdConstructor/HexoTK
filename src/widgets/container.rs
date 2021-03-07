// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use crate::constants::*;
use super::*;

#[derive(Debug)]
pub struct Container {
    debug: bool,
}

impl Container {
    pub fn new() -> Self {
        Self { debug: false }
    }
}

pub struct ContainerData {
    rows:   Vec<Vec<WidgetData>>,
    border: bool,
    contrast_border: bool,
    title:  Option<String>,
}

impl ContainerData {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            rows:               vec![],
            border:             false,
            contrast_border:    false,
            title:              None,
        })
    }

    pub fn new_row(&mut self) -> &mut Self {
        self.rows.push(vec![]);
        self
    }

    pub fn title(&mut self, title: &str) -> &mut Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn contrast_border(&mut self) -> &mut Self {
        self.contrast_border = true;
        self.border()
    }

    pub fn border(&mut self) -> &mut Self {
        self.border = true;
        self
    }

    pub fn add_direct(&mut self, widget_data: WidgetData) -> &mut Self {
        if self.rows.len() > 0 {
            let last_idx = self.rows.len() - 1;
            self.rows[last_idx].push(widget_data);
        }

        self
    }

    pub fn add(&mut self, wtype: Rc<dyn WidgetType>, id: ParamID, pos: UIPos, data: Box<dyn std::any::Any>) -> &mut Self {
        self.add_direct(WidgetData::new(wtype, id, pos, data));

        self
    }
}

impl WidgetType for Container {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {

        data.with(|data: &mut ContainerData| {
            let inner_pos =
                if data.border {
                    let pos =
                        if data.contrast_border {
                            p.rect_fill(UI_BG_CLR, pos.x, pos.y, pos.w, pos.h);
                            pos.shrink(UI_BORDER_WIDTH, UI_BORDER_WIDTH)
                        } else { pos };

                    let new_inner =
                        pos.shrink(UI_BORDER_WIDTH, UI_BORDER_WIDTH);

                    p.rect_fill(UI_BORDER_CLR, pos.x, pos.y, pos.w, pos.h);
                    p.rect_fill(
                        UI_BG_CLR,
                        new_inner.x, new_inner.y, new_inner.w, new_inner.h);

                    new_inner
                } else { pos };

            let cont_pos =
                if let Some(ref title) = &data.title {
                    let new_inner = inner_pos.crop_top(UI_ELEM_TXT_H);
                    p.rect_fill(
                        UI_LBL_BG_CLR,
                        inner_pos.x, inner_pos.y, new_inner.w, UI_ELEM_TXT_H);
                    p.label(
                        UI_CONT_FONT_SIZE, 0, UI_CONT_FONT_CLR,
                        inner_pos.x, inner_pos.y, new_inner.w, UI_ELEM_TXT_H,
                        title);
                    new_inner
                } else { pos };

            let mut row_offs = 0;
            for cols in data.rows.iter_mut() {
                let mut col_offs = 0;

                let mut min_row_offs = 255;

                for data in cols.iter_mut() {
                    let pos = data.pos();

                    let (widget_rect, ro, co) =
                        cont_pos.calc_widget_rect(row_offs, col_offs, pos);

                    col_offs = co;

                    if ro < min_row_offs { min_row_offs = ro; }

                    let mut xe = widget_rect.x;
                    let mut ye = widget_rect.y;
                    let align = pos.alignment();

                    let size = data.size(ui, (widget_rect.w, widget_rect.h));

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

                    let xe = xe.floor();
                    let ye = ye.floor();

                    data.draw(ui, p, Rect { x: xe, y: ye, w: size.0, h: size.1 });

                    if self.debug {
                        p.rect_stroke(1.0, (0.0, 1.0, 0.0),
                            xe - 0.5, ye - 0.5,
                            size.0 - 1.0, size.1 - 1.0);
                        p.rect_stroke(1.0, (1.0, 0.0, 0.0),
                            widget_rect.x + 0.5,
                            widget_rect.y + 0.5,
                            widget_rect.w - 1.0,
                            widget_rect.h - 1.0);
                    }

                }

                row_offs = min_row_offs;
            }
        });
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        data.with(|data: &mut ContainerData| {
            for cols in data.rows.iter_mut() {
                for data in cols.iter_mut() {
                    data.event(ui, ev);
                }
            }
        });
    }
}
