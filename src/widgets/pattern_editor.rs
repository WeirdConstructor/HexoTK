// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use crate::constants::*;
use super::*;
use super::util::*;

use std::rc::Rc;
use std::cell::RefCell;

pub trait UIPatternModel: Debug {
    fn get_cell(&mut self, row: usize, col: usize) -> Option<&str>;
    fn is_col_note(&self, col: usize) -> bool;
    fn is_col_gate(&self, col: usize) -> bool;

    fn rows(&self) -> usize;
    fn cols(&self) -> usize;

    fn clear_cell(&mut self, row: usize, col: usize);
    fn set_col_note_type(&mut self, col: usize);
    fn set_col_gate_type(&mut self, col: usize);
    fn set_col_value_type(&mut self, col: usize);

    fn set_cell_note(&mut self, row: usize, col: usize, note: &str);
    fn set_cell_value(&mut self, row: usize, col: usize, val: u16);

    fn set_cursor(&mut self, row: usize, col: usize);
    fn get_cursor(&self) -> (usize, usize);
    fn set_edit_step(&mut self, es: usize);
    fn get_edit_step(&mut self) -> usize;
}

//pub struct PatternData {
//    rows:   
//    column_types: 
//}
//
//pub struct TrackerBackend {
//    patterns: Vec<Rc<PatternData>>,
//    msg_cons: RingBuffer<TrackerMessage>,
//}
//
//pub struct TrackerFrontend {
//    patterns: Vec<Rc<PatternData>>,
//    msg_prod: RingBuffer<TrackerMessage>,
//}
//

/* Plan:

- The actual tracker contents is stored in a TrackerModel
    - Synchronization with the backend is done via messages:
        - TrackerMessage::
            - InitPattern   { pat_idx, data, columns, rows }
            - SetCell       { pat_idx, offs, value }
            - SetLength     { pat_idx, rows }
            - SetColumnInterpolation { pat_idx, interp }
    - TrackSequencer nodes are specialcased on reception in GraphMessage::NewNode
      and they are assigned an Rc<PatternData>.
- Selection of the displayed Tracker in the util panel is done by
  the focussed node id in the MatrixEditor.
- There are 256 predefined trackers, if trackers outside that range
  are accessed, they are ignored.
- Each pattern has at max 256 rows, which can be artificially
  limited.

UI:
    - Settings:
        - Length setting how?
        - Edit Jump Distance
        - Input Octave offset
        - Follow mode Cursor/Phase
    - Input notes via keyboard
    - KeyboardFocusZone
        => Sends all keypresses as event to the widget tree.
        => UIEvent::Key
        => Simulate Keyrepeat by a timestamp in the InputMode?!
        => Make a mode based input
            - Edit cell mode
                - Cursor keys for navigation
                - 1 to 9/0 keys insert digits, they are the parts after the
                  decimal dot.
                - ysxdcvgbhnjm are a pitch input
                    - +/- change octave
                - Return jumps to the set Edit Jump Distance
                - Esc exits enter mode
            - Return enters enter mode
            - CellClick enters enter mode too
        => Define a sub-area in the KeyboardFocusZone, which
           if clicked calculates the relative row/column
           that was clicked on. Send a UIEvent::CellClick with
           the row/offset.
    - Phase value visualizes the play position
    - Automatically focus scroll to the cursor or the play line,
      depending on the mode

TrackerData
    - Strings are stored in the TrackerData
    - Length in Lines is predefined.

DSP:
    - Phase input
    - Play mode: Loop / PingPong / Oneshot
    - Reset input
    - Reverse setting
    - Max row number input (in relation to the PatternData row count).
    - At least: Note/Gate and 2 Value columns
        - have to check how many fit into the UI
    - Outputs:
        - Pitches
        - Gates
        - Signals
        - End-Gate that is high if the sequence is done
            - Alternatively triggers only shortly when the Loop restarts/ends
*/

#[derive(Debug)]
pub struct PatternEditor {
    rows:       usize,
    columns:    usize,
}

