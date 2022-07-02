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
    fn cell_led(&self, _x: usize, _y: usize) -> Option<(f32, f32)> { Some((0.5, 0.2)) }
    fn cell_label<'a>(&self, _x: usize, _y: usize, _out: &'a mut [u8])
        -> Option<HexCell<'a>> { None }
    fn cell_edge<'a>(&self, _x: usize, _y: usize, _edge: HexDir)
        -> HexEdge { HexEdge::Arrow }
    fn cell_edge_label<'a>(&self, _x: usize, _y: usize, _edge: HexDir, _out: &'a mut [u8])
        -> Option<&'a str> { None }
    fn get_generation(&self) -> u64 { 1 }
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
//        style.border_style = BorderStyle::Hex { offset: 10.0 };
//        style.border = 4.0;
        style.border_style = BorderStyle::Bevel { corner_offsets: (5.0, 10.0, 20.0, 2.0) };
        style.shadow_offs = (-3.0, 3.0);

        let style_ref = Rc::new(style.clone());

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

        let ccdata = concurrent_data.clone();

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


        let sub4 = Widget::new(style_ref.with_style_clone(|style| {
            style.border_style = BorderStyle::Hex { offset: 10.0 };
        }));
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
            layout.layout_type = Some(LayoutType::Column);
//            layout.child_top = Units::Percentage(75.0);
//            layout.child_top = Units::Percentage(49.0);
        });

        let mut wstyle = Style::new();
        wstyle.bg_color = hexotk::style::UI_ACCENT_BG1_CLR;
        wstyle.font_size = 15.0;
        wstyle.pad_left = 5.0;
        wstyle.pad_top = 30.0;

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

        wtd.set_text("XXX\n[a:Click Here!]".to_string());

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
        lbl1.change_layout(|layout| {
            layout.top    = Some(Units::Pixels(4.0));
            layout.bottom = Some(Units::Pixels(4.0));
        });
        lbl1.set_ctrl(Control::Label {
            label: Box::new(format!("LBL Xyz:"))
        });
        lbl1.change_layout(|layout| {
            layout.max_height = Some(Units::Pixels(40.0));
        });

        let cont = Widget::new(style_ref.clone());
        cont.change_layout(|layout| {
            layout.layout_type = Some(LayoutType::Row);
        });

        let etf = TextField::new();
        let btn0 = Widget::new(style_ref.clone());
        btn0.enable_cache();
        btn0.set_ctrl(Control::Button {
            label: Box::new(ccdata.clone())
        });
        btn0.change_layout(|layout| {
            layout.top    = Some(Units::Pixels(4.0));
            layout.bottom = Some(Units::Pixels(4.0));
            layout.left   = Some(Units::Pixels(4.0));
        });
        btn0.reg("hover", |_ctx, _wid, _ev| {
            println!("HOVER BTN0");
        });
        layer2root.reg("hover", |_ctx, _wid, _ev| {
            println!("HOVER L2 ROOT");
        });



        let btn2 = Widget::new(style_ref.with_style_clone(|style| {
            style.border_style = BorderStyle::Hex { offset: 30.0 };
        }));
        btn2.change_layout(|layout| {
            layout.top    = Some(Units::Pixels(4.0));
            layout.bottom = Some(Units::Pixels(4.0));
            layout.left   = Some(Units::Pixels(4.0));
        });
        btn2.enable_cache();
        btn2.set_ctrl(Control::Button {
            label: Box::new(ccdata.clone())
        });

        let dw = Widget::new(style_ref.with_style_clone(|style| {
            style.bg_color = hexotk::style::UI_ACCENT_BG1_CLR;
            style.border_style = BorderStyle::Rect;
        }));
