// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use super::*;

use std::rc::Rc;
use std::cell::RefCell;

pub struct DialogModel {
    text:   Rc<TextSourceRef>,
    on_ok:  Option<Box<dyn FnMut(&mut dyn AtomDataModel)>>,
    visible: bool,
}

#[allow(clippy::new_ret_no_self)]
impl DialogModel {
    pub fn new() -> Self {
        Self {
            text:   Rc::new(TextSourceRef::new(94)),
            on_ok:  None,
            visible: false,
        }
    }

    pub fn open(&mut self, text: &str,
                on_ok: Box<dyn FnMut(&mut dyn AtomDataModel)>)
    {
        self.text.set(text);
        self.on_ok = Some(on_ok);
        self.visible = true;
    }

    pub fn close(&mut self) {
    }
}

impl Default for DialogModel { fn default() -> Self { Self::new() } }

#[derive(Debug)]
pub struct Dialog {
}

impl Dialog {
    pub fn new() -> Self {
        Self { }
    }
}

impl Default for Dialog { fn default() -> Self { Self::new() } }

pub struct DialogData {
    model:          Rc<RefCell<DialogModel>>,
    cont:           WidgetData,
    ok_btn_id:      AtomId,
    last_click_seq: i64,
}

impl DialogData {
    pub fn new(node_id: u32, ok_btn_id: AtomId, model: Rc<RefCell<DialogModel>>) -> Box<Self> {
        let wt_cont = Rc::new(Container::new());
        let wt_txt  = Rc::new(Text::new(12.0));
        let wt_btn  = Rc::new(Button::new(80.0, 12.0));

        let mut cont = ContainerData::new();
        cont.contrast_border()
            .level(2)
            .new_row()
            .add(wbox!(
                wt_txt,
                AtomId::new(node_id, 1),
                center(12, 10),
                TextData::new(model.borrow().text.clone())))
            .new_row()
            .add(wbox!(
                wt_btn, ok_btn_id, center(12, 2),
                ButtonData::new_setting_inc("Ok")));

        let cont =
            wbox!(wt_cont, AtomId::new(node_id, 2), center(12, 12), cont);

        Box::new(Self { model, cont, ok_btn_id, last_click_seq: 0 })
    }
}

impl WidgetType for Dialog {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        data.with(|data: &mut DialogData| {
            let btn_seq =
                if let Some(at) = ui.atoms().get(data.ok_btn_id) {
                    at.i()
                } else { 0 };

            if data.last_click_seq != btn_seq {
                data.model.borrow_mut().visible = false;
                if let Some(on_ok) = &mut data.model.borrow_mut().on_ok {
                    (on_ok)(ui.atoms_mut());
                }
            }

            data.last_click_seq = btn_seq;

            if data.model.borrow_mut().visible {
                data.cont.draw(ui, p, pos);
            }
        });
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        data.with(|data: &mut DialogData| {
            data.cont.event(ui, ev);
        });
    }
}
