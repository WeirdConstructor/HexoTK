// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::{EvPayload, Event, InputEvent, MButton, Style, Widget};

use crate::style::*;

use crate::painter::*;
use crate::rect::*;

use std::cell::RefCell;
use std::rc::Rc;

pub const UI_CON_BORDER_CLR: (f32, f32, f32) = UI_ACCENT_CLR;
pub const UI_CON_HOV_CLR: (f32, f32, f32) = UI_HLIGHT_CLR;
pub const UI_CON_BORDER_W: f32 = 2.0;

#[derive(Debug, Clone)]
pub struct ConnectorData {
    connection: Option<(usize, usize)>,
    items_left: Vec<(String, bool)>,
    items_right: Vec<(String, bool)>,
    generation: u64,
}

impl ConnectorData {
    pub fn new() -> Self {
        Self { connection: None, items_left: vec![], items_right: vec![], generation: 0 }
    }

    pub fn clear(&mut self) {
        self.connection = None;
        self.items_left.clear();
        self.items_right.clear();
        self.generation += 1;
    }

    pub fn add_input(&mut self, lbl: String, active: bool) {
        self.items_right.push((lbl, active));
        self.generation += 1;
    }
    pub fn add_output(&mut self, lbl: String, active: bool) {
        self.items_left.push((lbl, active));
        self.generation += 1;
    }

    pub fn set_connection(&mut self, i: usize, o: usize) {
        self.connection = Some((i, o));
        self.generation += 1;
    }

    pub fn clear_connection(&mut self) {
        self.connection = None;
        self.generation += 1;
    }

    pub fn get_connection(&mut self) -> Option<(usize, usize)> {
        self.connection
    }
}

pub struct Connector {
    data: Rc<RefCell<ConnectorData>>,

    yrow: f32,
    hover_idx: Option<(bool, usize)>,
    drag_src_idx: Option<(bool, usize)>,
    drag: bool,

    mouse_pos: (f32, f32),
    zones: Vec<(Rect, (bool, usize))>,
    debug_lbls: Vec<(&'static str, &'static str)>,
}

impl Connector {
    pub fn new(data: Rc<RefCell<ConnectorData>>) -> Self {
        Self {
            data,

            yrow: 0.0,
            hover_idx: None,
            drag_src_idx: None,
            drag: false,

            mouse_pos: (0.0, 0.0),
            zones: vec![],
            debug_lbls: vec![
                ("input_0", "output_0"),
                ("input_1", "output_1"),
                ("input_2", "output_2"),
                ("input_3", "output_3"),
                ("input_4", "output_4"),
                ("input_5", "output_5"),
                ("input_6", "output_6"),
                ("input_7", "output_7"),
                ("input_8", "output_8"),
                ("input_9", "output_9"),
                ("input_10", "output_10"),
                ("input_11", "output_11"),
                ("input_12", "output_12"),
                ("input_13", "output_13"),
                ("input_14", "output_14"),
                ("input_15", "output_15"),
                ("input_16", "output_16"),
                ("input_17", "output_17"),
                ("input_18", "output_18"),
                ("input_19", "output_19"),
            ],
        }
    }

    fn xy2pos(&self, x: f32, y: f32) -> Option<(bool, usize)> {
        for z in &self.zones {
            if z.0.is_inside(x, y) {
                return Some(z.1);
            }
        }

        None
    }

    fn get_current_con(&self) -> Option<(bool, (usize, usize))> {
        let data = self.data.borrow();

        let (a_inp, a) = if let Some((inputs, row)) = self.drag_src_idx {
            (inputs, row)
        } else {
            return data.connection.map(|con| (false, con));
        };

        let (b_inp, b) = if let Some((inputs, row)) = self.hover_idx {
            (inputs, row)
        } else {
            return data.connection.map(|con| (false, con));
        };

        if a_inp == b_inp {
            if a_inp {
                if data.items_left.len() == 1 {
                    return Some((true, (0, a)));
                }
            } else {
                if data.items_right.len() == 1 {
                    return Some((true, (a, 0)));
                }
            }
            return data.connection.map(|con| (false, con));
        }

        let (a, b) = if b_inp { (a, b) } else { (b, a) };

        if !data.items_left.get(a).map(|x| x.1).unwrap_or(false) {
            return data.connection.map(|con| (false, con));
        }

        if !data.items_right.get(b).map(|x| x.1).unwrap_or(false) {
            return data.connection.map(|con| (false, con));
        }

        Some((true, (a, b)))
    }
}

impl Connector {
    pub fn handle(&mut self, w: &Widget, event: &InputEvent, out_events: &mut Vec<(usize, Event)>) {
        match event {
            InputEvent::MouseButtonPressed(MButton::Left) => {
                if !w.is_hovered() {
                    return;
                }

                let (x, y) = self.mouse_pos;
                self.drag = true;
                self.drag_src_idx = self.xy2pos(x, y);

                if let Some((inputs, _)) = self.drag_src_idx {
                    if inputs {
                        if self.data.borrow().items_left.len() == 1 {
                            self.drag_src_idx = Some((false, 0));
                        }
                    } else {
                        if self.data.borrow().items_right.len() == 1 {
                            self.drag_src_idx = Some((true, 0));
                        }
                    }
                }

                w.activate();
                w.emit_redraw_required();
            }
            InputEvent::MouseButtonReleased(MButton::Left) => {
                if !w.is_active() {
                    return;
                }

                if let Some((_drag, con)) = self.get_current_con() {
                    self.data.borrow_mut().connection = Some(con);
                } else {
                    self.data.borrow_mut().connection = None;
                }

                out_events.push((
                    w.id(),
                    Event {
                        name: "change".to_string(),
                        data: EvPayload::SetConnection(self.data.borrow().connection),
                    },
                ));

                self.drag = false;
                self.drag_src_idx = None;

                w.deactivate();
                w.emit_redraw_required();
            }
            InputEvent::MousePosition(x, y) => {
                self.mouse_pos = (*x, *y);

                let old_hover = self.hover_idx;
                self.hover_idx = self.xy2pos(*x, *y);

                if old_hover != self.hover_idx {
                    if let Some((inputs, idx)) = self.hover_idx {
                        out_events.push((
                            w.id(),
                            Event {
                                name: "connection_hover".to_string(),
                                data: EvPayload::ConnectionHover { is_input: inputs, index: idx },
                            },
                        ));
                    }

                    w.emit_redraw_required();
                }
            }
            _ => {}
        }
    }

