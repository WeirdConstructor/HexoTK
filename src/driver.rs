use crate::AtomId;
use crate::{WindowUI, InputEvent, ActiveZone};
use crate::Rect;

use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::time::Duration;

#[derive(Debug, Clone)]
enum DriverRequest {
    MoveMouse       { x: f64, y: f64 },
    BeQuiet,
    Exit,
    Query,
}

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
}

pub struct Driver {
    texts:  HashMap<(AtomId, usize), (String, Rect)>,
    rx:     Receiver<DriverRequest>,
    tx:     Sender<DriverReply>,
    inhibit_frame_time: bool,
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
        }, DriverFrontend {
            tx: tx2,
            rx: rx1,
            zones: vec![],
            texts: HashMap::new(),
            hover: None,
            mouse_pos: (0.0, 0.0),
        })
    }

    pub fn be_quiet(&self) -> bool {
        self.inhibit_frame_time
    }

    pub fn handle_request(&mut self, ui: &mut dyn WindowUI) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                DriverRequest::Exit => {
                    std::process::exit(0);
                },
                DriverRequest::BeQuiet => {
                    self.inhibit_frame_time = true;
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
                DriverRequest::MoveMouse { x, y } => {
                    ui.handle_input_event(
                        InputEvent::MousePosition(x, y));
                    let _ = self.tx.send(DriverReply::Ack);
                },
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