//        dw.set_ctrl(Control::Rect);
        dw.change_layout(|layout| {
            layout.position_type = Some(PositionType::SelfDirected);
            layout.layout_type   = Some(LayoutType::Column);
            layout.width         = Some(Units::Auto);
            layout.height        = Some(Units::Auto);
            layout.visible       = false;
        });

        let dw_style = style_ref.with_style_clone(|style| {
            style.border_style = BorderStyle::Bevel {
                corner_offsets: (10.0, 0.0, 0.0, 0.0),
            };
        });

        let dw_x1 = Widget::new(dw_style.clone());
        let dw_x2 = Widget::new(dw_style.clone());
        let dw_x3 = Widget::new(dw_style.clone());
        dw_x1.set_ctrl(crate::Control::Button { label: Box::new("Test1".to_string()) });
        dw_x2.set_ctrl(crate::Control::Button { label: Box::new("Test ABC".to_string()) });
        dw_x3.set_ctrl(crate::Control::Button { label: Box::new("Super Synth Hexo".to_string()) });

        let dw_x_layout = |layout: &mut hexotk::Layout| {
            layout.width  = Some(Units::Pixels(200.0));
            layout.height = Some(Units::Pixels(50.0));
        };

        for dw_x in [dw_x1, dw_x2, dw_x3] {
            dw_x.change_layout(dw_x_layout);
            dw_x.reg("click", move |_ctx, wid, _ev| println!("Test Click"));
            dw.add(dw_x);
        }

        dw.auto_hide();

        btn2.reg("click", {
            let dw = dw.clone();
            move |_ctx, _wid, ev| { dw.popup_at(PopupPos::MousePos); }
        });

        let drag_wid = Widget::new(style_ref.clone());
