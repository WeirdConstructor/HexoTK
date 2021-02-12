use hexotk::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::BufWriter;

const DEFAULT_COARSE_STEP : f32 = 0.05;
const DEFAULT_FINE_STEP   : f32 = 0.01;

struct OpDesc {
    // - Description for module
    // - Ports
    //    - Input / Output
    //    - Type: Control, Audio, Range (200...10000)
    //    - Label
    //    - Description (Help Text)
    // - UI Definition
    //    - Private Settings (UI Only)
    //    - Layout for Ports (multiple containers?)
}

// Backend:
// - All DSP Objects are represented as Node
//   - A node has a buffer for all inputs and outputs
//   - They have a "next()" method, producing the next sample
// - Define two continous arrays of input (parameters) and output values for the N nodes
//   Each node can at max have 28 inputs and 4 outputs.
//   - this ensures that all params are packed well in memory
// - A node only gets a slice of it's inputs and outputs it operates on
//   => I can reuse my abstracted stuff from Kickmess almost without change
//   => Denormalization happens here
//   => Signal outputs need to be normalized (x + 1.0) * 0.5
// - The byte code defines when a node produces a sample
//    - Index 0 is reserved for "nop"
//    - Vec<NodeOp>
//      struct NodeOp {
//          exec_node_idx: u32,
//          in_from: (u32, u32, u32),
//          out_to: (u32, u32, u32),
//      }
//    => Code is configuring up to K (max prog length) execution nodes
//      - They get the index stored
//      - They get 3 in/out indices stored
//      - If one exec node operates, it runs the Node
//        and then maps to/from the indices
//    => Audio Signals are always converted to 1.0 range!
// - UI & Host Parameter changes are transmitted via RingBuffer
//    - Ring buf transfers ramp operations
//          struct Ramp { param_idx: u32, dest: f32 }
//      It is pre-calcuated on receival to a:
//          struct SmoothSource { cur_idx: usize, samples: [f32; 64], param_idx: u32, }
//      On each sample calculation, one value from `samples` is copied to the
//      given param_idx in the big vector.

// Widgets:
//  - Matrix
//      - Access to data model?
//      - draws the hex tiles
//      - and the labels
//  - Envelope view
//      - Point formular in Data
//  - Toggle Button
//      - Labels and values in Data
//  - Knob (3 Sizes?)
//  - Container
//      - Bordered Container
//      - Bordered Container with Title
//      - Tabs!

// - How to connect with UI code?

struct DemoUI {
    types:      Vec<Box<dyn WidgetType>>,
    main:       Option<Box<WidgetData>>,
    zones:      Option<Vec<ActiveZone>>,
    hover_zone: Option<ActiveZone>,
    mouse_pos:  (f64, f64),
    params:     Option<Box<dyn Parameters>>,
    input_mode: Option<InputMode>,
    mod_keys:   ModifierKeys,
}

struct ModifierKeys {
    fine_drag_key: bool,
}

struct WidgetUIHolder<'a> {
    types:          &'a Vec<Box<dyn WidgetType>>,
    zones:          Vec<ActiveZone>,
    hover_zone:     Option<ActiveZone>,
    params:         Box<dyn Parameters>,
    input_mode:     Option<InputMode>,
}

struct SomeParameters {
    params: [f32; 100],
}

