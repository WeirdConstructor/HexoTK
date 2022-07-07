// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::{Widget, InputEvent, Event, MButton, Style};
use keyboard_types::Key;

use super::ModifierTracker;

use crate::style::*;

use crate::painter::*;
use crate::rect::*;

pub use hexodsp::dsp::tracker::UIPatternModel;
pub use hexodsp::dsp::tracker::PatternData;

use std::sync::{Arc, Mutex};

pub const UI_TRK_ROW_HEIGHT        : f32 = 14.0;
pub const UI_TRK_COL_WIDTH         : f32 = 38.0;
pub const UI_TRK_FONT_SIZE         : f32 = 12.0;
pub const UI_TRK_COL_DIV_PAD       : f32 = 3.0;
//pub const UI_TRK_BG_CLR            : (f32, f32, f32) = UI_LBL_BG_CLR;
pub const UI_TRK_BG_ALT_CLR        : (f32, f32, f32) = UI_LBL_BG_ALT_CLR;
pub const UI_TRK_COL_DIV_CLR       : (f32, f32, f32) = UI_PRIM2_CLR;
//pub const UI_TRK_BORDER_CLR        : (f32, f32, f32) = UI_ACCENT_CLR;
//pub const UI_TRK_BORDER_HOVER_CLR  : (f32, f32, f32) = UI_HLIGHT_CLR;
//pub const UI_TRK_BORDER_EDIT_CLR   : (f32, f32, f32) = UI_SELECT_CLR;
//pub const UI_TRK_BORDER_INACT_CLR  : (f32, f32, f32) = UI_INACTIVE_CLR;
pub const UI_TRK_TEXT_CLR          : (f32, f32, f32) = UI_TXT_CLR;
pub const UI_TRK_CURSOR_BG_CLR     : (f32, f32, f32) = UI_PRIM2_CLR;
pub const UI_TRK_CURSOR_BG_HOV_CLR : (f32, f32, f32) = UI_PRIM_CLR;
pub const UI_TRK_CURSOR_BG_SEL_CLR : (f32, f32, f32) = UI_SELECT_CLR;
pub const UI_TRK_CURSOR_FG_CLR     : (f32, f32, f32) = UI_LBL_BG_CLR;
pub const UI_TRK_PHASEROW_BG_CLR   : (f32, f32, f32) = UI_HLIGHT_CLR;
pub const UI_TRK_PHASEROW_FG_CLR   : (f32, f32, f32) = UI_LBL_BG_CLR;


