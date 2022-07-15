mod connector;
mod entry;
mod graph;
mod graph_minmax;
mod hexgrid;
mod hexknob;
mod octave_keys;
mod pattern_editor;
mod wichtext;

pub use connector::{Connector, ConnectorData};
pub use entry::{EditableText, Entry, TextField};
pub use graph::{Graph, GraphModel, StaticGraphData};
pub use graph_minmax::{GraphMinMax, GraphMinMaxModel, StaticGraphMinMaxData};
pub use hexgrid::{HexCell, HexDir, HexEdge, HexGrid, HexGridModel, HexHLight};
pub use hexknob::{ChangeRes, DummyParamModel, HexKnob, ParamModel};
pub use octave_keys::{DummyOctaveKeysData, OctaveKeys, OctaveKeysModel};
pub use pattern_editor::{
    PatternData, PatternEditor, PatternEditorFeedback, PatternEditorFeedbackDummy, UIPatternModel,
};
pub use wichtext::{WichText, WichTextData, WichTextSimpleDataStore};

use keyboard_types::Key;

#[derive(Debug)]
pub struct ModifierTracker {
    pub ctrl: bool,
    pub shift: bool,
    pub mouse: crate::rect::Rect,
}

impl ModifierTracker {
    pub fn new() -> Self {
        Self { ctrl: false, shift: false, mouse: crate::rect::Rect::from(0.0, 0.0, 0.0, 0.0) }
    }

    pub fn handle(&mut self, event: &crate::InputEvent) {
        match event {
            crate::InputEvent::KeyPressed(key) => match key.key {
                Key::Shift => {
                    self.shift = true;
                }
                Key::Control => {
                    self.ctrl = true;
                }
                _ => {}
            },
            crate::InputEvent::KeyReleased(key) => match key.key {
                Key::Shift => {
                    self.shift = false;
                }
                Key::Control => {
                    self.ctrl = false;
                }
                _ => {}
            },
            crate::InputEvent::MousePosition(x, y) => {
                self.mouse.x = *x;
                self.mouse.y = *y;
            }
            _ => {}
        }
    }
}
