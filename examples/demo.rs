use hexotk::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::BufWriter;

const window_w : i32 = 1150;
const window_h : i32 = 720;

struct SomeParameters {
    params: [f32; 100],
}

impl Parameters for SomeParameters {
    fn len(&self) -> usize { self.params.len() }
    fn get(&self, id: ParamID) -> f32 { self.params[id.param_id() as usize] }
    fn get_denorm(&self, id: ParamID) -> f32 { self.params[id.param_id() as usize] }
    fn set(&mut self, id: ParamID, v: f32) { self.params[id.param_id() as usize] = v; }
    fn set_default(&mut self, id: ParamID) {
        self.set(id, 0.0);
    }

    fn change_start(&mut self, id: ParamID) {
//        println!("CHANGE START: {}", id);
    }

    fn change(&mut self, id: ParamID, v: f32, single: bool) {
//        println!("CHANGE: {},{} ({})", id, v, single);
        self.set(id, v);
    }

    fn change_end(&mut self, id: ParamID, v: f32) {
//        println!("CHANGE END: {},{}", id, v);
        self.set(id, v);
    }

    fn step_next(&mut self, id: ParamID) {
        self.set(id, (self.get(id) + 0.2).fract());
    }

    fn step_prev(&mut self, id: ParamID) {
        self.set(id, ((self.get(id) - 0.2) + 1.0).fract());
    }

    fn fmt<'a>(&self, id: ParamID, buf: &'a mut [u8]) -> usize {
        use std::io::Write;
        let mut bw = std::io::BufWriter::new(buf);
        match write!(bw, "{:6.3}", self.get_denorm(id)) {
            Ok(_)  => bw.buffer().len(),
            Err(_) => 0,
        }
    }
}

fn main() {
    use hexotk::widgets::*;
    use hexotk::components::matrix::NodeMatrixData;

    open_window("HexoTK Demo", window_w, window_h, None, Box::new(|| {
        let wt_btn      = Rc::new(Button::new(80.0, 10.0));
        let wt_hexgrid  = Rc::new(HexGrid::new(14.0, 10.0));
        let wt_knob     = Rc::new(Knob::new(30.0, 10.0, 10.0));
        let wt_cont     = Rc::new(Container::new());

        let mut node_ctrls = ContainerData::new();
        node_ctrls.new_row()
           .add(wt_btn,          1.into(), UIPos::right( 6, 6), ButtonData::new_toggle("Test Btn"))
           .add(wt_knob.clone(), 2.into(), UIPos::center(3, 6), KnobData::new())
           .add(wt_knob.clone(), 2.into(), UIPos::center(3, 6), KnobData::new())
           .new_row()
           .add(wt_knob.clone(), 4.into(), UIPos::center(3, 6), KnobData::new())
           .add(wt_knob.clone(), 5.into(), UIPos::center(3, 6), KnobData::new())
           .add(wt_knob.clone(), 6.into(), UIPos::center(3, 6), KnobData::new())
           .add(wt_knob.clone(), 7.into(), UIPos::center(3, 6), KnobData::new());

        let mut con = ContainerData::new();
        con.new_row()
           .add_direct(NodeMatrixData::new(UIPos::center(7, 12), 11))
           .add(wt_cont.clone(), 0.into(), UIPos::center(5, 12), node_ctrls);

        let mut ui = Box::new(UI::new(
            WidgetData::new_box(
                wt_cont, 0.into(), UIPos::center(12, 12), con),
            Box::new(SomeParameters { params: [0.0; 100] }),
            (window_w as f64, window_h as f64),
        ));

        ui
    }));
}