//#[derive(Debug, Clone)]
//pub struct ConnectorData {
//    connection:  Option<(usize, usize)>,
//    items_left:  Vec<(String, bool)>,
//    items_right: Vec<(String, bool)>,
//    generation:  u64,
//}
//
//impl ConnectorData {
//    pub fn new() -> Self {
//        Self {
//            connection:  None,
//            items_left:  vec![],
//            items_right: vec![],
//            generation:  0,
//        }
//    }
//
//    pub fn clear(&mut self) {
//        self.connection = None;
//        self.items_left.clear();
//        self.items_right.clear();
//        self.generation += 1;
//    }
//
//    pub fn add_input(&mut self, lbl: String, active: bool) {
//        self.items_right.push((lbl, active));
//        self.generation += 1;
//    }
//    pub fn add_output(&mut self, lbl: String, active: bool) {
//        self.items_left.push((lbl, active));
//        self.generation += 1;
//    }
//
//    pub fn set_connection(&mut self, i: usize, o: usize) {
//        self.connection = Some((i, o));
//        self.generation += 1;
//    }
//
//    pub fn clear_connection(&mut self) {
//        self.connection = None;
//        self.generation += 1;
//    }
//
//    pub fn get_connection(&mut self) -> Option<(usize, usize)> {
//        self.connection
//    }
//}
//
//pub struct Connector {
//    data:           Rc<RefCell<ConnectorData>>,
//
//    yrow:           f32,
//    hover_idx:      Option<(bool, usize)>,
//    drag_src_idx:   Option<(bool, usize)>,
//    drag:           bool,
//
//    mouse_pos:      (f32, f32),
//    zones:          Vec<(Rect, (bool, usize))>,
//    debug_lbls:     Vec<(&'static str, &'static str)>,
//}
//
//impl Connector {
//    pub fn new(data: Rc<RefCell<ConnectorData>>) -> Self {
//        Self {
//            data,
//
//            yrow:           0.0,
//            hover_idx:      None,
//            drag_src_idx:   None,
//            drag:           false,
//
//            mouse_pos:      (0.0, 0.0),
//            zones:          vec![],
//            debug_lbls: vec![
//                ("input_0",  "output_0"),
//                ("input_1",  "output_1"),
//                ("input_2",  "output_2"),
//                ("input_3",  "output_3"),
//                ("input_4",  "output_4"),
//                ("input_5",  "output_5"),
//                ("input_6",  "output_6"),
//                ("input_7",  "output_7"),
//                ("input_8",  "output_8"),
//                ("input_9",  "output_9"),
//                ("input_10", "output_10"),
//                ("input_11", "output_11"),
//                ("input_12", "output_12"),
//                ("input_13", "output_13"),
//                ("input_14", "output_14"),
//                ("input_15", "output_15"),
//                ("input_16", "output_16"),
//                ("input_17", "output_17"),
//                ("input_18", "output_18"),
//                ("input_19", "output_19"),
//            ],
//        }
//    }
//
//    fn xy2pos(&self, x: f32, y: f32)
//        -> Option<(bool, usize)>
//    {
//        for z in &self.zones {
//            if z.0.is_inside(x, y) {
//                return Some(z.1);
//            }
//        }
//
//        None
//    }
//
//    fn get_current_con(&self) -> Option<(bool, (usize, usize))> {
//        let data = self.data.borrow();
//
//        let (a_inp, a) =
//            if let Some((inputs, row)) = self.drag_src_idx {
//                (inputs, row)
//            } else {
//                return data.connection.map(|con| (false, con));
//            };
//
//        let (b_inp, b) =
//            if let Some((inputs, row)) = self.hover_idx {
//                (inputs, row)
//            } else {
//                return data.connection.map(|con| (false, con));
//            };
//
//        if a_inp == b_inp {
//            if a_inp {
//                if data.items_left.len() == 1 {
//                    return Some((true, (0, a)))
//                }
//            } else {
//                if data.items_right.len() == 1 {
//                    return Some((true, (a, 0)))
//                }
//            }
//            return data.connection.map(|con| (false, con));
//        }
//
//        let (a, b) =
//            if b_inp { (a, b) }
//            else     { (b, a) };
//
//        if !data.items_left.get(a).map(|x| x.1).unwrap_or(false) {
//            return data.connection.map(|con| (false, con));
//        }
//
//        if !data.items_right.get(b).map(|x| x.1).unwrap_or(false) {
//            return data.connection.map(|con| (false, con));
//        }
//
//        Some((true, (a, b)))
//    }
//}
//
//impl Connector {
//    pub fn handle(
//        &mut self, w: &Widget, event: &InputEvent,
//        out_events: &mut Vec<(usize, Event)>)
//    {
//        match event {
//            InputEvent::MouseButtonPressed(MButton::Left) => {
//                if !w.is_hovered() {
//                    return;
//                }
//
//                let (x, y) = self.mouse_pos;
//                self.drag = true;
//                self.drag_src_idx = self.xy2pos(x, y);
//
//                if let Some((inputs, _)) = self.drag_src_idx {
//                    if inputs {
//                        if self.data.borrow().items_left.len() == 1 {
//                            self.drag_src_idx = Some((false, 0));
//                        }
//                    } else {
//                        if self.data.borrow().items_right.len() == 1 {
//                            self.drag_src_idx = Some((true, 0));
//                        }
//                    }
//                }
//
//                w.activate();
//                w.emit_redraw_required();
//            }
//            InputEvent::MouseButtonReleased(MButton::Left) => {
//                if !w.is_active() {
//                    return;
//                }
//
//                if let Some((_drag, con)) = self.get_current_con() {
//                    self.data.borrow_mut().connection = Some(con);
//                } else {
//                    self.data.borrow_mut().connection = None;
//                }
//
//                out_events.push((w.id(), Event {
//                    name: "change".to_string(),
//                    data: EvPayload::SetConnection(self.data.borrow().connection)
//                }));
//
//                self.drag = false;
//                self.drag_src_idx = None;
//
//                w.deactivate();
//                w.emit_redraw_required();
//            }
//            InputEvent::MousePosition(x, y) => {
//                self.mouse_pos = (*x, *y);
//
//                let old_hover = self.hover_idx;
//                self.hover_idx = self.xy2pos(*x, *y);
//
//                if old_hover != self.hover_idx {
//                    if let Some((inputs, idx)) = self.hover_idx {
//                        out_events.push((w.id(), Event {
//                            name: "connection_hover".to_string(),
//                            data: EvPayload::ConnectionHover {
//                                is_input: inputs,
//                                index: idx
//                            }
//                        }));
//                    }
//
//                    w.emit_redraw_required();
//                }
//            }
//            _ => {}
//        }
//    }
//
//    pub fn draw(
//        &mut self, w: &Widget, style: &Style, pos: Rect,
//        real_pos: Rect, p: &mut Painter)
//    {
//        let mut dbg = LblDebugTag::from_id(w.id());
//        dbg.set_offs((real_pos.x - pos.x, real_pos.y - pos.y));
//
//        let data = self.data.borrow();
//
//        self.zones.clear();
//        let pos = pos.floor();
//
//        let row_h = data.items_left.len().max(data.items_right.len());
//
//        // XXX: - 1.0 on height and width for some extra margin so that the
//        //      rectangles for the ports are not clipped.
//        let yrow = (pos.h / (row_h as f32)).floor();
//        let xcol = (pos.w / 3.0).floor();
//
//        self.yrow = yrow;
//
//        let pos = Rect {
//            x: pos.x,
//            y: pos.y,
//            w: xcol * 3.0,
//            h: yrow * (row_h as f32),
//        };
//
//        let does_hover_this_widget = w.is_hovered();
//
//        let btn_rect = Rect::from(0.0, 0.0, xcol, yrow);
//        for row in 0..row_h {
//            let yo      = row as f32 * yrow;
//            let txt_pad = 2.0 * UI_CON_BORDER_W;
//            let txt_w   = xcol - 2.0 * txt_pad;
//
//            if let Some((lbl, active)) = data.items_left.get(row) {
//                p.rect_stroke_r(
//                    UI_CON_BORDER_W,
//                    UI_CON_BORDER_CLR,
//                    btn_rect.offs(pos.x, pos.y + yo));
//                self.zones.push(
//                    (btn_rect.offs(real_pos.x, real_pos.y + yo),
//                     (false, row)));
//
//                let fs =
//                    calc_font_size_from_text(
//                        p, &lbl, style.font_size, txt_w);
//                p.label(
//                    fs, -1, if *active { UI_PRIM_CLR } else { UI_INACTIVE_CLR },
//                    pos.x + txt_pad, pos.y + yo,
//                    txt_w, yrow, &lbl,
//                    dbg.source(
//                        self.debug_lbls.get(row)
//                            .unwrap_or(&("input_", "output_")).0));
//            }
//
//            if let Some((lbl, active)) = data.items_right.get(row) {
//                p.rect_stroke_r(
//                    UI_CON_BORDER_W,
//                    UI_CON_BORDER_CLR,
//                    btn_rect.offs(
//                        pos.x + 2.0 * xcol - 1.0,
//                        pos.y + yo));
//                self.zones.push(
//                    (btn_rect.offs(
//                        real_pos.x + 2.0 * xcol - 1.0,
//                        real_pos.y + yo),
//                     (true, row)));
//
//                let fs =
//                    calc_font_size_from_text(
//                        p, &lbl, style.font_size, txt_w);
//                p.label(
//                    fs, 1, if *active { UI_PRIM_CLR } else { UI_INACTIVE_CLR },
//                    pos.x + txt_pad + 2.0 * xcol - UI_CON_BORDER_W,
//                    pos.y + yo,
//                    txt_w, yrow, &lbl,
//                    dbg.source(
//                        self.debug_lbls.get(row)
//                            .unwrap_or(&("input_", "output_")).1));
//            }
//        }
//
//        if let Some((inputs, row)) = self.hover_idx {
//            let items = if inputs { &data.items_right } else { &data.items_left };
//
//            if let Some((_lbl, active)) = items.get(row) {
//                if *active {
//                    let xo = if inputs { xcol * 2.0 - 1.0 } else { 0.0 };
//                    let yo = row as f32 * yrow;
//
//                    if does_hover_this_widget {
//                        p.rect_stroke_r(
//                            UI_CON_BORDER_W,
//                            UI_CON_HOV_CLR,
//                            btn_rect.offs(pos.x + xo, pos.y + yo));
//                    }
//                }
//            }
//        }
//
//        if let Some((inputs, row)) = self.drag_src_idx {
//            let xo = if inputs { xcol * 2.0 - 1.0 } else { 0.0 };
//            let yo = row as f32 * yrow;
//
//            if self.drag {
//                p.rect_stroke_r(
//                    UI_CON_BORDER_W,
//                    UI_SELECT_CLR,
//                    btn_rect.offs(pos.x + xo, pos.y + yo));
//            }
//        }
//
//        if let Some((drag, (a, b))) = self.get_current_con() {
//            let ay = a as f32 * yrow;
//            let by = b as f32 * yrow;
//
//            p.path_stroke(
//                4.0,
//                if drag { UI_CON_HOV_CLR } else { UI_PRIM_CLR },
//                &mut [
//                    (xcol,                         ay + yrow * 0.5),
//                    (xcol + xcol * 0.25,           ay + yrow * 0.5),
//                    (2.0 * xcol - xcol * 0.25,     by + yrow * 0.5),
//                    (2.0 * xcol - UI_CON_BORDER_W, by + yrow * 0.5),
//                ].iter().copied().map(|(x, y)|
//                    ((pos.x + x).floor(),
//                     (pos.y + y).floor())),
//                false);
//        }
//    }
//
//    pub fn get_generation(&self) -> u64 {
//        self.data.borrow().generation
//    }
//}

