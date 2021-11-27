use hexotk::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

const WINDOW_W : i32 = 1150 + 360;
const WINDOW_H : i32 = 720;

fn main() {
    open_window("HexoTK 0.5 Demo", WINDOW_W, WINDOW_H, None, Box::new(|| {
        let mut ui = Box::new(UI::new());
        let wid = Widget::new_ref();
        wid.borrow_mut().set_direct_ctrl(
            Box::new(Control::Rect),
            Rect::from(10.0, 20.0, 300.0, 200.0));
        ui.set_root(wid);
        ui
    }));
}
