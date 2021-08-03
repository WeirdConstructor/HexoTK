use hexotk::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use hexotk::widgets::DialogModel;
use hexotk::widgets::UIPatternModel;

const WINDOW_W : i32 = 1150 + 360;
const WINDOW_H : i32 = 720;

// Following functions taken from fastapprox-rs
// https://github.com/loony-bean/fastapprox-rs
// Under MIT License
// Copyright 2018 - Alexey Suslov <alexey.suslov@gmail.com>
mod fastapprox {
    #[inline]
    pub fn to_bits(x: f32) -> u32 {
        unsafe { ::std::mem::transmute::<f32, u32>(x) }
    }

    #[inline]
    pub fn from_bits(x: u32) -> f32 {
        unsafe { ::std::mem::transmute::<u32, f32>(x) }
    }

    /// Base 2 logarithm.
    #[inline]
    pub fn log2(x: f32) -> f32 {
        let mut y = to_bits(x) as f32;
        y *= 1.1920928955078125e-7_f32;
        y - 126.94269504_f32
    }


    /// Raises 2 to a floating point power.
    #[inline]
    pub fn pow2(p: f32) -> f32 {
        let clipp = if p < -126.0 { -126.0_f32 } else { p };
        let v = ((1 << 23) as f32 * (clipp + 126.94269504_f32)) as u32;
        from_bits(v)
    }