pub trait PatternEditorFeedback: std::fmt::Debug {
    fn get_phase(&self) -> f32;
}

#[derive(Debug)]
pub struct PatternEditorFeedbackDummy { }

impl PatternEditorFeedbackDummy {
    pub fn new() -> Self { Self { } }
}

impl PatternEditorFeedback for PatternEditorFeedbackDummy {
    fn get_phase(&self) -> f32 { 0.5 }
}

#[derive(Debug)]
pub struct PatternEditor {
    rows:       usize,
    columns:    usize,

    pattern:        Arc<Mutex<dyn UIPatternModel>>,
    pattern_fb:     Arc<Mutex<dyn PatternEditorFeedback>>,

    cursor:         (usize, usize),
    enter_mode:     EnterMode,

    cell_zone:      Rect,

    last_set_value: u16,

    edit_step:      usize,
    octave:         u16,
    follow_phase:   bool,
    info_line:      String,
    update_info_line: bool,

    real_pos:       Rect,
    modkeys:        ModifierTracker,
    mouse_pos:      (f32, f32),
}

impl PatternEditor {
    pub fn new(columns: usize) -> Self {
        Self {
            rows:       10,
            columns,

            pattern:    Arc::new(Mutex::new(PatternData::new(256))),
            pattern_fb: Arc::new(Mutex::new(PatternEditorFeedbackDummy::new())),
            cursor:     (1, 2),
            enter_mode: EnterMode::None,

            cell_zone: Rect::from(0.0, 0.0, 0.0, 0.0),

            last_set_value: 0,

            edit_step:        4,
            octave:           4,
            follow_phase:     false,
            update_info_line: true,
            info_line:        String::from(""),
            real_pos:         Rect::from(0.0, 0.0, 0.0, 0.0),
            modkeys:          ModifierTracker::new(),

            mouse_pos:        (0.0, 0.0),
        }
    }

    pub fn calc_row_offs(&self, rows: usize) -> usize {
        let rows = rows as i64;
        let mut cur = self.cursor.0 as i64;

        let margin = rows * 1 / 3;
        let margin = (margin / 2) * 2;
        let page_rows = rows - margin;

        if page_rows <= 0 {
            return cur as usize;
        }

        let mut scroll_page = 0;

        while cur >= page_rows {
            cur -= page_rows;
            scroll_page += 1;
        }

        if scroll_page > 0 {
            (scroll_page * page_rows - (margin / 2)) as usize
        } else {
            0
        }
    }

