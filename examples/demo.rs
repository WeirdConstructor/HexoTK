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
        style.font_size = 20.0;

        let style_ref = Rc::new(style);

        let wid = Widget::new(style_ref.clone());

        let sub = Widget::new(style_ref.clone());
        sub.enable_cache();
        wid.add(sub.clone());

        sub.set_ctrl(
            Control::Button { label: Box::new(concurrent_data.clone()) });

        sub.reg("click", move |wid, ev| {
            if let Ok(mut data) = concurrent_data.lock() {
                (*data).1 += 1;
            }
            println!("Button clicked!");
        });

        let sub2 = Widget::new(style_ref.clone());
        sub2.enable_cache();
        wid.add(sub2.clone());

        let data =
            Rc::new(RefCell::new(
                CloneMutable::new(("Other:".to_string(), 0))));

        sub2.set_ctrl(Control::Button { label: Box::new(data.clone()) });

        sub2.reg("click", move |wid, ev| {
            (*data.borrow_mut()).1 += 1;
            println!("Button clicked!");
        });

        let sub3 = Widget::new(style_ref.clone());
        wid.add(sub3.clone());

        let sub4 = Widget::new(style_ref.clone());
        wid.add(sub4.clone());

        sub3.set_ctrl(Control::Button { label: Box::new("Sub3".to_string()) });
        sub4.set_ctrl(Control::Button {
            label: Box::new("Add to Layer2".to_string())
        });

        wid.change_layout(|layout| {
            layout.layout_type = LayoutType::VBox;
        });
        sub.change_layout(|layout| {
            layout.width = Units::Perc(50.0);
            layout.height = Units::Perc(50.0);
//            layout.align = Align::Right;
//            layout.pad_left = Units::Perc(25.0);
//            layout.pad_top  = Units::Perc(50.0);
        });
        sub2.change_layout(|layout| {
//            layout.width     = Units::Perc(100.0);
//            layout.max_width = Units::Perc(25.0);
            layout.height    = Units::Perc(25.0);
            layout.align     = Align::Right;
        });
        sub3.change_layout(|layout| {
//            layout.width      = Units::Perc(100.0);
//            layout.max_width  = Units::Perc(50.0);
            layout.height     = Units::S(2.0);
//            layout.min_width  = Units::Px(50.0);
            layout.max_height = Units::Perc(50.0);
            layout.valign     = VAlign::Bottom;
            layout.align      = Align::Right;
        });
        sub4.change_layout(|layout| {
//            layout.width     = Units::S(1.0);
            layout.height    = Units::S(1.0);
//            layout.min_width = Units::Px(10.0);
//            layout.max_height = Units::Perc(70.0);
            layout.valign     = VAlign::Top;
        });

        let layer2root = Widget::new(style_ref.clone());
//        layer2root.enable_cache();
        layer2root.change_layout(|layout| {
            layout.layout_type = LayoutType::HBox;
//            layout.spacing    = Units::Px(2.0);
            layout.pad_bottom = Units::Px(100.0);
        });

        let mut cnt = 0;
        sub4.reg("click", {
            let layer2root = layer2root.clone();
            let style_ref  = style_ref.clone();

            move |wid, ev| {
                cnt += 1;
                let btn1 = Widget::new(style_ref.clone());
                btn1.set_ctrl(Control::Button {
                    label: Box::new(format!("Sub Btn {}", cnt))
                });
                btn1.change_layout(|layout| {
                    layout.width  = Units::S(1.0);
                    layout.height = Units::Perc(10.0);
                });
                layer2root.add(btn1.clone());
            }
        });

        let btn1 = Widget::new(style_ref.clone());
        btn1.set_ctrl(Control::Button {
            label: Box::new(format!("Lay2 Btn {}", cnt))
        });
        btn1.change_layout(|layout| {
            layout.width  = Units::S(1.0);
            layout.height = Units::Perc(10.0);
//                    layout.width  = Units::Perc(10.0);
//                    layout.height = Units::Perc(10.0);
        });
        layer2root.add(btn1.clone());

        let mut ui = Box::new(UI::new());
        ui.add_layer_root(wid);

        ui.add_layer_root(layer2root.clone());


        ui
    }));
}
