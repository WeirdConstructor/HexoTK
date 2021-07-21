use crate::AtomId;
use crate::{WindowUI, InputEvent, ActiveZone};

use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::time::Duration;

#[derive(Debug, Clone)]
enum DriverRequest {
    MoveMouse       { x: f64, y: f64 },
    Query,
}

#[derive(Debug, Clone)]
enum DriverReply {
    Ack,
    State {
        zones:      Vec<ActiveZone>,
        texts:      HashMap<(AtomId, usize), String>,
        hover:      Option<ActiveZone>,
        mouse_pos:  (f64, f64),
    },
}

pub struct DriverFrontend {
    rx: Receiver<DriverReply>,
    tx: Sender<DriverRequest>,

    pub zones:      Vec<ActiveZone>,
    pub texts:      HashMap<(AtomId, usize), String>,
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

//    pub fn get_texts(&self, id: AtomId) -> Vec<(usize, String)> {
//        let mut texts = vec![];
//
//        for ((aid, idx), s) in self.texts.iter() {
//            if *aid == id {
//                texts.push((*idx, s.to_string()));
//            }
//        }
//
//        texts
//    }
//
//    pub fn get_text_dump(&self)
//        -> Result<HashMap<(AtomId, usize), String>, DriverError>
//    {
//        request!{self
//            DriverRequest::QueryTextDump
//            => DriverReply::TextDump { dump }
//            => Ok(dump)
//        }
//    }
//
//    pub fn get_zone_pos(&self, id: AtomId, dbgid: usize)
//        -> Result<Rect, DriverError>
//    {
//        let zones = self.query_zones(id)?;
//        for z in zones.iter() {
//            if z.dbgid == dbgid {
//                return Ok(z.pos);
//            }
//        }
//
//        Err(DriverError::NotFound)
//    }
//
//    pub fn query_hover(&self)
//        -> Result<Option<ActiveZone>, DriverError>
//    {
//        request!{self
//            DriverRequest::QueryHover
//            => DriverReply::Hover { zone }
//            => Ok(zone)
//        }
//    }
//
//    pub fn query_zones(&self, id: AtomId)
//        -> Result<Vec<ActiveZone>, DriverError>
//    {
//        request!{self
//            DriverRequest::QueryZones { id }
//            => DriverReply::Zones { zones }
//            => Ok(zones)
//        }
//    }

    pub fn move_mouse(&self, x: f64, y: f64) -> Result<(), DriverError> {
        request!{self DriverRequest::MoveMouse { x, y } }
    }
}

pub struct Driver {
    texts:  HashMap<(AtomId, usize), String>,
    rx:     Receiver<DriverRequest>,
    tx:     Sender<DriverReply>,
}

impl Driver {
    pub fn new() -> (Self, DriverFrontend) {
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();
        (Self {
            texts: HashMap::new(),
            tx: tx1,
            rx: rx2,
        }, DriverFrontend {
            tx: tx2,
            rx: rx1,
            zones: vec![],
            texts: HashMap::new(),
            hover: None,
            mouse_pos: (0.0, 0.0),
        })
    }

    pub fn handle_request(&mut self, ui: &mut dyn WindowUI) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
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

    pub fn update_text(&mut self, id: AtomId, idx: usize, txt: &str) {
        let key = (id, idx);
        if let Some(s) = self.texts.get(&key) {
            if txt != s {
                self.texts.insert(key, txt.to_string());
            }
        } else {
            self.texts.insert(key, txt.to_string());
        }
    }
}