    fn handle_key_event(&mut self, key: &Key) {
        let mut pat = self.pattern.lock().unwrap();

        let mut edit_step = self.edit_step as i16;

        if self.modkeys.ctrl {
            edit_step = 1;
        }

        if edit_step < 1 { edit_step = 1; }

        let octave = self.octave;

        let mut reset_entered_value = false;

        match key {
            Key::Home => {
                self.cursor.0 = 0;
                reset_entered_value = true;
            },
            Key::End => {
                self.cursor.0 = pat.rows() - self.edit_step;
                reset_entered_value = true;
            },
            Key::PageUp => {
                if self.modkeys.shift {
                    pat.change_value(
                        self.cursor.0,
                        self.cursor.1,
                        0x100);

                } else {
                    advance_cursor(
                        &mut self.cursor,
                        -2 * edit_step as i16,
                        0, &mut *pat);
                }
                reset_entered_value = true;
            },
            Key::PageDown => {
                if self.modkeys.shift {
                    pat.change_value(
                        self.cursor.0,
                        self.cursor.1,
                        -0x100);

                } else {
                    advance_cursor(
                        &mut self.cursor,
                        2 * edit_step as i16,
                        0, &mut *pat);
                }
                reset_entered_value = true;
            },
            Key::ArrowUp => {
                if self.modkeys.shift {
                    if self.modkeys.ctrl {
                        pat.change_value(
                            self.cursor.0,
                            self.cursor.1,
                            0x100);
                    } else {
                        pat.change_value(
                            self.cursor.0,
                            self.cursor.1,
                            0x10);
                    }

                } else if let EnterMode::Rows(_) = self.enter_mode {
                    let rows = pat.rows() + 1;
                    pat.set_rows(rows);
                    self.update_info_line = true;

                } else {
                    advance_cursor(
                        &mut self.cursor,
                        -edit_step as i16,
                        0, &mut *pat);
                }
                reset_entered_value = true;
            },
            Key::ArrowDown => {
                if self.modkeys.shift {
                    if self.modkeys.ctrl {
                        pat.change_value(
                            self.cursor.0,
                            self.cursor.1,
                            -0x100);
                    } else {
                        pat.change_value(
                            self.cursor.0,
                            self.cursor.1,
                            -0x10);
                    }

                } else if let EnterMode::Rows(_) = self.enter_mode {
                    if pat.rows() > 0 {
                        let rows = pat.rows() - 1;
                        pat.set_rows(rows);
                        self.update_info_line = true;
                    }
                } else {
                    advance_cursor(
                        &mut self.cursor,
                        edit_step as i16,
                        0, &mut *pat);
                }
                reset_entered_value = true;
            },
            Key::ArrowLeft => {
                if self.modkeys.shift {
                    pat.change_value(
                        self.cursor.0,
                        self.cursor.1,
                        -0x1);

                } else {
                    advance_cursor(
                        &mut self.cursor, 0, -1, &mut *pat);
                }
                reset_entered_value = true;
            },
            Key::ArrowRight => {
                if self.modkeys.shift {
                    pat.change_value(
                        self.cursor.0,
                        self.cursor.1,
                        0x1);

                } else {
                    advance_cursor(
                        &mut self.cursor, 0, 1, &mut *pat);
                }
                reset_entered_value = true;
            },
            Key::Delete => {
                pat.clear_cell(
                    self.cursor.0,
                    self.cursor.1);
                advance_cursor(
                    &mut self.cursor,
                    edit_step as i16, 0, &mut *pat);
                reset_entered_value = true;
            },
            Key::Character(c) => {
                match &c[..] {
                    "+" => {
                        self.octave += 1;
                        self.octave = self.octave.min(9);
                        self.update_info_line = true;
                    },
                    "-" => {
                        if self.octave > 0 {
                            self.octave -= 1;
                            self.update_info_line = true;
                        }
                    },
                    "/" => {
                        if self.edit_step > 0 {
                            self.edit_step -= 1;
                        }
                        self.update_info_line = true;
                    },
                    "*" => {
                        self.edit_step += 1;
                        self.update_info_line = true;
                    },
                    _ => {},
                }

                match self.enter_mode {
                    EnterMode::EnterValues(v) => {
                        match &c[..] {
                            "." => {
                                pat.set_cell_value(
                                    self.cursor.0,
                                    self.cursor.1,
                                    self.last_set_value);
                                advance_cursor(
                                    &mut self.cursor,
                                    edit_step as i16, 0, &mut *pat);
                                reset_entered_value = true;
                            },
                            "," => {
                                let cell_value =
                                    pat.get_cell_value(
                                        self.cursor.0,
                                        self.cursor.1);
                                self.last_set_value = cell_value;
                                advance_cursor(
                                    &mut self.cursor,
                                    edit_step as i16, 0, &mut *pat);
                                reset_entered_value = true;
                            },
                            _ if pat.is_col_note(self.cursor.1) => {
                                if let Some(value) =
                                    note_from_char(&c[..], octave)
                                {
                                    pat.set_cell_value(
                                        self.cursor.0,
                                        self.cursor.1,
                                        value as u16);
                                    advance_cursor(
                                        &mut self.cursor,
                                        edit_step as i16, 0, &mut *pat);
                                    self.last_set_value = value as u16;
                                }
                            },
                            _ => {
                                if let Some(value) = num_from_char(&c[..]) {
                                    match v {
                                        EnterValue::None => {
                                            let nv = value << 0x8;
                                            self.enter_mode =
                                                EnterMode::EnterValues(
                                                    EnterValue::One(nv as u16));
                                            pat.set_cell_value(
                                                self.cursor.0,
                                                self.cursor.1,
                                                nv as u16);
                                            self.last_set_value = nv as u16;
                                        },
                                        EnterValue::One(v) => {
                                            let nv = v | (value << 0x4);
                                            self.enter_mode =
                                                EnterMode::EnterValues(
                                                    EnterValue::Two(nv as u16));
                                            pat.set_cell_value(
                                                self.cursor.0,
                                                self.cursor.1,
                                                nv as u16);
                                            self.last_set_value = nv as u16;
                                        },
                                        EnterValue::Two(v) => {
                                            let nv = v | value;
                                            self.enter_mode =
                                                EnterMode::EnterValues(
                                                    EnterValue::None);
                                            pat.set_cell_value(
                                                self.cursor.0,
                                                self.cursor.1,
                                                nv as u16);
                                            self.last_set_value = nv as u16;
                                            advance_cursor(
                                                &mut self.cursor,
                                                edit_step as i16, 0, &mut *pat);
                                        },
                                    }
                                }
                            },
                        }
                    },
                    EnterMode::Rows(v) => {
                        match v {
                            EnterValue::None => {
                                if let Some(value) = num_from_char(&c[..]) {
                                    pat.set_rows((value << 4) as usize);
                                    self.update_info_line = true;
                                    self.enter_mode =
                                        EnterMode::Rows(EnterValue::One(value));
                                }
                            },
                            EnterValue::One(v) => {
                                if let Some(value) = num_from_char(&c[..]) {
                                    pat.set_rows((v << 4 | value) as usize);
                                    self.update_info_line = true;
                                    self.enter_mode = EnterMode::None;
                                }
                            },
                            _ => {
                                self.enter_mode = EnterMode::None;
                            },
                        }
                    },
                    EnterMode::EditStep => {
                        if let Some(value) = num_from_char(&c[..]) {
                            if self.modkeys.ctrl {
                                self.edit_step = (value + 0x10) as usize;
                            } else {
                                self.edit_step = value as usize;
                            }
                            self.update_info_line = true;
                        }

                        self.enter_mode = EnterMode::None;
                    },
                    EnterMode::Octave => {
                        if let Some(value) = num_from_char(&c[..]) {
                            self.octave = value;
                            self.update_info_line = true;
                        }

                        self.enter_mode = EnterMode::None;
                    },
                    EnterMode::ColType => {
                        match &c[..] {
                            "n" => {
                                pat.set_col_note_type(self.cursor.1);
                            },
                            "s" => {
                                pat.set_col_step_type(self.cursor.1);
                            },
                            "v" => {
                                pat.set_col_value_type(self.cursor.1);
                            },
                            "g" => {
                                pat.set_col_gate_type(self.cursor.1);
                            },
                            _ => {},
                        }
                        self.enter_mode = EnterMode::None;
                    },
                    EnterMode::Delete => {
                        match &c[..] {
                            "r" => {
                                for i in 0..pat.cols() {
                                    pat.clear_cell(
                                        self.cursor.0,
                                        i);
                                }
                            },
                            "c" => {
                                for i in 0..pat.rows() {
                                    pat.clear_cell(
                                        i,
                                        self.cursor.1);
                                }
                            },
                            "s" => {
                                for i in 0..self.edit_step {
                                    pat.clear_cell(
                                        self.cursor.0 + i,
                                        self.cursor.1);
                                }
                            },
                            _ => {},
                        }
                        self.enter_mode = EnterMode::None;
                    },
                    EnterMode::None => {
                        match &c[..] {
                            "e" => {
                                self.enter_mode = EnterMode::EditStep;
                            },
                            "r" => {
                                self.enter_mode =
                                    EnterMode::Rows(EnterValue::None);
                            },
                            "o" => {
                                self.enter_mode = EnterMode::Octave;
                            },
                            "c" => {
                                self.enter_mode = EnterMode::ColType;
                            },
                            "d" => {
                                self.enter_mode = EnterMode::Delete;
                            },
                            "f" => {
                                self.follow_phase = !self.follow_phase;
                                self.update_info_line = true;
                            },
                            _ => {},
                        }
                    },
                }
            },
            Key::Escape => {
                self.enter_mode = EnterMode::None;
            },
            Key::Enter => {
                self.enter_mode =
                    match self.enter_mode {
                        EnterMode::EnterValues(_)
                            => EnterMode::None,
                        _   => EnterMode::EnterValues(EnterValue::None),
                    }
            },
            _ => {},
        }

        if reset_entered_value {
            if let EnterMode::EnterValues(_) = self.enter_mode {
                self.enter_mode = EnterMode::EnterValues(EnterValue::None);
            }
        }
    }

