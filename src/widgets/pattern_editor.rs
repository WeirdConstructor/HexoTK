// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use crate::constants::*;
use super::*;
use super::util::*;

//pub struct PatternData {
//    rows:   
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
    - Clock/Phase input (each pulse advances a line)
        - switchable by a mode setting
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

#[derive(Debug)]
pub struct PatternEditorData {
    pattern_index: usize,
}

impl PatternEditorData {
    pub fn new() -> Box<dyn std::any::Any> {
        Box::new(Self {
            pattern_index: 0,
        })
    }
}

impl WidgetType for PatternEditor {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData,
            p: &mut dyn Painter, pos: Rect)
    {
        let id        = data.id();
        let highlight = ui.hl_style_for(id, None);

        let border_color =
            match highlight {
                HLStyle::Hover(_) => UI_TRK_BORDER_HOVER_CLR,
                _                 => UI_TRK_BORDER_CLR,
            };

        let pos =
            rect_border(p, UI_TRK_BORDER, border_color, UI_TRK_BG_CLR, pos);

        data.with(|data: &mut PatternEditorData| {
            for ir in 0..self.rows {
                for ic in 0..self.columns {
                    let x = ic as f64 * UI_TRK_COL_WIDTH + UI_TRK_COL_DIV_PAD;
                    let y = ir as f64 * UI_TRK_ROW_HEIGHT;

                    p.label_mono(
                        UI_TRK_FONT_SIZE,
                        -1,
                        UI_TRK_TEXT_CLR,
                        pos.x + x,
                        pos.y + y,
                        UI_TRK_COL_WIDTH,
                        UI_TRK_ROW_HEIGHT,
                        "0.12");
                }
            }

            for ic in 1..self.columns {
                let x = ic as f64 * UI_TRK_COL_WIDTH + UI_TRK_COL_DIV_PAD;
                let x = x - UI_TRK_COL_DIV_PAD;

                p.path_stroke(
                    1.0,
                    UI_TRK_COL_DIV_CLR,
                    &mut [
                        (pos.x + x + 0.5, pos.y),
                        (pos.x + x + 0.5, pos.y + pos.h),
                    ].iter().copied(),
                    false);
            }

            let phase =
                if let Some(phase) = ui.atoms().get_phase_value(id) {
                    phase as f64
                } else { 0.0 };
        });
    }

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _avail: (f64, f64)) -> (f64, f64) {
        (self.columns as f64 * UI_TRK_COL_WIDTH  + UI_TRK_BORDER * 2.0,
         self.rows    as f64 * UI_TRK_ROW_HEIGHT + UI_TRK_BORDER * 2.0)
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        match ev {
            UIEvent::Click { id, index, .. } => {
                if *id == data.id() {
                    data.with(|data: &mut PatternEditorData| {
                        // TODO => find cell!
                        ui.queue_redraw();
                    });
                }
            },
            // UIEvent::Key
            _ => {},
        }
    }
}