    pub fn draw(&mut self, w: &Widget, style: &Style, pos: Rect, real_pos: Rect, p: &mut Painter) {
        let mut dbg = w.debug_tag();
        dbg.set_offs((real_pos.x - pos.x, real_pos.y - pos.y));

        let data = self.data.borrow();

        self.zones.clear();
        let pos = pos.floor();

        let row_h = data.items_left.len().max(data.items_right.len());

        // XXX: - 1.0 on height and width for some extra margin so that the
        //      rectangles for the ports are not clipped.
        let yrow = (pos.h / (row_h as f32)).floor();
        let xcol = (pos.w / 3.0).floor();

        self.yrow = yrow;

        let pos = Rect { x: pos.x, y: pos.y, w: xcol * 3.0, h: yrow * (row_h as f32) };

        let does_hover_this_widget = w.is_hovered();

        let btn_rect = Rect::from(0.0, 0.0, xcol, yrow);
        for row in 0..row_h {
            let yo = row as f32 * yrow;
            let txt_pad = 2.0 * UI_CON_BORDER_W;
            let txt_w = xcol - 2.0 * txt_pad;

            if let Some((lbl, active)) = data.items_left.get(row) {
                p.rect_stroke_r(
                    UI_CON_BORDER_W,
                    UI_CON_BORDER_CLR,
                    btn_rect.offs(pos.x, pos.y + yo),
                );
                self.zones.push((btn_rect.offs(real_pos.x, real_pos.y + yo), (false, row)));

                let fs = calc_font_size_from_text(p, &lbl, style.font_size, txt_w);
                p.label(
                    fs,
                    -1,
                    if *active { UI_PRIM_CLR } else { UI_INACTIVE_CLR },
                    pos.x + txt_pad,
                    pos.y + yo,
                    txt_w,
                    yrow,
                    &lbl,
                    dbg.source(self.debug_lbls.get(row).unwrap_or(&("input_", "output_")).0),
                );
            }

            if let Some((lbl, active)) = data.items_right.get(row) {
                p.rect_stroke_r(
                    UI_CON_BORDER_W,
                    UI_CON_BORDER_CLR,
                    btn_rect.offs(pos.x + 2.0 * xcol - 1.0, pos.y + yo),
                );
                self.zones.push((
                    btn_rect.offs(real_pos.x + 2.0 * xcol - 1.0, real_pos.y + yo),
                    (true, row),
                ));

                let fs = calc_font_size_from_text(p, &lbl, style.font_size, txt_w);
                p.label(
                    fs,
                    1,
                    if *active { UI_PRIM_CLR } else { UI_INACTIVE_CLR },
                    pos.x + txt_pad + 2.0 * xcol - UI_CON_BORDER_W,
                    pos.y + yo,
                    txt_w,
                    yrow,
                    &lbl,
                    dbg.source(self.debug_lbls.get(row).unwrap_or(&("input_", "output_")).1),
                );
            }
        }

        if let Some((inputs, row)) = self.hover_idx {
            let items = if inputs { &data.items_right } else { &data.items_left };

            if let Some((_lbl, active)) = items.get(row) {
                if *active {
                    let xo = if inputs { xcol * 2.0 - 1.0 } else { 0.0 };
                    let yo = row as f32 * yrow;

                    if does_hover_this_widget {
                        p.rect_stroke_r(
                            UI_CON_BORDER_W,
                            UI_CON_HOV_CLR,
                            btn_rect.offs(pos.x + xo, pos.y + yo),
                        );
                    }
                }
            }
        }

        if let Some((inputs, row)) = self.drag_src_idx {
            let xo = if inputs { xcol * 2.0 - 1.0 } else { 0.0 };
            let yo = row as f32 * yrow;

            if self.drag {
                p.rect_stroke_r(
                    UI_CON_BORDER_W,
                    UI_SELECT_CLR,
                    btn_rect.offs(pos.x + xo, pos.y + yo),
                );
            }
        }

        if let Some((drag, (a, b))) = self.get_current_con() {
            let ay = a as f32 * yrow;
            let by = b as f32 * yrow;

            p.path_stroke(
                4.0,
                if drag { UI_CON_HOV_CLR } else { UI_PRIM_CLR },
                &mut [
                    (xcol, ay + yrow * 0.5),
                    (xcol + xcol * 0.25, ay + yrow * 0.5),
                    (2.0 * xcol - xcol * 0.25, by + yrow * 0.5),
                    (2.0 * xcol - UI_CON_BORDER_W, by + yrow * 0.5),
                ]
                .iter()
                .copied()
                .map(|(x, y)| ((pos.x + x).floor(), (pos.y + y).floor())),
                false,
            );
        }
    }

    pub fn get_generation(&self) -> u64 {
        self.data.borrow().generation
    }
}
