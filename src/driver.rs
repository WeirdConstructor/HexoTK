use crate::AtomId;

use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::time::Duration;

#[derive(Debug, Clone)]
enum DriverRequest {
    QueryText { id: AtomId, idx: usize },
}

#[derive(Debug, Clone)]
enum DriverReply {
    Text { text: Option<String> },
}

pub struct DriverFrontend {
    rx: Receiver<DriverReply>,
    tx: Sender<DriverRequest>,
}

#[derive(Debug, Copy, Clone)]
pub enum DriverError {
    Timeout,
    BadReply,
}

impl DriverFrontend {
    pub fn get_text(&self, id: AtomId, idx: usize) -> Result<Option<String>, DriverError> {
        self.tx.send(DriverRequest::QueryText { id, idx });
        match self.rx.recv_timeout(Duration::from_millis(1000)) {
            Ok(DriverReply::Text { text })
                    => Ok(text),
            Err(e)  => Err(DriverError::Timeout),
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

    pub fn handle_request(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                DriverRequest::QueryText { id, idx } => {
                    self.tx.send(DriverReply::Text {
                        text: self.texts.get(&(id, idx)).cloned()
                    });
                }
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