impl Parameters for SomeParameters {
    fn len(&self) -> usize { self.params.len() }
    fn get(&self, id: ParamID) -> f32 { self.params[id.param_id() as usize] }
    fn get_denorm(&self, id: ParamID) -> f32 { self.params[id.param_id() as usize] }
    fn set(&mut self, id: ParamID, v: f32) { self.params[id.param_id() as usize] = v; }
    fn change_start(&mut self, id: ParamID) {
        println!("CHANGE START: {}", id);
    }
    fn change(&mut self, id: ParamID, v: f32, single: bool) {
        println!("CHANGE: {},{} ({})", id, v, single);
        self.set(id, v);
    }
    fn change_end(&mut self, id: ParamID, v: f32) {
        println!("CHANGE END: {},{}", id, v);
        self.set(id, v);
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

#[derive(Debug, Clone)]
enum InputMode {
    None,
    ValueDrag {
        value:          f32,
        step_dt:        f32,
        zone:           ActiveZone,
        orig_pos:       (f64, f64),
        pre_fine_delta: f32,
        fine_key:       bool
    },
    SelectMod  { zone: ActiveZone },
    InputValue {
        zone:   ActiveZone,
        value:  String,
        input:  Rc<RefCell<BufWriter<Vec<u8>>>>
    },
    GetHelp,
}

impl InputMode {
    pub fn get_param_change_when_drag(&self, mouse_pos: (f64, f64)) -> Option<(ParamID, f32)> {
        match self {
            InputMode::ValueDrag { value, zone, step_dt, pre_fine_delta,
                                   fine_key, orig_pos, .. } => {

                let distance = (orig_pos.1 - mouse_pos.1) as f32;
                let steps =
                    if *fine_key { distance / 25.0 }
                    else         { distance / 10.0 };

                Some((
                    zone.id,
                    (value + steps * step_dt + pre_fine_delta)
                    .max(0.0).min(1.0)
                ))
            },
            _ => None,
        }
    }
}



impl<'a> WidgetUI for WidgetUIHolder<'a> {
    fn define_active_zone(&mut self, az: ActiveZone) {
        self.zones.push(az);
    }

    fn draw_widget(&mut self, w_type_id: usize, data: &mut WidgetData, p: &mut dyn Painter, rect: Rect) {
    }

    fn grab_focus(&mut self) {
    }

    fn release_focus(&mut self) {
    }

    fn hover_zone_for(&self, az_id: ParamID) -> Option<ActiveZone> {
        if let Some(hz) = self.hover_zone {
            if hz.id == az_id {
                Some(hz)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn hl_style_for(&self, az_id: ParamID) -> HLStyle {
        if let Some(hz) = self.hover_zone_for(az_id) {
            HLStyle::Hover(hz.zone_type)
        } else {
            HLStyle::None
        }
    }

    fn params_mut(&mut self) -> &mut dyn Parameters {
        &mut *self.params
    }
    fn params(&self) -> &dyn Parameters {
        &*self.params
    }
//    fn emit_event(&self, event: UIEvent) {
//    }
}

impl DemoUI {
    fn get_zone_at(&self, pos: (f64, f64)) -> Option<ActiveZone> {
        if let Some(zones) = self.zones.as_ref() {
            let mut zone : Option<ActiveZone> = None;

            for z in zones {
                match z.zone_type {
                    ZoneType::HexFieldClick { tile_size, .. } => {
                        println!("HEXFIELD! {:?} (mouse@ {:?})", z, pos);
                        if let Some(id) = z.id_if_inside(pos) {
                            let x = pos.0 - z.pos.x as f64;
                            let y = pos.1 - z.pos.y as f64;

                            // https://web.archive.org/web/20161024224848/http://gdreflections.com/2011/02/hexagonal-grid-math.html
                            let side   = ((tile_size * 3.0) / 2.0).floor();
                            let radius = tile_size;
                            let width  = tile_size * 2.0;
                            let height = (tile_size * (3.0_f64).sqrt()).floor();

                            let ci = (x / side).floor();
                            let cx = x - side * ci;

                            let ty = (y - (ci as usize % 2) as f64 * height / 2.0).floor();
                            let cj = (ty / height).floor();
                            let cy = (ty - height * cj).floor();

                            let (i, j) =
                                if cx > (radius / 2.0 - radius * cy / height).abs() {
                                    (ci, cj)
                                } else {
                                    (ci - 1.0,
                                     cj + (ci % 2.0)
                                        - (if cy < height / 2.0 { 1.0 } else { 0.0 }))
                                };

//                            let x = (x - tile_size) / (2.0 * tile_size);
//                            let t1 = y / tile_size;
//                            let t2 = (x + t1).floor();
//                            let r = (((t1 - x).floor() + t2) / 3.0).floor();
//                            let q = (((2.0 * x + 1.0).floor() + t2) / 3.0).floor() - r;

//                            let x = x / (tile_size * (3.0_f64).sqrt());
//                            let y = y / (tile_size * (3.0_f64).sqrt());
//
//                            let temp = (x + (3.0_f64).sqrt() * y + 1.0).floor();
//                            let q = (((2.0 * x + 1.0).floor() + temp) / 3.0).floor();
//                            let r = ((temp + (-x + (3.0_f64).sqrt() * y + 1.0).floor()) / 3.0).floor();

//                            println!("q={}, r={}", q, r);
                            println!("i={}, j={}", i, j);

                            let mut new_az = *z;
                            new_az.zone_type = ZoneType::HexFieldClick {
                                tile_size,
                                pos: (i as usize, j as usize),
                            };
                            zone = Some(new_az);
                            break;
                        }
                    },
                    _ => {
                        if let Some(id) = z.id_if_inside(pos) {
                            zone = Some(*z);
                            break;
                        }
                    },
                }
            }

            zone
        } else {
            None
        }
    }

    fn dispatch<F>(&mut self, f: F) where F: FnOnce(&mut dyn WidgetUI, &mut WidgetData, &dyn WidgetType) {
        let mut data        = self.main.take();
        let mut zones       = self.zones.take();
        let mut params      = self.params.take();
        let mut input_mode  = self.input_mode.take();

        if let Some(mut data) = data {
            let mut zones  = zones.unwrap();
            let mut params = params.unwrap();
            zones.clear();

            let w_type_id = data.widget_type();
            let wt        = &self.types[w_type_id];
            let mut wui   =
                WidgetUIHolder {
                    types:      &self.types,
                    hover_zone: self.hover_zone,
                    params,
                    zones,
                    input_mode,
                };

            f(&mut wui, &mut data, wt.as_ref());

            self.zones      = Some(wui.zones);
            self.main       = Some(data);
            self.params     = Some(wui.params);
            self.input_mode = wui.input_mode;
        }
    }
}

impl WindowUI for DemoUI {
    fn add_widget_type(&mut self, w_type_id: usize, wtype: Box<dyn WidgetType>) {
        if w_type_id >= self.types.len() {
            self.types.resize_with((w_type_id + 1) * 2, || Box::new(DummyWidget::new()));
        }

        self.types[w_type_id] = wtype;
    }

    fn pre_frame(&mut self) {
    }

    fn post_frame(&mut self) {
    }

    fn needs_redraw(&mut self) -> bool {
        true
    }

    fn is_active(&mut self) -> bool {
        true
    }

    fn handle_input_event(&mut self, event: InputEvent) {
        let mut dispatch_event = None;

        println!("INPUT: {:?}", event);
        match event {
            InputEvent::MousePosition(x, y) => {
                self.mouse_pos = (x, y);

                self.hover_zone = None;

                let mut new_hz = None;
                let mut param_change = None;

                if let Some(input_mode) = &self.input_mode {
                    if let Some(pc) = input_mode.get_param_change_when_drag(self.mouse_pos) {

                        param_change = Some(pc);
                    } else {
                        new_hz = self.get_zone_at(self.mouse_pos);
                    };
                } else {
                    new_hz = self.get_zone_at(self.mouse_pos);
                }

                if let Some((id, val)) = param_change {
                    self.params.as_mut().unwrap().change(id, val, false);
                }

                self.hover_zone = new_hz;
            },
            InputEvent::MouseButtonReleased(btn) => {
                if let Some(input_mode) = self.input_mode.take() {
                    if let Some((id, val)) =
                        input_mode.get_param_change_when_drag(self.mouse_pos) {

                        self.params.as_mut().unwrap().change_end(id, val);
                    }

                } else {
                    let az = self.get_zone_at(self.mouse_pos);

                    if let Some(az) = az {
                        match az.zone_type {
                            ZoneType::Click => {
                                dispatch_event =
                                    Some(UIEvent::Click {
                                        id:     az.id,
                                        button: btn,
                                        x:      self.mouse_pos.0,
                                        y:      self.mouse_pos.1,
                                    });
                            },
                            _ => {},
                        }
                    }
                }

                // TODO: Handle drag mode end!

                self.input_mode = None;
            },
            InputEvent::MouseButtonPressed(btn) => {
                let az = self.get_zone_at(self.mouse_pos);

                if let Some(az) = az {
                    if let MButton::Left = btn {
                        match az.zone_type {
                            ZoneType::ValueDragCoarse | ZoneType::ValueDragFine => {
                                let step_dt =
                                    if let ZoneType::ValueDragCoarse = az.zone_type {
                                        DEFAULT_COARSE_STEP
                                    } else {
                                        DEFAULT_FINE_STEP
                                    };

                                let v = self.params.as_mut().unwrap().get(az.id);

                                self.input_mode =
                                    Some(InputMode::ValueDrag {
                                        step_dt,
                                        value:          v,
                                        orig_pos:       self.mouse_pos,
                                        zone:           az,
                                        fine_key:       self.mod_keys.fine_drag_key,
                                        pre_fine_delta: 0.0,
                                    });

                                self.params.as_mut().unwrap().change_start(az.id);
                            },
                            _ => {},
                        }
                    }
                }

            },
            _ => {},
        }

        if let Some(event) = dispatch_event {
            self.dispatch(|ui: &mut dyn WidgetUI, data: &mut WidgetData,
                           wt: &dyn WidgetType| {

                wt.event(ui, data, event);
            });
        }
    }

    fn draw(&mut self, painter: &mut dyn Painter) {
        painter.label(20.0, 0, (1.0, 1.0, 0.0), 10.0, 40.0, 100.0, 20.0, "TEST");
        self.dispatch(|ui: &mut dyn WidgetUI, data: &mut WidgetData, wt: &dyn WidgetType| {
            wt.draw(ui, data, painter, Rect::from(30.0, 30.0, 600.0, 440.0));
        });
    }

    fn set_window_size(&mut self, w: f64, h: f64) {
    }
}

#[derive(Debug)]
pub struct Connection {
    node: Box<dyn NodeType>,
    input_param: u8,
}

pub trait NodeType : std::fmt::Debug {
    fn name(&self) -> &str;
    fn instance_num(&self) -> usize;
    fn connection(&self, edge: u8) -> Option<Connection>;
    fn param_label(&self, param: usize) -> Option<&str>;
}

#[derive(Debug)]
struct MatrixCell {
    node: Option<Box<dyn NodeType>>,
}

#[derive(Debug)]
struct MatrixModel {
    cells: Vec<MatrixCell>,
}

impl MatrixModel {
    fn new() -> Self {
        Self {
            cells: vec![],
        }
    }
}

impl hexotk::widgets::hexgrid::HexGridModel for MatrixModel {
    fn cell_visible(&self, x: usize, y: usize) -> bool {
        true
    }

    fn cell_label(&self, x: usize, y: usize, out: &mut [u8]) {
    }

    fn cell_edge_connection(&self, x: usize, y: usize, edge: u8, out: &mut [u8]) -> bool {
        true
    }
}

fn main() {
    open_window("HexoTK Demo", 800, 700, None, Box::new(|| {
        let mut ui = Box::new(DemoUI {
            types:      vec![],
            zones:      Some(vec![]),
            mouse_pos:  (0.0, 0.0),
            hover_zone: None,
            input_mode: None,
            params:     Some(Box::new(SomeParameters { params: [0.0; 100] })),
            mod_keys:
                ModifierKeys {
                    fine_drag_key: false
                },
            main:
//                Some(Box::new((0, hexotk::WidgetData::new(
                Some(Box::new(hexotk::WidgetData::new(
                    0,
                    10.into(),
                    Box::new(hexotk::widgets::ButtonData::new("Test Btn"))
//                   Box::new(hexotk::widgets::KnobData::new())
//                   Box::new(hexotk::widgets::HexGridData::new(
//                      std::sync::Arc::new(MatrixModel::new())))
                )))
//                    Box::new(hexotk::widgets::ButtonData {
//                        label:  String::from("UWU"),
//                        counter: 0,
//                    })))))
        });

        ui.add_widget_type(0, Box::new(hexotk::widgets::Button::new(80.0, 10.0)));
        ui.add_widget_type(1, Box::new(hexotk::widgets::HexGrid { }));
        ui.add_widget_type(2, Box::new(hexotk::widgets::Knob::new(30.0, 10.0, 10.0)));

        ui
    }));
}