    /// Raises a number to a floating point power.
    #[inline]
    pub fn pow(x: f32, p: f32) -> f32 {
        pow2(p * log2(x))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum PatternColType {
    Note,
    Step,
    Value,
    Gate,
}

#[derive(Debug)]
struct PatternData {
    col_types:  [PatternColType; 6],
    data:       Vec<Vec<Option<u16>>>,
    strings:    Vec<Vec<Option<String>>>,
    cursor:     (usize, usize),
    edit_step:  usize,
    rows:       usize,
}

impl PatternData {
    pub fn new(len: usize) -> Self {
        Self {
            col_types:  [PatternColType::Value; 6],
            data:       vec![vec![None; 6]; len],
            strings:    vec![vec![None; 6]; len],
            cursor:     (2, 2),
            edit_step:  4,
            rows:       32,
        }
    }
}

impl UIPatternModel for PatternData {
    fn get_cell(&mut self, row: usize, col: usize) -> Option<&str> {
        if row >= self.data.len()    { return None; }
        if col >= self.data[0].len() { return None; }

        if self.strings[row][col].is_none() {
            if let Some(v) = self.data[row][col] {
                self.strings[row][col] = Some(format!("{:03x}", v));
            } else {
                return None;
            }
        }

        Some(self.strings[row][col].as_ref().unwrap())
    }

    fn clear_cell(&mut self, row: usize, col: usize) {
        if row >= self.data.len()    { return; }
        if col >= self.data[0].len() { return; }

        self.data[row][col]    = None;
        self.strings[row][col] = None;
    }

    fn get_cell_value(&mut self, row: usize, col: usize) -> u16 {
        if row >= self.data.len()    { return 0; }
        if col >= self.data[0].len() { return 0; }

        self.data[row][col].unwrap_or(0)
    }

    fn set_cell_value(&mut self, row: usize, col: usize, val: u16) {
        if row >= self.data.len()    { return; }
        if col >= self.data[0].len() { return; }

        self.data[row][col]    = Some(val);
        self.strings[row][col] = None;
    }

    fn is_col_note(&self, col: usize) -> bool {
        if let Some(ct) = self.col_types.get(col) {
            *ct == PatternColType::Note
        } else {
            false
        }
    }

    fn is_col_step(&self, col: usize) -> bool {
        if let Some(ct) = self.col_types.get(col) {
            *ct == PatternColType::Step
        } else {
            false
        }
    }

    fn is_col_gate(&self, col: usize) -> bool {
        if let Some(ct) = self.col_types.get(col) {
            *ct == PatternColType::Gate
        } else {
            false
        }
    }

    fn cols(&self) -> usize { self.data[0].len() }

    fn rows(&self) -> usize { self.rows }

    fn set_rows(&mut self, rows: usize) {
        self.rows = rows.min(self.data.len());
    }

    fn set_col_note_type(&mut self, col: usize) {
        if col >= self.col_types.len() { return; }
        self.col_types[col] = PatternColType::Note;
    }

    fn set_col_step_type(&mut self, col: usize) {
        if col >= self.col_types.len() { return; }
        self.col_types[col] = PatternColType::Step;
    }

    fn set_col_value_type(&mut self, col: usize) {
        if col >= self.col_types.len() { return; }
        self.col_types[col] = PatternColType::Value;
    }

    fn set_col_gate_type(&mut self, col: usize) {
        if col >= self.col_types.len() { return; }
        self.col_types[col] = PatternColType::Gate;
    }

    fn set_cursor(&mut self, row: usize, col: usize) {
        self.cursor = (row, col);
    }
    fn get_cursor(&self) -> (usize, usize) { self.cursor }
    fn set_edit_step(&mut self, es: usize) { self.edit_step = es; }
    fn get_edit_step(&mut self) -> usize { self.edit_step }
}

struct SomeParameters {
    dialog_model:   Rc<RefCell<DialogModel>>,
    atoms:          Vec<Atom>,
    modamts:        Vec<Option<f32>>,
    phase:          f32,
    test_list1_items: crate::widgets::ListItems,
}

impl AtomDataModel for SomeParameters {
    fn check_sync(&mut self) {
        self.phase = (self.phase + (1.0 / 120.0)).fract();
    }
    fn get_phase_value(&self, _id: AtomId) -> Option<f32> {
        Some(self.phase)
    }
    fn get_led_value(&self, _id: AtomId) -> Option<f32> {
        Some(0.0)
    }
    fn get(&self, id: AtomId) -> Option<&Atom> {
        Some(&self.atoms[id.atom_id() as usize])
    }
    fn enabled(&self, id: AtomId) -> bool {
        id.atom_id() % 2 == 0
    }
    fn get_ui_steps(&self, id: AtomId) -> Option<(f32, f32)> {
        if id.atom_id() == 6 {
            Some((0.005, 0.001))
        } else {
            None
        }
    }
    fn get_ui_range(&self, id: AtomId) -> Option<f32> {
        if id.atom_id() == 7 {
            let v = self.get(id)?.f();
            Some(((v + 1.0) * 0.5).clamp(0.0, 1.0))

        } else if id.atom_id() == 6 {
            Some(self.get(id)?.f() * 10.0)

        } else {
            self.get(id).map(|v| v.f())
        }
    }

    fn get_denorm(&self, id: AtomId) -> Option<f32> {
        Some(self.atoms[id.atom_id() as usize].f())
    }

    fn get_mod_amt(&self, id: AtomId) -> Option<f32> {
        *self.modamts.get(id.atom_id() as usize)?
    }

    fn get_ui_mod_amt(&self, id: AtomId) -> Option<f32> {
        let amt = *self.modamts.get(id.atom_id() as usize)?;
        if let Some(amt) = amt {
            if id.atom_id() == 6 {
                Some(amt * 10.0)
            } else {
                Some(amt)
            }
        } else {
            None
        }
    }

    fn set_mod_amt(&mut self, id: AtomId, amt: Option<f32>) {
        self.modamts[id.atom_id() as usize] = amt;
    }

    fn set(&mut self, id: AtomId, v: Atom) {
        println!("SET ATOM: {}: {:?}", id, v);
        let atid = id.atom_id() as usize;
        if atid == 24 {
            self.set(AtomId::new(id.node_id(), 23), v.clone());

        } else if atid == 25 {
            self.test_list1_items.clear();

            self.dialog_model.borrow_mut().open(
                "Test\nFofeo woei jfweo\nfewiofewiofewfoweifewfewfoiwe jfweofi jewf ijwefo we\
                 \n---page---\nNew page!\nYay you read the second page!",
                Box::new(|_atoms: &mut dyn AtomDataModel| {
                    println!("ATOMS CLICK!!!!");
                }));
        }
        self.atoms[id.atom_id() as usize] = v;
    }

    fn set_denorm(&mut self, id: AtomId, v: f32) {
        self.set(id, v.into());
    }

    fn set_default(&mut self, id: AtomId) {
        self.set(id, self.get(id).unwrap().default_of());
    }

    fn change_start(&mut self, _id: AtomId) {
//        println!("CHANGE START: {}", id);
    }

    fn change(&mut self, id: AtomId, v: f32, _single: bool, res: ChangeRes) {
//        println!("CHANGE: {},{} ({})", id, v, single);
        if id.atom_id() == 5 {
            match res {
                ChangeRes::Coarse =>
                    self.set(id, Atom::param((v * 10.0).round() / 10.0)),
                ChangeRes::Fine =>
                    self.set(id, Atom::param((v * 100.0).round() / 100.0)),
                _ => self.set(id, Atom::param(v)),
            }
        } else {
            self.set(id, Atom::param(v));
        }
    }

    fn change_end(&mut self, id: AtomId, v: f32, res: ChangeRes) {
//        println!("CHANGE END: {},{}", id, v);
        if id.atom_id() == 5 {
            match res {
                ChangeRes::Coarse =>
                    self.set(id, Atom::param((v * 10.0).round() / 10.0)),
                ChangeRes::Fine =>
                    self.set(id, Atom::param((v * 100.0).round() / 100.0)),
                _ => self.set(id, Atom::param(v)),
            }
        } else {
            self.set(id, Atom::param(v));
        }
    }

    fn step_next(&mut self, id: AtomId) {
        self.set(id, Atom::setting(self.get(id).unwrap().i() + 1));
    }

    fn step_prev(&mut self, id: AtomId) {
        self.set(id, Atom::setting(self.get(id).unwrap().i() - 1));
    }

    fn fmt_norm<'a>(&self, id: AtomId, buf: &'a mut [u8]) -> usize {
        use std::io::Write;
        let mut bw = std::io::BufWriter::new(buf);
        match write!(bw, "{:6.4}", self.get(id).unwrap().f()) {
            Ok(_)  => bw.buffer().len(),
            Err(_) => 0,
        }
    }

    fn fmt_mod<'a>(&self, id: AtomId, buf: &'a mut [u8]) -> usize {
        let modamt =
            if let Some(ma) = self.get_mod_amt(id) {
                ma
            } else {
                return 0;
            };
        let norm = self.atoms[id.atom_id() as usize].f();

        use std::io::Write;
        let mut bw = std::io::BufWriter::new(buf);
        match write!(bw, "{:6.3}", norm + modamt) {
            Ok(_)  => bw.buffer().len(),
            Err(_) => 0,
        }
    }

