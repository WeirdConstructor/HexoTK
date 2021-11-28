use hexotk::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

const WINDOW_W : i32 = 720;
const WINDOW_H : i32 = 720;

fn main() {
    let concurrent_data =
        Arc::new(Mutex::new(CloneMutable::new(
            ("Count:".to_string(), 0))));

    let t = std::thread::spawn({
        let data = concurrent_data.clone();
        move || {
            loop {
                if let Ok(mut data) = data.lock() {
                    (*data).1 += 1;
                }
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
        }
    });

    open_window("HexoTK 0.5 Demo", WINDOW_W, WINDOW_H, None, Box::new(|| {
        let mut style = Style::new();
        style.font_size = 40.0;

        let style_ref = Rc::new(style);

        let wid = Widget::new(style_ref.clone());
        wid.set_direct_ctrl(
            Control::None, Rect::from(0.0, 0.0, 400.0, 400.0));

        let sub = Widget::new(style_ref.clone());
        wid.add(sub.clone());

        sub.set_direct_ctrl(
            Control::Button { label: Box::new(concurrent_data.clone()) },
            Rect::from(10.0, 20.0, 300.0, 200.0));

        sub.reg("click", Box::new(move |wid, ev| {
            if let Ok(mut data) = concurrent_data.lock() {
                (*data).1 += 1;
            }
            println!("Button clicked!");
        }));

        let sub2 = Widget::new(style_ref);
        wid.add(sub2.clone());

        let data =
            Rc::new(RefCell::new(
                CloneMutable::new(("Other:".to_string(), 0))));

        sub2.set_direct_ctrl(
            Control::Button { label: Box::new(data.clone()) },
            Rect::from(360.0, 50.0, 200.0, 50.0));

        sub2.reg("click", Box::new(move |wid, ev| {
            (*data.borrow_mut()).1 += 1;
            println!("Button clicked!");
        }));

        let mut ui = Box::new(UI::new());
        ui.set_root(wid);
        ui
    }));
}