//        drag_wid.change_layout(|layout| {
//            layout.visible = false;
//        });
        drag_wid.set_ctrl(crate::Control::Button { label: Box::new(ccdata) });
        drag_wid.set_pos(Rect { x: 0.0, y: 0.0, w: 130.0, h: 50.0 });
        btn2.set_drag_widget(drag_wid);

        btn2.reg("drag", {
            move |_ctx, _wid, ev| {
                if let EvPayload::UserData(rc) = &ev.data {
                    let data : Box<dyn std::any::Any> = Box::new(10_usize);
                    *rc.borrow_mut() = data;
                }
                println!("DRAG EV HexGrid: {:?}", ev);
            }
        });

        let btn1 = Widget::new(style_ref.clone());
        btn1.set_ctrl(Control::Button {
            label: Box::new(format!("Lay2 Btn {}", cnt))
        });
        btn1.change_layout(|layout| {
            layout.top    = Some(Units::Pixels(4.0));
            layout.bottom = Some(Units::Pixels(4.0));
            layout.left   = Some(Units::Pixels(4.0));
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

        cont.add(btn0);
        cont.add(btn1);
        cont.add(btn2);

        let knob = Widget::new(
            style_ref.with_style_clone(|style| {
                style.pad_top = 20.0;
                style.pad_left = 30.0;
            }));
        knob.enable_cache();
        let param =
            Rc::new(RefCell::new(
                hexotk::DummyParamModel::new()));
        knob.set_ctrl(Control::HexKnob {
            knob: Box::new(HexKnob::new(param.clone())),
        });

        let hexmodel = Rc::new(RefCell::new(TestGrid { }));
        let hexgrid = Widget::new(
            style_ref.with_style_clone(|style| {
//                style.pad_top = 20.0;
                style.bg_color = hexotk::style::UI_ACCENT_BG1_CLR;
                style.border_style = BorderStyle::Hex { offset: 50.0 };
            }));
        hexgrid.set_ctrl(Control::HexGrid {
            grid: Box::new(HexGrid::new(hexmodel)),
        });
        hexgrid.reg("drop_query", {
            move |_ctx, _wid, ev| {
                println!("Drop Accept query!");
                if let EvPayload::DropAccept(rc) = &ev.data {
                    rc.borrow_mut().1 = true;
                }
            }
        });
        hexgrid.reg("drop", {
            move |_ctx, _wid, ev| {
                if let EvPayload::HexGridDropData { x, y, data: rc } = &ev.data {
                    println!("DROP EV HexGrid ({},{}): {:?}", x, y, rc);
                    if let Some(d) = rc.borrow().as_ref().downcast_ref::<usize>() {
                        println!("XXXX DROP EV HexGrid: {:?}", d);
                    }
                }
            }
        });




        let entry = Widget::new(style_ref.clone());
//        entry.change_layout(|layout| layout.max_height = Units::Pixels(40.0));
        entry.set_ctrl(Control::Entry {
            entry: Box::new(Entry::new(Box::new(etf))),
        });
        entry.change_layout(|layout| layout.max_height = Some(Units::Pixels(40.0)));
        entry.reg("changed", {
            move |_ctx, _wid, _ev| {
                println!("CHANGED ENTRY!");
            }
        });
        col.add(entry);
        col.add(cont);
        col.add(lbl1);

        let wtwid_row = Widget::new(style_ref.clone());
        wtwid_row.add(hexgrid.clone());
        wtwid_row.add(wtwid.clone());
        wtwid_row.change_layout(|layout| {
            layout.layout_type = Some(LayoutType::Row);
            layout.height = Some(Units::Percentage(60.0));
        });

        layer2root.add(col);
        layer2root.add(wtwid_row);

        let knrow = Widget::new(style_ref.clone());
        knrow.change_layout(|layout| {
            layout.layout_type = Some(LayoutType::Row);
        });
        let conwid = Widget::new(style_ref.with_style_clone(|style| {
            style.pad_top    = 5.0;
            style.pad_bottom = 5.0;
            style.bg_color   = hexotk::style::UI_ACCENT_BG1_CLR;
        }));
        let condata = Rc::new(RefCell::new(ConnectorData::new()));
        condata.borrow_mut().add_output("env1".to_string(),  true);
        condata.borrow_mut().add_output("out_l".to_string(), false);
        condata.borrow_mut().add_output("out_r".to_string(), false);
        condata.borrow_mut().add_output("sig".to_string(),   true);
        condata.borrow_mut().add_input("inp1".to_string(), true);
        condata.borrow_mut().add_input("freq".to_string(), true);
        condata.borrow_mut().add_input("gain".to_string(), true);
        condata.borrow_mut().add_input("vol".to_string(),  false);
        conwid.set_ctrl(Control::Connector { con: Box::new(Connector::new(condata)) });

        let octkeys = Widget::new(style_ref.clone());
        let octdata = Rc::new(RefCell::new(DummyOctaveKeysData::new()));
        octkeys.set_ctrl(Control::OctaveKeys {
            keys: Box::new(OctaveKeys::new(octdata))
        });
        octkeys.reg("changed", {
            move |_ctx, _wid, ev| {
                println!("CHANGED KEYS! {:?}", ev);
            }
        });

        knrow.add(knob);
        knrow.add(conwid);
        knrow.add(octkeys);
        layer2root.add(knrow);

        let root3 = Widget::new(style_ref.clone());
        root3.enable_cache();
        root3.add(dw);


        let mut ui = Box::new(UI::new(Rc::new(RefCell::new(1))));
        ui.install_test_script(TestScript::new("test1".to_string()));

        if false {
            style.border_style = BorderStyle::Rect;
            style.border_style = BorderStyle::Hex { offset: 100.0 };
            style.border_style = BorderStyle::Bevel {
                corner_offsets: (5.0, 7.0, 3.0, 0.0),
            };
            let mstyle = Rc::new(style);
            let xxx = Widget::new(mstyle.clone());
            xxx.enable_cache();
            let my_root = Widget::new(mstyle.clone());
            xxx.set_ctrl(Control::Rect);
            xxx.change_layout(|l| l.left = Some(Units::Pixels(10.0)));
            my_root.add(xxx);
            ui.add_layer_root(my_root);
        }

        let mytestroot = Widget::new(style_ref.clone());

//        hexgrid.enable_cache();
//        mytestroot.enable_cache();
//            hexgrid.change_layout(|l| l.left = Some(Units::Pixels(10.0)));
//        mytestroot.add(hexgrid);
//        mytestroot.add(wtwid.clone());

//        ui.add_layer_root(mytestroot.clone());


//        println!("mytestroot id={}", mytestroot.id());
//        println!("wtwid id={}", wtwid.id());



        ui.add_layer_root(layer2root.clone());
        ui.add_layer_root(root3);

        ui
    }));
}