    fn fmt<'a>(&self, id: AtomId, buf: &'a mut [u8]) -> usize {
        use std::io::Write;
        let mut bw = std::io::BufWriter::new(buf);
        match write!(bw, "{:6.3}", self.get_denorm(id).unwrap()) {
            Ok(_)  => bw.buffer().len(),
            Err(_) => 0,
        }
    }
}

#[inline]
pub fn myfun1(x: f32, v: f32) -> f32 {
    if v > 0.75 {
        let xsq1 = x.sqrt();
        let xsq = xsq1.sqrt();
        let v = (v - 0.75) * 4.0;
        xsq1 * (1.0 - v) + xsq * v

    } else if v > 0.5 {
        let xsq = x.sqrt();
        let v = (v - 0.5) * 4.0;
        x * (1.0 - v) + xsq * v

    } else if v > 0.25 {
        let xx = x * x;
        let v = (v - 0.25) * 4.0;
        x * v + xx * (1.0 - v)

    } else {
        let xx = x * x;
        let xxxx = xx * xx;
        let v = v * 4.0;
        xx * v + xxxx * (1.0 - v)
    }
}

struct MinMaxSrc(usize);

impl hexotk::widgets::GraphMinMaxSource for MinMaxSrc {
    fn read(&mut self, buf: &mut [(f64, f64)]) {
        if self.0 == 1 {
            buf[0] = (-1.0,  -0.8);
            buf[1] = (-0.85, -0.5);
            buf[2] = (-0.6,   0.0);
            buf[3] = (0.0,    1.0);
            buf[4] = (0.6,    0.9);
            buf[5] = (0.3,    0.5);
        } else {
            buf[0] = (0.5111111, 0.5111111);
            buf[1] = (0.5111111, 0.5111111);
            buf[2] = (0.5111111, 0.5111111);
            buf[3] = (0.5111111, 0.5111111);
            buf[4] = (0.5111111, 0.5111111);
            buf[5] = (0.5111111, 0.5111111);
        }
    }

