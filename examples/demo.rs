use hexotk::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

const WINDOW_W : i32 = 1020;
const WINDOW_H : i32 = 720;

pub struct TestGrid {
}

impl HexGridModel for TestGrid {
    fn width(&self) -> usize { 10 }
    fn height(&self) -> usize { 10 }
    fn cell_visible(&self, _x: usize, _y: usize) -> bool { true }
    fn cell_empty(&self, _x: usize, _y: usize) -> bool { false }
    fn cell_color(&self, _x: usize, _y: usize) -> u8 { 0 }
    fn cell_label<'a>(&self, _x: usize, _y: usize, _out: &'a mut [u8])
        -> Option<HexCell<'a>> { None }
    fn cell_edge<'a>(&self, _x: usize, _y: usize, _edge: HexDir, _out: &'a mut [u8])
        -> Option<(&'a str, HexEdge)> { None }
//    fn cell_click(&mut self, x: usize, y: usize, btn: MButton) { }
//    fn cell_drag(&mut self, x: usize, y: usize, x2: usize, y2: usize, btn: MButton) { }
}

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

        let new_txt_btn = { let style_ref = style_ref.clone(); Box::new(move |txt: &str| {
            let btn1 = Widget::new(style_ref.clone());
            btn1.set_ctrl(Control::Button { label: Box::new(txt.to_string()) });
            btn1
        }) };

        let wid = Widget::new(style_ref.clone());

        let sub = Widget::new(style_ref.clone());
        sub.enable_cache();
        wid.add(sub.clone());

        sub.set_ctrl(
            Control::Button { label: Box::new(concurrent_data.clone()) });

        sub.reg("click", move |_ctx, _wid, _ev| {
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

        sub2.reg("click", move |_ctx, _wid, _ev| {
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
            layout.width  = Some(Units::Stretch(1.0));
            layout.height = Some(Units::Pixels(100.0));
        });
        sub2.change_layout(|layout| {
            layout.width  = Some(Units::Stretch(1.0));
            layout.left   = Some(Units::Percentage(50.0));
            layout.height = Some(Units::Pixels(100.0));
        });
        sub3.change_layout(|layout| {
            layout.width  = Some(Units::Percentage(20.0));
            layout.left   = Some(Units::Percentage(40.0));
            layout.right  = Some(Units::Percentage(40.0));
            layout.height = Some(Units::Pixels(100.0));
        });
        sub4.change_layout(|layout| {
            layout.width  = Some(Units::Pixels(200.0));
            layout.left   = Some(Units::Stretch(1.0));
            layout.height = Some(Units::Pixels(100.0));
        });

        let layer2root = Widget::new(style_ref.clone());
        layer2root.enable_cache();
        layer2root.change_layout(|layout| {
            layout.layout_type = Some(LayoutType::Row);
//            layout.child_top = Units::Percentage(75.0);
//            layout.child_top = Units::Percentage(49.0);
        });

        let mut wstyle = Style::new();
        wstyle.bg_color = hexotk::style::UI_ACCENT_BG1_CLR;
        wstyle.font_size = 15.0;
        wstyle.pad_left = 15.0;
        wstyle.pad_top = 15.0;

        let wtd = Rc::new(WichTextSimpleDataStore::new());
        wtd.set_data_source("XXX", Rc::new(vec![
            0.0, 0.1, 0.11, 0.2, 0.4, 0.1, 0.9, 0.8, 0.4, 0.0
        ]));
        let wt = WichText::new(wtd.clone());
        let wtwid = Widget::new(Rc::new(wstyle));
        wtwid.enable_cache();
        wtwid.set_ctrl(Control::WichText { wt: Box::new(wt) });

        wtwid.reg("click", |_ctx, _wid, ev| {
            match &ev.data {
                EvPayload::WichTextCommand { cmd, .. } => {
                    println!("CLICK ON: {:?}", cmd);
                },
                _ => {},
            }
        });

        wtd.set_text("WichText Widget is Back!\n\nLololol\n[c15:fiuewhfiu wiufhwei]\nfewfuwefewifw\n fuiei fwi wei fewi fwei\nfeiwgureirege\nfuweifuewifewiuf\nHere a value: [h40avK:Test] [a:Click Here!]\nAnd a graph: [h30gXXX:Graph 1]".to_string());

        let mut cnt = 0;
        sub4.reg("click", {
            let layer2root = layer2root.clone();
            let style_ref  = style_ref.clone();

            move |_ctx, _wid, _ev| {
                cnt += 1;
                let btn1 = Widget::new(style_ref.clone());
                btn1.set_ctrl(Control::Button {
                    label: Box::new(format!("Sub Btn {}", cnt))
                });
                btn1.change_layout(|layout| {
                    layout.max_height = Some(Units::Pixels(40.0));
                });
                layer2root.add(btn1.clone());
            }
        });

        let col = Widget::new(style_ref.clone());
        col.change_layout(|layout| {
            layout.layout_type = Some(LayoutType::Column);
//            layout.width  = Units::Pixels(200.0);
//            layout.height = Units::Pixels(200.0);
        });

        let lbl1 = Widget::new(
            style_ref.with_style_clone(|style| {
                style.border     = 0.0;
                style.pad_left   = 5.0;
                style.text_align = Align::Left;
            }));
        lbl1.set_ctrl(Control::Label {
            label: Box::new(format!("LBL Xyz:"))
        });
        lbl1.change_layout(|layout| {
            layout.max_height = Some(Units::Pixels(40.0));
        });

        let etf = TextField::new();
        let btn1 = Widget::new(style_ref.clone());
        btn1.set_ctrl(Control::Button {
            label: Box::new(format!("Lay2 Btn {}", cnt))
        });
        btn1.change_layout(|layout| {
            layout.max_height = Some(Units::Pixels(40.0));
        });
        btn1.reg("click", {
            let mut counter = 0;
            let etf = etf.clone();
            let lbl = lbl1.clone();
            let col = col.clone();
            move |_ctx, _wid, _ev| {
                if let Some(par) = lbl.parent() {
                    par.remove_child(lbl.clone());
                } else {
                    col.add(lbl.clone());
                }

                etf.set(format!("{} {}", etf.get(), counter));
                counter += 1;

                if counter > 5 {
                    col.remove_childs();
                }
            }
        });

        let knob = Widget::new(style_ref.clone());
        let param =
            Rc::new(RefCell::new(
                hexotk::DummyParamModel::new()));
        knob.set_ctrl(Control::HexKnob {
            knob: Box::new(HexKnob::new(param.clone())),
        });

        let hexmodel = Rc::new(RefCell::new(TestGrid { }));
        let hexgrid = Widget::new(style_ref.clone());
        hexgrid.set_ctrl(Control::HexGrid {
            grid: Box::new(HexGrid::new(hexmodel)),
        });
//        entry.change_layout(|layout| layout.max_height = Units::Pixels(40.0));

        let entry = Widget::new(style_ref.clone());
        entry.set_ctrl(Control::Entry {
            entry: Box::new(Entry::new(Box::new(etf))),
        });
        entry.change_layout(|layout| layout.max_height = Some(Units::Pixels(40.0)));
        col.add(entry);
        col.add(btn1);
        col.add(lbl1);
        col.add(knob);
        layer2root.add(col);
        layer2root.add(wtwid);
        layer2root.add(hexgrid);

        let mut ui = Box::new(UI::new(Rc::new(RefCell::new(1))));
//        ui.add_layer_root(wid);
        ui.add_layer_root(layer2root.clone());
//        ui.add_layer_root(col);


        ui
    }));
}
