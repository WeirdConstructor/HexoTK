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

    std::thread::spawn({
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

        sub.reg("click", move |_wid, _ev| {
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

        sub2.reg("click", move |_wid, _ev| {
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

//        wid.change_layout(|layout| {
//        });

        sub.change_layout(|layout| {
            layout.width  = Units::Stretch(1.0);
            layout.height = Units::Pixels(100.0);
        });
        sub2.change_layout(|layout| {
            layout.width  = Units::Stretch(1.0);
            layout.left   = Units::Percentage(50.0);
            layout.height = Units::Pixels(100.0);
        });
        sub3.change_layout(|layout| {
            layout.width  = Units::Percentage(20.0);
            layout.left   = Units::Percentage(40.0);
            layout.right  = Units::Percentage(40.0);
            layout.height = Units::Pixels(100.0);
        });
        sub4.change_layout(|layout| {
            layout.width  = Units::Pixels(200.0);
            layout.left   = Units::Stretch(1.0);
            layout.height = Units::Pixels(100.0);
        });

        let layer2root = Widget::new(style_ref.clone());
        layer2root.enable_cache();
        layer2root.change_layout(|layout| {
            layout.layout_type = LayoutType::Row;
//            layout.child_top = Units::Percentage(75.0);
//            layout.child_top = Units::Percentage(49.0);
        });

        let mut wstyle = Style::new();
        wstyle.bg_color = hexotk::style::UI_ACCENT_BG1_CLR;
        wstyle.font_size = 15.0;

        let wtd = Rc::new(WichTextSimpleDataStore::new());
        let wt = WichText::new(wtd.clone());
        let wtwid = Widget::new(Rc::new(wstyle));
        wtwid.enable_cache();
        wtwid.set_ctrl(Control::WichText { wt: Box::new(wt) });

        wtd.set_text("WichText Widget is Back!\n\nLololol\n[c15:fiuewhfiu wiufhwei]\nfewfuwefewifw\n fuiei fwi wei fewi fwei\nfeiwgureirege\nfuweifuewifewiuf".to_string());

        let mut cnt = 0;
        sub4.reg("click", {
            let layer2root = layer2root.clone();
            let style_ref  = style_ref.clone();

            move |_wid, _ev| {
                cnt += 1;
                let btn1 = Widget::new(style_ref.clone());
                btn1.set_ctrl(Control::Button {
                    label: Box::new(format!("Sub Btn {}", cnt))
                });
                btn1.change_layout(|layout| {
                    layout.max_height = Units::Pixels(40.0);
                });
                layer2root.add(btn1.clone());
            }
        });

        let col = Widget::new(style_ref.clone());
        col.change_layout(|layout| {
            layout.layout_type = LayoutType::Column;
//            layout.width  = Units::Pixels(200.0);
//            layout.height = Units::Pixels(200.0);
        });

        let etf = TextField::new();
        let btn1 = Widget::new(style_ref.clone());
        btn1.set_ctrl(Control::Button {
            label: Box::new(format!("Lay2 Btn {}", cnt))
        });
        btn1.change_layout(|layout| {
            layout.max_height = Units::Pixels(40.0);
        });
        btn1.reg("click", {
            let mut counter = 0;
            let etf = etf.clone();
            move |_wid, _ev| {
                etf.set(format!("{} {}", etf.get(), counter));
                counter += 1;
            }
        });

        let entry = Widget::new(style_ref.clone());
        entry.set_ctrl(Control::Entry {
            entry: Box::new(Entry::new(Box::new(etf))),
        });
        entry.change_layout(|layout| layout.max_height = Units::Pixels(40.0));
        col.add(entry);
        col.add(btn1);
        layer2root.add(col);
//        layer2root.add(wtwid);

        let mut ui = Box::new(UI::new());
//        ui.add_layer_root(wid);
//        ui.add_layer_root(layer2root.clone());
//        ui.add_layer_root(col);

        let new_txt_btn = { let style_ref = style_ref.clone(); Box::new(move |txt: &str| {
            let btn1 = Widget::new(style_ref.clone());
            btn1.set_ctrl(Control::Button { label: Box::new(txt.to_string()) });
            btn1
        }) };

        let row = Widget::new(style_ref.clone());
        row.change_layout(|layout| {
            layout.layout_type = LayoutType::Row;
        });

        let col = Widget::new(style_ref.clone());
        col.change_layout(|layout| {
            layout.layout_type = LayoutType::Column;
            layout.width = Units::Percentage(50.0);
        });

        let btn2 = new_txt_btn("Text 2");
        let btn3 = new_txt_btn("Text 2");

        let btn1 = new_txt_btn("Text 1");
        btn1.change_layout(|layout| {
            layout.width = Units::Percentage(50.0);
        });

        col.add(btn2.clone());
        col.add(btn3.clone());

        row.add(col.clone());
        row.add(btn1.clone());
        ui.add_layer_root(row.clone());


        println!("row : {}", row.id());
        println!("col : {}", col.id());
        println!("btn1: {}", btn1.id());
        println!("btn2: {}", btn2.id());
        println!("btn3: {}", btn3.id());

        ui
    }));
}
