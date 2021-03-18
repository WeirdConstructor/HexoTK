use hexotk::*;
use std::rc::Rc;

const WINDOW_W : i32 = 1150;
const WINDOW_H : i32 = 720;

struct SomeParameters {
    atoms: Vec<Atom>,
}

impl AtomDataModel for SomeParameters {
    fn len(&self) -> usize { self.atoms.len() }
    fn check_sync(&mut self) { }
    fn get(&self, id: AtomId) -> Option<&Atom> {
        Some(&self.atoms[id.atom_id() as usize])
    }
    fn get_denorm(&self, id: AtomId) -> Option<f32> {
        Some(self.atoms[id.atom_id() as usize].f())
    }
    fn set(&mut self, id: AtomId, v: Atom) {
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

fn main() {
    use hexotk::widgets::*;
    use hexotk::components::matrix::NodeMatrixData;

    open_window("HexoTK Demo", WINDOW_W, WINDOW_H, None, Box::new(|| {
        let wt_btn      = Rc::new(Button::new(80.0, 10.0));
        let wt_knob     = Rc::new(Knob::new(30.0, 10.0, 10.0));
        let wt_knob_11  = Rc::new(Knob::new(30.0, 10.0, 10.0).range_signed());
        let wt_cont     = Rc::new(Container::new());
        let wt_text     = Rc::new(Text::new(15.0));

        let txtsrc = Rc::new(TextSourceRef::new(5));
        txtsrc.set("Foobar\nXXX1239\nfiewfwe\n* 1\n* 2\n* 3");

        let mut fourbtns = ContainerData::new();
        fourbtns.contrast_border().title("Test Container 4 Btns")
           .level(1).shrink(0.0, 10.0)
           .new_row()
           .add(wbox!(wt_knob, 4.into(), center(3, 6), KnobData::new("A")))
           .add(wbox!(wt_knob, 5.into(), center(3, 6), KnobData::new("B")))
           .add(wbox!(wt_knob, 6.into(), center(3, 6), KnobData::new("C")))
           .add(wbox!(wt_knob, 7.into(), center(3, 6), KnobData::new("D")));

        let wt_graph = Rc::new(Graph::new(60.0, 60.0));

        let mut xd = 0.0;

        let fun0 =
            Box::new(move |ui: &mut dyn WidgetUI, init: bool, x: f64| -> f64 {
                let v = ui.atoms().get_denorm(4.into()).unwrap_or(1.0) as f64;
                let v = v.clamp(0.0, 1.0);
//                if v > 0.5 {
//                    let v = (v - 0.5) * 2.0;
//                    ((v * x).exp() - 1.0) / ((x).exp() - 1.0)
////                    (1.0 - x).powf((v - 0.5) * 2.0)
//                } else {

//                let x = 1.0 - x;
//                (x * v + ((x * 10.0 - 10.0) + 0.999).exp() * (1.0 - v))
//                let fact = v * 10.0;
                x * v + (1.0 - v) * x * x * x
//                x * v + (1.0 - v) * (x).powf(10.0)
//                }
            });

        let fun =
            Box::new(move |ui: &mut dyn WidgetUI, init: bool, x: f64| -> f64 {
                let v = ui.atoms().get_denorm(4.into()).unwrap_or(1.0) as f64;
                let v = v.clamp(0.0, 1.0);
//                if v > 0.5 {
//                    let v = (v - 0.5) * 2.0;
//                    ((v * x).exp() - 1.0) / ((x).exp() - 1.0)
////                    (1.0 - x).powf((v - 0.5) * 2.0)
//                } else {

//                let x = 1.0 - x;
//                (x * v + ((x * 10.0 - 10.0) + 0.999).exp() * (1.0 - v))
//                let fact = v * 10.0;
                x * v + (1.0 - v) * (x * 10.0 - 10.0).exp()
//                x * v + (1.0 - v) * (x).powf(10.0)
//                }
            });

        let fun2 =
            Box::new(move |ui: &mut dyn WidgetUI, init: bool, x: f64| -> f64 {
                let v = ui.atoms().get_denorm(4.into()).unwrap_or(1.0) as f64;
                let v = v.clamp(0.0, 1.0);
//                if v > 0.5 {
//                    let v = (v - 0.5) * 2.0;
//                    ((v * x).exp() - 1.0) / ((x).exp() - 1.0)
////                    (1.0 - x).powf((v - 0.5) * 2.0)
//                } else {

//                let x = 1.0 - x;
//                (x * v + ((x * 10.0 - 10.0) + 0.999).exp() * (1.0 - v))
//                let fact = v * 10.0;
//                (x * v * 10.0 - v * 10.0).exp()
//                x * v + (1.0 - v) * (x).powf(v)
//                if v > 0.5 {
//                    let v = (v - 0.5) * 2.0;
//                    1.0 - (1.0 - x).powf(1.0 + v * 4.0)
//                } else {
//                    let v = v * 2.0;
                    (x).powf(0.25 * v + (1.0 - v) * 4.0)
//                }
//                }
            });

        let mut cont = ContainerData::new();
        cont.new_row().border()
             .add(wbox!(wt_graph,99.into(), right( 12, 4), GraphData::new(30, fun0)))
             .new_row()
             .add(wbox!(wt_graph,99.into(), right( 12, 4), GraphData::new(30, fun)))
             .new_row()
             .add(wbox!(wt_graph,99.into(), right( 12, 4), GraphData::new(30, fun2)));

        let mut node_ctrls = ContainerData::new();
        node_ctrls.new_row()
           .add(wbox!(wt_cont,99.into(), center(3, 6), cont))
           .add(wbox!(wt_btn,  1.into(), right( 3, 6), ButtonData::new_toggle("Test Btn")))
           .add(wbox!(wt_text, 6.into(), center(3, 6), TextData::new(txtsrc.clone())))
           .add(wbox!(wt_knob_11, 2.into(), center(3, 6), KnobData::new("A")))
           .new_row()
           .add(wbox!(wt_cont,100.into(),center(12,6), fourbtns));

        let mut con = ContainerData::new();
        con.new_row()
           .add(NodeMatrixData::new(UIPos::center(7, 12), 11))
           .add(wbox!(wt_cont, 0.into(), center(5, 12), node_ctrls));

        let mut atoms = vec![];
        atoms.resize_with(100, || Atom::default());

        let ui = Box::new(UI::new(
            WidgetData::new_box(
                wt_cont, 0.into(), UIPos::center(12, 12), con),
            Box::new(SomeParameters { atoms }),
            (WINDOW_W as f64, WINDOW_H as f64),
        ));

        ui
    }));
}