    pub fn handle(
        &mut self, w: &Widget, event: &InputEvent,
        _out_events: &mut Vec<(usize, Event)>)
    {
        self.modkeys.handle(event);

        match event {
//                WindowEvent::FocusOut => {
//                    self.enter_mode = EnterMode::None;
//                },
              InputEvent::MouseButtonPressed(MButton::Left)
            | InputEvent::MouseButtonPressed(MButton::Right) => {

                if w.is_hovered() {
                    let x = self.mouse_pos.0 - self.real_pos.x;
                    let y = self.mouse_pos.1 - self.real_pos.y;

                    let pat = self.pattern.lock().unwrap();

                    let xi = (x - self.cell_zone.x) / UI_TRK_COL_WIDTH;
                    let yi = (y - self.cell_zone.y) / UI_TRK_ROW_HEIGHT;

                    let xi = xi.max(1.0);
                    let yi = yi.max(1.0);

                    let row_scroll_offs = self.calc_row_offs(self.rows);
                    let yr = (yi as usize - 1) + row_scroll_offs;

                    //d// println!("INDEX: {} {},{} => {},{}", index, x, y, xi, yi);
                    self.cursor = (yr, xi as usize - 1);

                    self.cursor.0 = self.cursor.0.min(pat.rows() - 1);
                    self.cursor.1 = self.cursor.1.min(pat.cols() - 1);

                    w.activate();

                    w.emit_redraw_required();
                }
            },
            InputEvent::MouseWheel(y) => {
                if w.is_hovered() {
                    let pat = self.pattern.lock().unwrap();

                    if *y > 0.0 {
                        if self.cursor.0 > 0 {
                            self.cursor.0 -= 1;
                        }
                    } else {
                        if (self.cursor.0 + 1) < pat.rows() {
                            self.cursor.0 += 1;
                        }
                    }

                    w.emit_redraw_required();
                }
            },
            InputEvent::KeyPressed(key) => {
                //d// println!("KEY: {:?}", key);

                if w.is_hovered() || w.is_active() {
                    self.handle_key_event(&key.key);

                    w.emit_redraw_required();
                }
            },
            InputEvent::MousePosition(x, y) => {
                self.mouse_pos = (*x, *y);
            },
            _ => {},
        }
    }

