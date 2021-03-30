use hexotk::*;
use std::rc::Rc;

const WINDOW_W : i32 = 1150;
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

struct SomeParameters {
    atoms: Vec<Atom>,
}

impl AtomDataModel for SomeParameters {
    fn check_sync(&mut self) { }
    fn get_phase_value(&self, id: AtomId) -> Option<f32> {
        Some(0.0)
    }
    fn get_led_value(&self, id: AtomId) -> Option<f32> {
        Some(0.0)
    }
    fn get(&self, id: AtomId) -> Option<&Atom> {
        Some(&self.atoms[id.atom_id() as usize])
    }
    fn get_denorm(&self, id: AtomId) -> Option<f32> {
        Some(self.atoms[id.atom_id() as usize].f())
    }
    fn set(&mut self, id: AtomId, v: Atom) {
        println!("SET ATOM: {}: {:?}", id, v);
        let atid = id.atom_id() as usize;
        if atid == 24 {
            self.set(AtomId::new(id.node_id(), 23), v.clone());

        }
        self.atoms[id.atom_id() as usize] = v;
    }
    fn set_default(&mut self, id: AtomId) {
        self.set(id, self.get(id).unwrap().default_of());
    }

    fn change_start(&mut self, _id: AtomId) {
//        println!("CHANGE START: {}", id);
    }

    fn change(&mut self, id: AtomId, v: f32, _single: bool) {
//        println!("CHANGE: {},{} ({})", id, v, single);
        self.set(id, Atom::param(v));
    }

    fn change_end(&mut self, id: AtomId, v: f32) {
//        println!("CHANGE END: {},{}", id, v);
        self.set(id, Atom::param(v));
    }

    fn step_next(&mut self, id: AtomId) {
        self.set(id, Atom::setting(self.get(id).unwrap().i() + 1));
    }

    fn step_prev(&mut self, id: AtomId) {
        self.set(id, Atom::setting(self.get(id).unwrap().i() - 1));
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
}

fn main() {
    use hexotk::widgets::*;
    use hexotk::components::matrix::NodeMatrixData;

    open_window("HexoTK Demo", WINDOW_W, WINDOW_H, None, Box::new(|| {
        let wt_btn      = Rc::new(Button::new(80.0, 10.0));
        let wt_btn_spc  = Rc::new(Button::new(80.0, 12.0));
        let wt_knob     = Rc::new(Knob::new(30.0, 10.0, 10.0));
        let wt_knob_11  = Rc::new(Knob::new(30.0, 10.0, 10.0).range_signed());
        let wt_cont     = Rc::new(Container::new());
        let wt_text     = Rc::new(Text::new(15.0));
        let wt_entry    = Rc::new(Entry::new(100.0, 12.0, 13));
        let wt_list     = Rc::new(List::new(100.0, 12.0, 8));

        let txtsrc = Rc::new(TextSourceRef::new(5));
        txtsrc.set("Foobar\nXXX1239\nfiewfwe\n* 1\n* 2\n* 3");

        let mut fourbtns = ContainerData::new();
        fourbtns.contrast_border().title("Test Container 4 Btns")
           .level(1).shrink(0.0, 10.0)
           .new_row()
           .add(wbox!(wt_knob,    4.into(), center(3, 12), KnobData::new("A")))
           .add(wbox!(wt_knob,    5.into(), center(3, 12), KnobData::new("B")))
           .add(wbox!(wt_knob,    6.into(), center(3, 12), KnobData::new("C")))
           .add(wbox!(wt_knob_11, 7.into(), center(3, 12), KnobData::new("D")));

        let li = ListItems::new(12);
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
                wt_btn_spc, 25.into(), center(12, 3),
                ButtonData::new_setting_inc("Set Inc")))
            .new_row()
            .add(wbox!(
                wt_btn_spc, 25.into(), center(12, 3),
                ButtonData::new_setting_toggle("Set Tog")))
            .new_row()
            .add(wbox!(
                wt_btn_spc, 25.into(), center(12, 3),
                ButtonData::new_param_toggle("Param Tog")))
            .new_row()
            .add(wbox!(
                wt_btn_spc, 25.into(), center(12, 3),
                ButtonData::new_param_click("Param Clk")));

        let mut other = ContainerData::new();
        other
           .level(1)
           .new_row()
           .add(wbox!(
                wt_entry, 23.into(), center(3, 12),
                EntryData::new("Preset Name:")))
           .add(wbox!(
                wt_list, 24.into(), center(3, 12),
                ListData::new("Preset:", ListOutput::ByString, li)))
            .add(wbox!(
                wt_cont, 103.into(), center(3, 12),
                special_buttons));

        let wt_graph = Rc::new(Graph::new(60.0, 60.0));

        let fun0 =
            Box::new(move |ui: &dyn WidgetUI, _init: bool, x: f64| -> f64 {
                let v = ui.atoms().get_denorm(4.into()).unwrap_or(1.0) as f64;
                let v = v.clamp(0.0, 1.0);
                myfun1(x as f32, v as f32) as f64
            });

        let fun =
            Box::new(move |ui: &dyn WidgetUI, _init: bool, x: f64| -> f64 {
                let v = ui.atoms().get_denorm(4.into()).unwrap_or(1.0) as f64;
                let v = v.clamp(0.0, 1.0);
                fastapprox::pow(x as f32, 0.25 + (1.0 - v as f32) * 3.75) as f64
            });

        let fun2 =
            Box::new(move |ui: &dyn WidgetUI, _init: bool, x: f64| -> f64 {
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
           .add(wbox!(wt_cont,99.into(), center(2, 6), cont))
           .add(wbox!(wt_btn,  1.into(), right( 3, 6), ButtonData::new_toggle("Test Btn")))
           .add(wbox!(wt_text, 6.into(), center(3, 6), TextData::new(txtsrc.clone())))
           .add(wbox!(wt_cont,101.into(),center(4, 6), cont2))
           .new_row()
           .add(wbox!(wt_cont,100.into(),center(12,3), fourbtns))
           .new_row()
           .add(wbox!(wt_cont,100.into(),center(12,3), other));


        let mut con = ContainerData::new();
        con.new_row()
           .add(wbox!(wt_cont, 0.into(), center(5, 12), node_ctrls))
           .add(NodeMatrixData::new(UIPos::center(7, 12), 11));

        let mut atoms = vec![];
        atoms.resize_with(100, || Atom::default());

        atoms[23] = Atom::str("Test");

        let ui = Box::new(UI::new(
            WidgetData::new_box(
                wt_cont, 0.into(), UIPos::center(12, 12), con),
            Box::new(SomeParameters { atoms }),
            (WINDOW_W as f64, WINDOW_H as f64),
        ));

        ui
    }));
}

