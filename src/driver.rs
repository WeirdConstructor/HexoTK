use crate::AtomId;
use crate::{WindowUI, InputEvent, MButton, ActiveZone};
use crate::Rect;

use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use keyboard_types::Key;

#[derive(Debug, Clone)]
pub enum DriverRequest {
    MoveMouse       { x: f64, y: f64 },
    MouseDown       { btn: MButton },
    MouseUp         { btn: MButton },
    KeyDown         { key: Key },
    KeyUp           { key: Key },
    BeQuiet,
    Exit,
    Query,
    ClearStateForQuery,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
enum DriverReply {
    Ack,
    State {
        zones:      Vec<ActiveZone>,
        texts:      HashMap<(AtomId, usize), (String, Rect)>,
        hover:      Option<ActiveZone>,
        mouse_pos:  (f64, f64),
    },
}

pub struct DriverFrontend {
    rx: Receiver<DriverReply>,
    tx: Sender<DriverRequest>,

    pub zones:      Vec<ActiveZone>,
    pub texts:      HashMap<(AtomId, usize), (String, Rect)>,
    pub hover:      Option<ActiveZone>,
    pub mouse_pos:  (f64, f64),
}

#[derive(Debug, Copy, Clone)]
pub enum DriverError {
    Timeout,
    BadReply,
    NotFound,
}

macro_rules! request {
    ($self: ident
        $sid: ident :: $send: tt $sb: tt
        => $mid: ident :: $match: tt $mb: tt
        => $arm: expr
    ) => {
        let _ = $self.tx.send($sid::$send $sb);
        match $self.rx.recv_timeout(Duration::from_millis(1000)) {
            Ok($mid::$match $mb) => $arm,
            Err(_)     => Err(DriverError::Timeout),
            _          => Err(DriverError::BadReply),
        }
    };

    ($self: ident
        $sid: ident :: $send: tt
        => $mid: ident :: $match: tt $mb: tt
        => $arm: expr
    ) => {
        let _ = $self.tx.send($sid::$send);
        match $self.rx.recv_timeout(Duration::from_millis(1000)) {
            Ok($mid::$match $mb) => $arm,
            Err(_)     => Err(DriverError::Timeout),
            _          => Err(DriverError::BadReply),
        }
    };

    ($self: ident $sid: ident :: $send: tt $sb: tt) => {
        let _ = $self.tx.send($sid::$send $sb);
        match $self.rx.recv_timeout(Duration::from_millis(1000)) {
            Ok(DriverReply::Ack) => Ok(()),
            Err(_)     => Err(DriverError::Timeout),
            _          => Err(DriverError::BadReply),
        }
    };

    ($self: ident $sid: ident :: $send: tt) => {
        let _ = $self.tx.send($sid::$send);
        match $self.rx.recv_timeout(Duration::from_millis(1000)) {
            Ok(DriverReply::Ack) => Ok(()),
            Err(_)     => Err(DriverError::Timeout),
            _          => Err(DriverError::BadReply),
        }
    }
}

impl DriverFrontend {
    pub fn query_state(&mut self) -> Result<(), DriverError> {
        let _ = { request!{self DriverRequest::ClearStateForQuery} };

        request!{self
            DriverRequest::Query
            => DriverReply::State { zones, texts, hover, mouse_pos }
            => {
                self.zones      = zones;
                self.texts      = texts;
                self.hover      = hover;
                self.mouse_pos  = mouse_pos;
                Ok(())
            }
        }
    }

    pub fn be_quiet(&mut self) {
        let _ = { request!{self DriverRequest::BeQuiet} };
    }

    pub fn exit(&mut self) {
        let _ = { request!{self DriverRequest::Exit} };
    }

    pub fn move_mouse(&self, x: f64, y: f64) -> Result<(), DriverError> {
        request!{self DriverRequest::MoveMouse { x, y } }
    }

    pub fn mouse_down(&self, btn: MButton) -> Result<(), DriverError> {
        request!{self DriverRequest::MouseDown { btn } }
    }

    pub fn mouse_up(&self, btn: MButton) -> Result<(), DriverError> {
        request!{self DriverRequest::MouseUp { btn } }
    }

    pub fn key_down(&self, key: Key) -> Result<(), DriverError> {
        request!{self DriverRequest::KeyDown { key } }
    }