    fn fmt_val(&mut self, buf: &mut[u8]) -> usize {
        use std::io::Write;
        let mut bw = std::io::BufWriter::new(buf);
        match write!(bw, "{:5.3} | {:5.3} | {:5.3}", -0.843223, 0.99932, 0.12132) {
            Ok(_)  => bw.buffer().len(),
            Err(_) => 0,
        }
    }
}

fn main() {
    use hexotk::widgets::*;
    use hexotk::components::matrix::NodeMatrixData;

    open_window("HexoTK Demo", WINDOW_W, WINDOW_H, None, Box::new(|| {
        let dialog_model = Rc::new(RefCell::new(DialogModel::new()));

        let wt_btn      = Rc::new(Button::new(80.0, 10.0));
        let wt_btn_spc  = Rc::new(Button::new(80.0, 12.0));
        let wt_knob     = Rc::new(Knob::new(30.0, 10.0, 10.0));
        let wt_cont     = Rc::new(Container::new());
        let wt_text     = Rc::new(Text::new(15.0));
        let wt_entry    = Rc::new(Entry::new_not_editable(60.0, 12.0, 8));
        let wt_list     = Rc::new(List::new(60.0, 12.0, 4));
        let wt_cva      = Rc::new(CvArray::new(16, 120.0, 30.0, 12.0, false));
        let wt_cvab     = Rc::new(CvArray::new(16, 120.0, 20.0, 12.0, true));
        let wt_keys     = Rc::new(Keys::new(220.0, 50.0, 12.0));
        let wt_diag     = Rc::new(Dialog::new());
        let wt_tabs     = Tabs::new_ref();

        let txtsrc = Rc::new(TextSourceRef::new(5));
        txtsrc.set("Foobar\nXXX1239\nfiewfwe\n* 1\n* 2\n* 3");

        let mut fourbtns = ContainerData::new();
        fourbtns.contrast_border().title("Test Container 4 Btns")
           .level(1).shrink(0.0, 10.0)
           .new_row()
           .add(wbox!(wt_knob,    4.into(), center(3, 12), KnobData::new("A")))
           .add(wbox!(wt_knob,    5.into(), center(3, 12), KnobData::new("B")))
           .add(wbox!(wt_knob,    6.into(), center(3, 12), KnobData::new("C0.1")))
           .add(wbox!(wt_knob,    7.into(), center(3, 12), KnobData::new("D-11")));

        let li = ListItems::new(6);
        li.push(-1, String::from("Main"));
        li.push(2,  String::from("Super loud"));
        li.push(3,  String::from("Awesome!"));
        li.push(4,  String::from("Ambient bling"));
        li.push(5,  String::from("Spacebloop"));
        li.push(6,  String::from("MakMukLol"));
        li.push(7,  String::from("ROFLROFLROFLROFL"));
        li.push(8,  String::from("Megalol"));
        li.push(9,  String::from("1."));
        li.push(10, String::from("2."));
        li.push(11, String::from("3."));
        li.push(12, String::from("4."));
        li.push(13, String::from("5."));
        li.push(14, String::from("6.XXXXXXXXXXXX"));

        let mut special_buttons = ContainerData::new();
        special_buttons.level(2)
            .new_row()
            .add(wbox!(
                wt_btn_spc, 25.into(), center(3, 12),
                ButtonData::new_setting_inc("Set Inc")))
            .add(wbox!(
                wt_btn_spc, 25.into(), center(3, 12),
                ButtonData::new_setting_toggle("Set Tog")))
            .add(wbox!(
                wt_btn_spc, 25.into(), center(3, 12),
                ButtonData::new_param_toggle("Param Tog")))
            .add(wbox!(
                wt_btn_spc, 25.into(), center(3, 12),
                ButtonData::new_param_click("Param Clk")));

        let mut tabs = TabsData::new();
        tabs.add(
            "4 Btns",
            wbox!(wt_cont, 100.into(), center(12,12), fourbtns));
        tabs.add(
            "Specials",
            wbox!(wt_cont, 103.into(), center(12,12), special_buttons));

        let mut graphs = ContainerData::new();
        graphs.level(2)
            .new_row()
            .add(wbox!(
                wt_keys, 30.into(), center(12, 4),
                KeysData::new("TestKeys")))
            .new_row()
            .add(wbox!(
                wt_cvab, 29.into(), center(12, 4),
                CvArrayData::new("TestCv")))
            .new_row()
            .add(wbox!(
                wt_cva, 28.into(), center(12, 4),
                CvArrayData::new("TestCv")));

        let mut pres = ContainerData::new();
        pres.level(2)
            .new_row()
            .add(wbox!(
                 wt_entry, 23.into(), center(12, 4),
                 EntryData::new("Preset:")))
            .new_row()
            .add(wbox!(
                 wt_list, 24.into(), center(12, 8),
                 ListData::new("Preset:", ListOutput::ByString, li.clone())));

        let mut other = ContainerData::new();
        other
           .level(1)
           .new_row()
           .add(wbox!(wt_cont, 104.into(), center(6, 12), pres))
           .add(wbox!(wt_cont, 104.into(), center(6, 12), graphs));

        let wt_graph = Rc::new(Graph::new(60.0, 60.0));

        let fun0 =
            Box::new(move |ui: &dyn WidgetUI, _init: bool, x: f64, _xn: f64| -> f64 {
                let v = ui.atoms().get_denorm(4.into()).unwrap_or(1.0) as f64;
                let v = v.clamp(0.0, 1.0);
                myfun1(x as f32, v as f32) as f64
            });

        let fun =
            Box::new(move |ui: &dyn WidgetUI, _init: bool, x: f64, _xn: f64| -> f64 {
                let v = ui.atoms().get_denorm(4.into()).unwrap_or(1.0) as f64;
                let v = v.clamp(0.0, 1.0);
                fastapprox::pow(x as f32, 0.25 + (1.0 - v as f32) * 3.75) as f64
            });

        let fun2 =
            Box::new(move |ui: &dyn WidgetUI, _init: bool, x: f64, _xn: f64| -> f64 {
                let v = ui.atoms().get_denorm(4.into()).unwrap_or(1.0) as f64;
                let v = v.clamp(0.0, 1.0);
                    (x).powf(0.25 * v + (1.0 - v) * 4.0)
            });

        let mut cont = ContainerData::new();
        cont.new_row().border()
             .add(wbox!(wt_graph,99.into(), right( 12, 4), GraphData::new(30, fun0)))
             .new_row()
             .add(wbox!(wt_graph,99.into(), right( 12, 4), GraphData::new(30, fun2)))
             .new_row()
             .add(wbox!(wt_graph,99.into(), right( 12, 4), GraphData::new(30, fun)));

        let wt_graph_mm = Rc::new(GraphMinMax::new(120.0, 90.0));

        let txtsrc2 = Rc::new(TextSourceRef::new(100));
        txtsrc2.set("sig");

        let mut cont2 = ContainerData::new();
        cont2.new_row().border()
           .add(wbox!(wt_graph_mm, 2.into(), center(12, 6),
                      GraphMinMaxData::new(
                        8.0, txtsrc2.clone(), 6, Box::new(MinMaxSrc(1)))))
           .new_row()
           .add(wbox!(wt_graph_mm, 3.into(), center(12, 6),
                      GraphMinMaxData::new(
                        8.0, txtsrc2, 6, Box::new(MinMaxSrc(2))))) ;

        let mut node_ctrls = ContainerData::new();
        node_ctrls.new_row()
           .add(wbox!(wt_cont,99.into(), center(2, 4), cont))
           .add(wbox!(wt_btn,  1.into(), right( 3, 4), ButtonData::new_toggle("Test Btn")))
           .add(wbox!(wt_text, 9.into(), center(3, 4), TextData::new(txtsrc.clone())))
           .add(wbox!(wt_cont,101.into(),center(4, 4), cont2))
           .new_row()
           .add(wbox!(wt_tabs, 46.into(),center(12,4), tabs))
           .new_row()
           .add(wbox!(wt_cont,100.into(),center(12,4), other));

        let pattern_data = Arc::new(Mutex::new(PatternData::new(256)));
        {
            let mut p = pattern_data.lock().unwrap();
            p.set_cell_value(0, 0, 0xFFF);
            p.set_cell_value(0, 1, 0x00F);
            p.set_cell_value(0, 2, 0xF00);
            p.set_cell_value(1, 2, 0x011);
            p.set_cell_value(2, 2, 0x013);
            p.set_cell_value(3, 2, 0x014);
            p.set_cell_value(4, 2, 0x016);
            p.set_cell_value(5, 2, 0x019);

            p.set_col_note_type(0);
            p.set_col_note_type(1);

            for i in 0..50 {
                p.set_cell_value(i, 0, (i + 21) as u16);
                p.set_cell_value(i, 1, (i + 21) as u16);
                p.set_cell_value(i, 0, (i + 21 + 50) as u16);
                p.set_cell_value(i, 1, (i + 21 + 50) as u16);
            }
        }

        let mut con = ContainerData::new();
        con.new_row()
           .add(wbox!(wt_cont, 0.into(), center(4, 12), node_ctrls))
           .add(NodeMatrixData::new(UIPos::center(5, 12), 11))
           .add(wbox!(
                PatternEditor::new_ref(6, 32),
                102.into(),
                center(3, 12),
                PatternEditorData::new(pattern_data)));

        let mut atoms = vec![];
        atoms.resize_with(100, || Atom::default());
        let mut modamts = vec![];
        modamts.resize_with(100, || None);

        atoms[23] = Atom::str("Test");
        atoms[28] = Atom::micro(&[
            0.1, 0.5, 0.01, 0.75,
            0.9, 1.0, 0.99, 1.00,
            0.1, 0.5, 0.01, 0.75,
            0.9, 1.0, 0.99, 1.00,
        ]);

        let ui = Box::new(UI::new(
            WidgetData::new_box(
                wt_cont, 0.into(), UIPos::center(12, 12), con),
            Box::new(wbox!(
                wt_diag, 90000.into(), center(12, 12),
                DialogData::new(90001, 45.into(), dialog_model.clone()))),
            Box::new(SomeParameters {
                atoms,
                modamts,
                phase: 0.0,
                dialog_model: dialog_model.clone(),
                test_list1_items: li,
            }),
            (WINDOW_W as f64, WINDOW_H as f64),
        ));

        let (drv, mut drv_frontend) = Driver::new();

        std::thread::spawn(move || {
            loop {
                std::thread::sleep(
                    std::time::Duration::from_millis(1000));

                drv_frontend.query_state().unwrap();
                println!("TEXTS: {:#?}", drv_frontend.texts);
                println!("MOUSE: {:#?}", drv_frontend.mouse_pos);

//                println!("A");
//                let pos =
//                    drv_frontend.get_zone_pos(
//                        6.into(), DBGID_KNOB_FINE)
//                    .unwrap();
//                println!("b");

//                drv_frontend.move_mouse(316.0, 375.0).unwrap();
//
//                drv_frontend.query_state().unwrap();
//                let hz = drv_frontend.hover.unwrap();
//                println!(">= {:?}", hz);

//                assert_eq!(hz.unwrap().zone_type, ZoneType::ValueDragFine);

//                assert_eq!(
//                    drv_frontend.get_text(
//                        6.into(),
//                        DBGID_KNOB_NAME).unwrap(),
//                    "0.0000");
//                println!("d");
            }
        });

        (drv, ui)
    }));
}