    pub fn draw(
        &mut self, w: &Widget, style: &Style, pos: Rect,
        real_pos: Rect, p: &mut Painter)
    {
        let mut dbg = LblDebugTag::from_id(w.id());
        dbg.set_offs((real_pos.x - pos.x, real_pos.y - pos.y));

        p.clip_region(pos.x, pos.y, pos.w, pos.h);

        let is_hovered = w.is_hovered();
        let is_active  = w.is_active();

        let orig_pos  = pos;

        let mut pat = self.pattern.lock().unwrap();

        if self.cursor.0 >= pat.rows() {
            self.cursor.0 = pat.rows() - 1;
        }

        let notify_click = false;

        p.rect_fill(style.bg_color, pos.x, pos.y, pos.w, pos.h);

        let mode_line =
            match self.enter_mode {
                EnterMode::EnterValues(_) => {
                    Some("> [Values]")
                },
                EnterMode::EditStep => {
                    Some("> [Step] (0-F, Ctrl + 0-F)")
                },
                EnterMode::Octave => {
                    Some("> [Octave] (0-8)")
                },
                EnterMode::ColType => {
                    Some("> [Column] (n)ote,(s)tep,(v)alue,(g)ate")
                },
                EnterMode::Delete => {
                    Some("> [Delete] (r)ow,(c)olumn,(s)tep")
                },
                EnterMode::Rows(EnterValue::None) => {
                    Some("> [Rows] (0-F 00-F0, Up/Down +1/-1)")
                },
                EnterMode::Rows(EnterValue::One(_)) => {
                    Some("> [Rows] (0-F 00-0F)")
                },
                EnterMode::Rows(EnterValue::Two(_)) => None,
                EnterMode::None => {
                    if notify_click {
                        Some("*** >>> click for keyboard focus <<< ***")
                    } else {
                        None
                    }
                },
            };

        if let Some(mode_line) = mode_line {
            p.label_mono(
                UI_TRK_FONT_SIZE * 0.9,
                -1,
                UI_TRK_TEXT_CLR,
                pos.x,
                pos.y,
                pos.w,
                UI_TRK_ROW_HEIGHT,
                &mode_line,
                dbg.source("mode_line"));
        }

        if self.update_info_line {
            self.info_line =
                format!(
                    "ES: {:02} | Oct: {:02} | Curs: {} | R: {:02}",
                    self.edit_step,
                    self.octave,
                    if self.follow_phase { "->" }
                    else                 { "." },
                    pat.rows());
            self.update_info_line = false;
        }

        p.label_mono(
            UI_TRK_FONT_SIZE,
            -1,
            UI_TRK_TEXT_CLR,
            pos.x,
            pos.y + UI_TRK_ROW_HEIGHT,
            pos.w,
            UI_TRK_ROW_HEIGHT,
            &self.info_line,
            dbg.source("info_line"));

        for ic in 0..self.columns {
            let x = (ic + 1) as f32 * UI_TRK_COL_WIDTH;
            let y = 2.0             * UI_TRK_ROW_HEIGHT;

            dbg.set_logic_pos(ic as i32, -1 as i32);

            p.label_mono(
                UI_TRK_FONT_SIZE,
                0,
                UI_TRK_TEXT_CLR,
                pos.x + x,
                pos.y + y,
                UI_TRK_COL_WIDTH,
                UI_TRK_ROW_HEIGHT,
                if pat.is_col_note(ic) {
                    "Note"
                } else if pat.is_col_step(ic) {
                    "Step"
                } else if pat.is_col_gate(ic) {
                    "Gate"
                } else {
                    "Value"
                },
                dbg.source("header"));
        }

        p.path_stroke(
            1.0,
            UI_TRK_COL_DIV_CLR,
            &mut [
                (pos.x,         pos.y + 3.0 * UI_TRK_ROW_HEIGHT - 0.5),
                (pos.x + pos.w, pos.y + 3.0 * UI_TRK_ROW_HEIGHT - 0.5),
            ].iter().copied(),
            false);

        let pos = pos.crop_top(2.0 * UI_TRK_ROW_HEIGHT);

        self.cell_zone = Rect {
            x: pos.x - orig_pos.x,
            y: pos.y - orig_pos.y,
            w: pos.w,
            h: pos.h,
        };

        self.rows = (self.cell_zone.h / UI_TRK_ROW_HEIGHT).round() as usize - 1;

        // center the cursor row
        // - but lock the start of the pattern to the top
        // - and lock the end of the pattern to the end
        let row_scroll_offs = self.calc_row_offs(self.rows);

        for ir in 0..self.rows {
            let y = (ir + 1) as f32 * UI_TRK_ROW_HEIGHT;
            let ir = row_scroll_offs as usize + ir;

            if ir >= pat.rows() {
                break;
            }

            dbg.set_logic_pos(-1 as i32, ir as i32);

            if self.edit_step > 0 && ir % self.edit_step == 0 {
                p.rect_fill(
                    UI_TRK_BG_ALT_CLR,
                    pos.x,
                    pos.y + y,
                    pos.w,
                    UI_TRK_ROW_HEIGHT);
            }

            p.label_mono(
                UI_TRK_FONT_SIZE,
                1,
                UI_TRK_TEXT_CLR,
                pos.x - UI_TRK_COL_DIV_PAD,
                pos.y + y,
                UI_TRK_COL_WIDTH,
                UI_TRK_ROW_HEIGHT,
                &format!("{:-02}", ir),
                dbg.source("row_idx"));

            let pat_fb    = self.pattern_fb.lock().expect("Matrix lockable");
            let phase     = pat_fb.get_phase();
            let phase_row = (pat.rows() as f32 * phase).floor() as usize;

            if self.follow_phase {
                self.cursor.0 = phase_row;
            }

            for ic in 0..self.columns {
                let x = (ic + 1) as f32 * UI_TRK_COL_WIDTH;
                let is_note_col = pat.is_col_note(ic);

                dbg.set_logic_pos(ic as i32, ir as i32);

                let txt_clr =
                    if (ir, ic) == self.cursor || ir == phase_row {
                        p.rect_fill(
                            if (ir, ic) == self.cursor {
                                if is_active {
                                    UI_TRK_CURSOR_BG_SEL_CLR
                                } else if is_hovered {
                                    UI_TRK_CURSOR_BG_HOV_CLR
                                } else {
                                    UI_TRK_CURSOR_BG_CLR
                                }
                            } else { UI_TRK_PHASEROW_BG_CLR },
                            pos.x + x,
                            pos.y + y,
                            UI_TRK_COL_WIDTH,
                            UI_TRK_ROW_HEIGHT);

                        if (ir, ic) == self.cursor {
                            UI_TRK_CURSOR_FG_CLR
                        } else {
                            UI_TRK_PHASEROW_FG_CLR
                        }
                    } else if
                           (is_active || is_hovered)
                        && ir == self.cursor.0
                    {
                        let hl_clr =
                            if is_active {
                                UI_TRK_CURSOR_BG_SEL_CLR
                            } else { // if is_hovered {
                                UI_TRK_CURSOR_BG_HOV_CLR
                            };

                        if (ir, ic) == self.cursor {
                            p.rect_fill(
                                hl_clr,
                                pos.x + x,
                                pos.y + y,
                                UI_TRK_COL_WIDTH,
                                UI_TRK_ROW_HEIGHT);

                        } else {
                            if self.enter_mode != EnterMode::None {
                                p.path_stroke(
                                    1.0,
                                    hl_clr,
                                    &mut [
                                        (pos.x + x + 1.5,              pos.y + y + UI_TRK_ROW_HEIGHT - 0.5),
                                        (pos.x + x + UI_TRK_COL_WIDTH - 0.5, pos.y + y + UI_TRK_ROW_HEIGHT - 0.5),
                                        (pos.x + x + UI_TRK_COL_WIDTH - 0.5, pos.y + y + 0.5),
                                        (pos.x + x + 1.5,              pos.y + y + 0.5),
                                    ].iter().copied(),
                                    true);
                            }
                        }

                        UI_TRK_TEXT_CLR
                    } else {
                        UI_TRK_TEXT_CLR
                    };

                let cell_value = pat.get_cell_value(ir, ic);
                if let Some(s) = pat.get_cell(ir, ic) {

                    if is_note_col {
                        p.label_mono(
                            UI_TRK_FONT_SIZE,
                            0,
                            txt_clr,
                            pos.x + x,
                            pos.y + y,
                            UI_TRK_COL_WIDTH,
                            UI_TRK_ROW_HEIGHT,
                            value2note_name(cell_value)
                                .unwrap_or(s),
                            dbg.source("cell"));
                    } else {
                        p.label_mono(
                            UI_TRK_FONT_SIZE,
                            0,
                            txt_clr,
                            pos.x + x,
                            pos.y + y,
                            UI_TRK_COL_WIDTH,
                            UI_TRK_ROW_HEIGHT,
                            s,
                            dbg.source("cell"));
                    }
                } else {
                    p.label_mono(
                        UI_TRK_FONT_SIZE,
                        0,
                        txt_clr,
                        pos.x + x,
                        pos.y + y,
                        UI_TRK_COL_WIDTH,
                        UI_TRK_ROW_HEIGHT,
                        "---",
                        dbg.source("cell"));
                }
            }
        }

        for ic in 0..self.columns {
            let x = (ic + 1) as f32 * UI_TRK_COL_WIDTH;

            p.path_stroke(
                1.0,
                UI_TRK_COL_DIV_CLR,
                &mut [
                    (pos.x + x + 0.5, pos.y),
                    (pos.x + x + 0.5, pos.y + pos.h),
                ].iter().copied(),
                false);
        }

        p.reset_clip_region();
    }
}

