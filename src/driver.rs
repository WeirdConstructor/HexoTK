use crate::AtomId;

use std::collections::HashMap;

pub struct Driver {
    texts:      HashMap<(AtomId, usize), String>,
}

impl Driver {
    pub fn new() -> Self {
        Self {
            texts: HashMap::new(),
        }
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
