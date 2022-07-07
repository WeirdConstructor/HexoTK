mod wichtext;
mod entry;
mod hexknob;
mod hexgrid;
mod connector;
mod octave_keys;
mod graph;
mod pattern_editor;

pub use wichtext::{WichText, WichTextData, WichTextSimpleDataStore};
pub use entry::{Entry, EditableText, TextField};
pub use hexknob::{ParamModel, DummyParamModel, HexKnob, ChangeRes};
pub use hexgrid::{HexGrid, HexGridModel, HexCell, HexDir, HexEdge, HexHLight};
pub use connector::{Connector, ConnectorData};
pub use octave_keys::{OctaveKeys, OctaveKeysModel, DummyOctaveKeysData};
pub use graph::{GraphModel, Graph, StaticGraphData};
pub use pattern_editor::{
    PatternEditor, PatternData, UIPatternModel,
    PatternEditorFeedback, PatternEditorFeedbackDummy
};

use keyboard_types::Key;

#[derive(Debug)]
pub struct ModifierTracker {
    pub ctrl:      bool,
    pub shift:     bool,
    pub mouse:     crate::rect::Rect,
}

impl ModifierTracker {
    pub fn new() -> Self {
        Self {
            ctrl:  false,
            shift: false,
            mouse: crate::rect::Rect::from(0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn handle(&mut self, event: &crate::InputEvent) {
        match event {
            crate::InputEvent::KeyPressed(key) => {
                match key.key {
                    Key::Shift   => { self.shift = true; },
                    Key::Control => { self.ctrl  = true; },
                    _ => {},
                }
            },
            crate::InputEvent::KeyReleased(key) => {
                match key.key {
                    Key::Shift   => { self.shift = false; },
                    Key::Control => { self.ctrl  = false; },
                    _ => {},
                }
            },
            crate::InputEvent::MousePosition(x, y) => {
                self.mouse.x = *x;
                self.mouse.y = *y;
            },
            _ => {},
        }
    }
}