impl PatternEditor {
    pub fn new_ref(columns: usize, rows: usize) -> Rc<Self> {
        Rc::new(Self { rows, columns })
    }
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
}

#[derive(Debug)]
pub struct PatternEditorData {
    pattern:       Rc<RefCell<dyn UIPatternModel>>,
    cursor:        (usize, usize),
    enter_mode:    EnterMode,
    edit_step:     usize,
}

impl PatternEditorData {
    pub fn new(pattern: Rc<RefCell<dyn UIPatternModel>>) -> Box<dyn std::any::Any> {
        Box::new(Self {
            pattern,
            cursor: (1, 2),
            enter_mode: EnterMode::None,
            edit_step: 4,
        })
    }
}

impl WidgetType for PatternEditor {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData,
            p: &mut dyn Painter, pos: Rect)
    {
        let id        = data.id();
        let highlight = ui.hl_style_for(id, None);

        data.with(|data: &mut PatternEditorData| {
            let mut pat = data.pattern.borrow_mut();

            let border_color =
                match highlight {
                    HLStyle::Hover(_) => {
                        if data.enter_mode != EnterMode::None {
                            UI_TRK_BORDER_EDIT_CLR
                        } else {
                            UI_TRK_BORDER_HOVER_CLR
                        }
                    },
                    _ => {
                        data.enter_mode = EnterMode::None;
                        UI_TRK_BORDER_CLR
                    },
                };

            let pos =
                rect_border(p, UI_TRK_BORDER, border_color, UI_TRK_BG_CLR, pos);

            let mode_line =
                match data.enter_mode {
                    EnterMode::EnterValues(_) => {
                        Some("> Mode: [Enter Values]")
                    },
                    EnterMode::EditStep => {
                        Some("> Mode: [Set Edit Step]")
                    },
                    _ => None,
                };

            if let Some(mode_line) = mode_line {
                p.label_mono(
                    UI_TRK_FONT_SIZE,
                    -1,
                    UI_TRK_TEXT_CLR,
                    pos.x,
                    pos.y,
                    pos.w,
                    UI_TRK_ROW_HEIGHT,
                    &mode_line);
            }

            ui.define_active_zone(ActiveZone::new_keyboard_zone(id, pos));

            for ic in 0..self.columns {
                let x = (ic + 1) as f64 * UI_TRK_COL_WIDTH;
                let y = 2        as f64 * UI_TRK_ROW_HEIGHT;

                p.label_mono(
                    UI_TRK_FONT_SIZE,
                    0,
                    UI_TRK_TEXT_CLR,
                    pos.x + x,
                    pos.y + y,
                    UI_TRK_COL_WIDTH,
                    UI_TRK_ROW_HEIGHT,
                    if ic > 5 {
                        "Value"
                    } else if ic > 2 {
                        "Gate"
                    } else {
                        "Note"
                    });
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

            for ir in 0..self.rows {
                let y = (ir + 1) as f64 * UI_TRK_ROW_HEIGHT;
                if ir % data.edit_step == 0 {
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
                    &format!("{:-02}", ir));

                for ic in 0..self.columns {
                    let x = (ic + 1) as f64 * UI_TRK_COL_WIDTH;

                    let txt_clr =
                        if (ir, ic) == data.cursor {
                            p.rect_fill(
                                UI_TRK_CURSOR_BG_CLR,
                                pos.x + x,
                                pos.y + y,
                                UI_TRK_COL_WIDTH,
                                UI_TRK_ROW_HEIGHT);

                            UI_TRK_CURSOR_FG_CLR
                        } else {
                            UI_TRK_TEXT_CLR
                        };

                    if let Some(s) = pat.get_cell(ir, ic) {
                        p.label_mono(
                            UI_TRK_FONT_SIZE,
                            0,
                            txt_clr,
                            pos.x + x,
                            pos.y + y,
                            UI_TRK_COL_WIDTH,
                            UI_TRK_ROW_HEIGHT,
                            s);
                    } else {
                        p.label_mono(
                            UI_TRK_FONT_SIZE,
                            0,
                            txt_clr,
                            pos.x + x,
                            pos.y + y,
                            UI_TRK_COL_WIDTH,
                            UI_TRK_ROW_HEIGHT,
                            "---");
                    }
                }
            }

            for ic in 0..self.columns {
                let x = (ic + 1) as f64 * UI_TRK_COL_WIDTH;

                p.path_stroke(
                    1.0,
                    UI_TRK_COL_DIV_CLR,
                    &mut [
                        (pos.x + x + 0.5, pos.y),
                        (pos.x + x + 0.5, pos.y + pos.h),
                    ].iter().copied(),
                    false);
            }

            let _phase =
                if let Some(phase) = ui.atoms().get_phase_value(id) {
                    phase as f64
                } else { 0.0 };
        });
    }

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _avail: (f64, f64)) -> (f64, f64) {
        ((self.columns + 1) as f64 * UI_TRK_COL_WIDTH  + UI_TRK_BORDER * 2.0,
         (self.rows + 3)    as f64 * UI_TRK_ROW_HEIGHT + UI_TRK_BORDER * 2.0)
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        match ev {
            UIEvent::Click { id, index, .. } => {
                if *id == data.id() {
                    data.with(|_data: &mut PatternEditorData| {
                        // TODO => find cell!
                        println!("INDEX: {}", index);
                        ui.queue_redraw();
                    });
                }
            },
            UIEvent::Key { id, key } => {
                if *id == data.id() {
                    data.with(|data: &mut PatternEditorData| {
                        let mut pat = data.pattern.borrow_mut();

                        match key {
                            Key::ArrowUp => {
                                if data.cursor.0 > 0 {
                                    data.cursor.0 -= 1;
                                }
                            },
                            Key::ArrowDown => {
                                if (data.cursor.0 + 1) < pat.rows() {
                                    data.cursor.0 += 1;
                                }
                            },
                            Key::ArrowLeft => {
                                if data.cursor.1 > 0 {
                                    data.cursor.1 -= 1;
                                }
                            },
                            Key::ArrowRight => {
                                if (data.cursor.1 + 1) < pat.cols() {
                                    data.cursor.1 += 1;
                                }
                            },
                            Key::Character(c) => {
                                match data.enter_mode {
                                    EnterMode::EnterValues(v) => {
                                        let value =
                                            match &c[..] {
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
                                            };
                                        if let Some(value) = value {
                                            match v {
                                                EnterValue::None => {
                                                    let nv = value << 0x8;
                                                    data.enter_mode =
                                                        EnterMode::EnterValues(
                                                            EnterValue::One(nv));
                                                    pat.set_cell_value(
                                                        data.cursor.0,
                                                        data.cursor.1,
                                                        nv);
                                                },
                                                EnterValue::One(v) => {
                                                    let nv = v | (value << 0x4);
                                                    data.enter_mode =
                                                        EnterMode::EnterValues(
                                                            EnterValue::Two(nv));
                                                    pat.set_cell_value(
                                                        data.cursor.0,
                                                        data.cursor.1,
                                                        nv);
                                                },
                                                EnterValue::Two(v) => {
                                                    let nv = v | value;
                                                    data.enter_mode =
                                                        EnterMode::EnterValues(
                                                            EnterValue::None);
                                                    pat.set_cell_value(
                                                        data.cursor.0,
                                                        data.cursor.1,
                                                        nv);
                                                },
                                            }
                                            println!("VALUE: {}", value);
                                        }
                                    },
                                    _ => {},
                                }
                            },
                            Key::Escape => {
                                data.enter_mode = EnterMode::None;
                            },
                            Key::Enter => {
                                data.enter_mode =
                                    match data.enter_mode {
                                        EnterMode::EnterValues(_)
                                            => EnterMode::None,
                                        _   => EnterMode::EnterValues(EnterValue::None),
                                    }
                            },
                            _ => {},
                        }
                        ui.queue_redraw();
                        println!("PATTERN EDIT KEY: {:?}", key);
                    });
                }
            },
            _ => {},
        }
    }
}