    pub fn key_up(&self, key: Key) -> Result<(), DriverError> {
        request!{self DriverRequest::KeyUp { key } }
    }
}

pub struct Driver {
    texts:              HashMap<(AtomId, usize), (String, Rect)>,
    rx:                 Receiver<DriverRequest>,
    tx:                 Sender<DriverReply>,
    sync_queue:         Option<Arc<Mutex<Vec<DriverRequest>>>>,
    inhibit_frame_time: bool,
    has_control:        bool,
}

fn handle_ui_command(msg: DriverRequest, ui: &mut dyn WindowUI) -> bool {
    match msg {
        DriverRequest::Exit => {
            std::process::exit(0);
        },
        DriverRequest::MoveMouse { x, y } => {
            ui.handle_input_event(
                InputEvent::MousePosition(x, y));
        },
        DriverRequest::MouseDown { btn } => {
            ui.handle_input_event(
                InputEvent::MouseButtonPressed(btn));
        },
        DriverRequest::MouseUp { btn } => {
            ui.handle_input_event(
                InputEvent::MouseButtonReleased(btn));
        },
        DriverRequest::KeyDown { key } => {
            use keyboard_types::{
                KeyboardEvent, Code, Location,
                Modifiers, KeyState
            };

            ui.handle_input_event(
                InputEvent::KeyPressed(KeyboardEvent {
                    key,
                    // XXX: the rest is not used by HexoTK!
                    state:          KeyState::Down,
                    code:           Code::Escape,
                    location:       Location::Standard,
                    modifiers:      Modifiers::empty(),
                    repeat:         false,
                    is_composing:   false,
                }));
        },
        DriverRequest::KeyUp { key } => {
            use keyboard_types::{
                KeyboardEvent, Code, Location,
                Modifiers, KeyState
            };

            ui.handle_input_event(
                InputEvent::KeyReleased(KeyboardEvent {
                    key,
                    // XXX: the rest is not used by HexoTK!
                    state:          KeyState::Up,
                    code:           Code::Escape,
                    location:       Location::Standard,
                    modifiers:      Modifiers::empty(),
                    repeat:         false,
                    is_composing:   false,
                }));
        },
        _ => { return false; }
    }

    true
}

impl Driver {
    pub fn new() -> (Self, DriverFrontend) {
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();
        (Self {
            texts: HashMap::new(),
            tx: tx1,
            rx: rx2,
            inhibit_frame_time: false,
            has_control:        false,
            sync_queue: None,
        }, DriverFrontend {
            tx: tx2,
            rx: rx1,
            zones: vec![],
            texts: HashMap::new(),
            hover: None,
            mouse_pos: (0.0, 0.0),
        })
    }

    pub fn take_control(&mut self) {
        self.has_control = true;
    }

    pub fn has_control(&self) -> bool {
        self.has_control
    }

    pub fn be_quiet(&self) -> bool {
        self.inhibit_frame_time
    }

    pub fn set_sync_queue(&mut self, queue: Arc<Mutex<Vec<DriverRequest>>>) {
        self.sync_queue = Some(queue);
    }

    pub fn handle_request(&mut self, ui: &mut dyn WindowUI) {
        if let Some(sync_queue) = self.sync_queue.as_mut() {
            if let Ok(mut queue) = sync_queue.lock() {
                while let Some(msg) = queue.pop() {
                    handle_ui_command(msg, ui);
                }
            }
        }

        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                DriverRequest::BeQuiet => {
                    self.inhibit_frame_time = true;
                    let _ = self.tx.send(DriverReply::Ack);
                },
                DriverRequest::ClearStateForQuery => {
                    self.texts.clear();
                    let _ = self.tx.send(DriverReply::Ack);
                },
                DriverRequest::Query => {
                    let state = ui.query_state();
                    let _ = self.tx.send(DriverReply::State {
                        zones:      state.zones,
                        texts:      self.texts.clone(),
                        hover:      state.hover,
                        mouse_pos:  state.mouse_pos,
                    });
                },
                _ => {
                    if handle_ui_command(msg, ui) {
                        let _ = self.tx.send(DriverReply::Ack);
                    }
                }
            }
        }
    }

    pub fn clear_texts(&mut self) {
        self.texts.clear();
    }

    pub fn update_text(&mut self, id: AtomId, idx: usize, txt: &str, pos: Rect) {
        let key = (id, idx);
        if let Some(s) = self.texts.get(&key) {
            if txt != s.0 {
                self.texts.insert(key, (txt.to_string(), pos));
            }
        } else {
            self.texts.insert(key, (txt.to_string(), pos));
        }
    }
}