fn value2note_name(val: u16) -> Option<&'static str> {
    if !(21..=127).contains(&val) {
        return None;
    }

    Some(match val {
        21 => "A-0",
        22 => "A#0",
        23 => "B-0",
        24 => "C-1", 25 => "C#1", 26 => "D-1", 27 => "D#1", 28 => "E-1", 29 => "F-1",
        30 => "F#1", 31 => "G-1", 32 => "G#1", 33 => "A-1", 34 => "A#1", 35 => "B-1",
        36 => "C-2", 37 => "C#2", 38 => "D-2", 39 => "D#2", 40 => "E-2", 41 => "F-2",
        42 => "F#2", 43 => "G-2", 44 => "G#2", 45 => "A-2", 46 => "A#2", 47 => "B-2",
        48 => "C-3", 49 => "C#3", 50 => "D-3", 51 => "D#3", 52 => "E-3", 53 => "F-3",
        54 => "F#3", 55 => "G-3", 56 => "G#3", 57 => "A-3", 58 => "A#3", 59 => "B-3",
        60 => "C-4", 61 => "C#4", 62 => "D-4", 63 => "D#4", 64 => "E-4", 65 => "F-4",
        66 => "F#4", 67 => "G-4", 68 => "G#4", 69 => "A-4", 70 => "A#4", 71 => "B-4",
        72 => "C-5", 73 => "C#5", 74 => "D-5", 75 => "D#5", 76 => "E-5", 77 => "F-5",
        78 => "F#5", 79 => "G-5", 80 => "G#5", 81 => "A-5", 82 => "A#5", 83 => "B-5",
        84 => "C-6", 85 => "C#6", 86 => "D-6", 87 => "D#6", 88 => "E-6", 89 => "F-6",
        90 => "F#6", 91 => "G-6", 92 => "G#6", 93 => "A-6", 94 => "A#6", 95 => "B-6",
        96 => "C-7", 97 => "C#7", 98 => "D-7", 99 => "D#7", 100 => "E-7", 101 => "F-7",
        102 => "F#7", 103 => "G-7", 104 => "G#7", 105 => "A-7", 106 => "A#7", 107 => "B-7",
        108 => "C-8", 109 => "C#8", 110 => "D-8", 111 => "D#8", 112 => "E-8", 113 => "F-8",
        114 => "F#8", 115 => "G-8", 116 => "G#8", 117 => "A-8", 118 => "A#8", 119 => "B-8",
        120 => "C-9", 121 => "C#9", 122 => "D-9", 123 => "D#9", 124 => "E-9", 125 => "F-9",
        126 => "F#9", 127 => "G-9", 128 => "G#9", 129 => "A-9", 130 => "A#9", 131 => "B-9",
        _ => "???",
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum EnterValue {
    None,
    One(u16),
    Two(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum EnterMode {
    None,
    EnterValues(EnterValue),
    EditStep,
    Octave,
    ColType,
    Delete,
    Rows(EnterValue),
}

pub fn advance_cursor(
    cursor:   &mut (usize, usize),
    row_offs: i16,
    col_offs: i16,
    pat:      &mut dyn UIPatternModel)
{
    if row_offs >= 0 {
        let row_offs = row_offs as usize;
        if ((*cursor).0 + row_offs) < pat.rows() {
            (*cursor).0 += row_offs;
        }

    } else {
        let row_offs = row_offs.abs() as usize;
        if (*cursor).0 >= row_offs {
            (*cursor).0 -= row_offs;
        } else {
            (*cursor).0 = 0;
        }
    }

    if col_offs >= 0 {
        let col_offs = col_offs as usize;
        if ((*cursor).1 + col_offs) < pat.cols() {
            (*cursor).1 += col_offs;
        }

    } else {
        let col_offs = col_offs.abs() as usize;
        if (*cursor).1 >= col_offs {
            (*cursor).1 -= col_offs;
        }
    }
}

fn note_from_char(c: &str, octave: u16) -> Option<u16> {
    let octave = (octave + 1) * 12;

    match c {
        "z" => Some(octave),
        "s" => Some(octave + 1),
        "x" => Some(octave + 2),
        "d" => Some(octave + 3),
        "c" => Some(octave + 4),
        "v" => Some(octave + 5),
        "g" => Some(octave + 6),
        "b" => Some(octave + 7),
        "h" => Some(octave + 8),
        "n" => Some(octave + 9),
        "j" => Some(octave + 10),
        "m" => Some(octave + 11),

        "," => Some(octave + 12),
        "l" => Some(octave + 13),
        "." => Some(octave + 14),
        ";" => Some(octave + 15),
        // "/" => Some(octave + 16), // collides with the "/" bind for edit step

        "q" => Some(octave + 12),
        "2" => Some(octave + 13),
        "w" => Some(octave + 14),
        "3" => Some(octave + 15),
        "e" => Some(octave + 16),
        "r" => Some(octave + 17),
        "5" => Some(octave + 18),
        "t" => Some(octave + 19),
        "6" => Some(octave + 20),
        "y" => Some(octave + 21),
        "7" => Some(octave + 22),
        "u" => Some(octave + 23),

        "i" => Some(octave + 24),
        "9" => Some(octave + 25),
        "o" => Some(octave + 26),
        "0" => Some(octave + 27),
        "p" => Some(octave + 28),
        "[" => Some(octave + 29),
        "=" => Some(octave + 30),
        "]" => Some(octave + 31),
        _ => None,
    }
}

fn num_from_char(c: &str) -> Option<u16> {
    match c {
        "0" => Some(0),
        "1" => Some(1),
        "2" => Some(2),
        "3" => Some(3),
        "4" => Some(4),
        "5" => Some(5),
        "6" => Some(6),
        "7" => Some(7),
        "8" => Some(8),
        "9" => Some(9),
        "a" => Some(0xA),
        "b" => Some(0xB),
        "c" => Some(0xC),
        "d" => Some(0xD),
        "e" => Some(0xE),
        "f" => Some(0xF),
        _ => None,
    }
}
