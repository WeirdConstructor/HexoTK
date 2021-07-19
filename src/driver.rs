use crate::AtomId;
use crate::{WindowUI, InputEvent, ActiveZone, Rect};

use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::time::Duration;

#[derive(Debug, Clone)]
enum DriverRequest {
    MoveMouse  { x: f64, y: f64 },
    QueryZones { id: AtomId },
    QueryText  { id: AtomId, idx: usize },
    QueryTexts { id: AtomId },
    QueryTextDump,
}

#[derive(Debug, Clone)]
enum DriverReply {
    Ack,
    Zones    { zones: Vec<ActiveZone> },
    Text     { text:  Option<String> },
    Texts    { texts: Vec<(usize, String)> },
    TextDump { dump:  HashMap<(AtomId, usize), String> },
}

pub struct DriverFrontend {
    rx: Receiver<DriverReply>,
    tx: Sender<DriverRequest>,
}

#[derive(Debug, Copy, Clone)]
pub enum DriverError {
    Timeout,
    BadReply,
    NotFound,
}

impl DriverFrontend {
    pub fn get_text(
        &self, id: AtomId, idx: usize
    ) -> Result<String, DriverError>
    {
        let _ = self.tx.send(DriverRequest::QueryText { id, idx });
        match self.rx.recv_timeout(Duration::from_millis(1000)) {
            Ok(DriverReply::Text { text })
                    => if let Some(text) = text { Ok(text) }
                       else { Err(DriverError::NotFound) },
            Err(_)  => Err(DriverError::Timeout),
            _       => Err(DriverError::BadReply),
        }
    }

    pub fn get_texts(
        &self, id: AtomId
    ) -> Result<Vec<(usize, String)>, DriverError>
    {
        let _ = self.tx.send(DriverRequest::QueryTexts { id });
        match self.rx.recv_timeout(Duration::from_millis(1000)) {
            Ok(DriverReply::Texts { texts })
                    => Ok(texts),
            Err(_)  => Err(DriverError::Timeout),
            _       => Err(DriverError::BadReply),
        }
    }

    pub fn get_text_dump(&self)
        -> Result<HashMap<(AtomId, usize), String>, DriverError>
    {
        let _ = self.tx.send(DriverRequest::QueryTextDump);
        match self.rx.recv_timeout(Duration::from_millis(1000)) {
            Ok(DriverReply::TextDump { dump })
                    => Ok(dump),
            Err(_)  => Err(DriverError::Timeout),
            _       => Err(DriverError::BadReply),
        }
    }

    pub fn get_zone_pos(&self, id: AtomId, dbgid: usize)
        -> Result<Rect, DriverError>
    {
        let zones = self.query_zones(id)?;
        for z in zones.iter() {
            if z.dbgid == dbgid {
                return Ok(z.pos);
            }
        }

        Err(DriverError::NotFound)
    }

    pub fn query_zones(&self, id: AtomId)
        -> Result<Vec<ActiveZone>, DriverError>
    {
        let _ = self.tx.send(DriverRequest::QueryZones { id });
        match self.rx.recv_timeout(Duration::from_millis(1000)) {
            Ok(DriverReply::Zones { zones })
                    => Ok(zones),
            Err(_)  => Err(DriverError::Timeout),
            _       => Err(DriverError::BadReply),
        }
    }

    pub fn move_mouse(&self, x: f64, y: f64) -> Result<(), DriverError> {
        let _ = self.tx.send(DriverRequest::MoveMouse { x, y });
        match self.rx.recv_timeout(Duration::from_millis(1000)) {
            Ok(DriverReply::Ack) => Ok(()),
            Err(_)  => Err(DriverError::Timeout),
            _       => Err(DriverError::BadReply),
        }
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
        })
    }

    pub fn handle_request(&mut self, ui: &mut dyn WindowUI) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                DriverRequest::QueryZones { id } => {
                    let _ = self.tx.send(DriverReply::Zones {
                        zones: ui.query_active_zones(id),
                    });
                },
                DriverRequest::QueryText { id, idx } => {
                    let _ = self.tx.send(DriverReply::Text {
                        text: self.texts.get(&(id, idx)).cloned()
                    });
                },
                DriverRequest::QueryTexts { id } => {
                    let mut texts = vec![];
                    for ((aid, idx), s) in self.texts.iter() {
                        if *aid == id {
                            texts.push((*idx, s.to_string()));
                        }
                    }
                    let _ = self.tx.send(DriverReply::Texts {
                        texts
                    });
                },
                DriverRequest::QueryTextDump => {
                    let _ = self.tx.send(DriverReply::TextDump {
                        dump: self.texts.clone()
                    });
                },
                DriverRequest::MoveMouse { x, y } => {
                    ui.handle_input_event(
                        InputEvent::MousePosition(x, y));
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
