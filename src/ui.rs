use crate::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::BufWriter;

const DEFAULT_COARSE_STEP : f32 = 0.05;
const DEFAULT_FINE_STEP   : f32 = 0.01;

struct ModifierKeys {
    fine_drag_key: bool,
}

/// The primary UI logic implementation.
///
/// It takes care of processing the basic mouse and keyboard inputs
/// and setting up the drawing context for the `WidgetData`.
///
/// You can create a new UI like this:
/// ```
///     use hexotk::*;
///     use hexotk::widgets::{Button, ButtonData};
///     use std::rc::Rc;
///
///     let window_w = 800;
///     let window_h = 600;
///
///     struct SomeParameters {
///         params: [f32; 100],
///     }
///
///     let wt_btn = Rc::new(Button::new(80.0, 10.0));
///
///     let mut ui = Box::new(UI::new(
///         WidgetData::new_box(
///             wt_btn, 0.into(), UIPos::center(12, 12),
///             ButtonData::new_toggle("Test Btn")),
///         Box::new(SomeParameters { params: [0.0; 100] }),
///         (window_w as f64, window_h as f64),
///     ));
///
///
///     impl Parameters for SomeParameters {
///         fn len(&self) -> usize { self.params.len() }
///         fn get(&self, id: ParamID) -> f32 { self.params[id.param_id() as usize] }
///         fn get_denorm(&self, id: ParamID) -> f32 { self.params[id.param_id() as usize] }
///         fn set(&mut self, id: ParamID, v: f32) { self.params[id.param_id() as usize] = v; }
///         fn set_default(&mut self, id: ParamID) { self.set(id, 0.0); }
///         fn change_start(&mut self, id: ParamID) { }
///         fn change(&mut self, id: ParamID, v: f32, single: bool) {
///             self.set(id, v);
///         }
///         fn change_end(&mut self, id: ParamID, v: f32) {
///             self.set(id, v);
///         }
///         fn step_next(&mut self, id: ParamID) {
///             self.set(id, (self.get(id) + 0.2).fract());
///         }
///         fn step_prev(&mut self, id: ParamID) {
///             self.set(id, ((self.get(id) - 0.2) + 1.0).fract());
///         }
///         fn fmt<'a>(&self, id: ParamID, buf: &'a mut [u8]) -> usize {
///             use std::io::Write;
///             let mut bw = std::io::BufWriter::new(buf);
///             match write!(bw, "{:6.3}", self.get_denorm(id)) {
///                 Ok(_)  => bw.buffer().len(),
///                 Err(_) => 0,
///             }
///         }
///     }
/// ```

pub struct UI {
    main:           Option<Box<WidgetData>>,
    zones:          Option<Vec<ActiveZone>>,
    hover_zone:     Option<ActiveZone>,
    mouse_pos:      (f64, f64),
    params:         Option<Box<dyn Parameters>>,
    input_mode:     Option<InputMode>,
    mod_keys:       ModifierKeys,
    needs_redraw:   bool,
    window_size:    (f64, f64),
}

struct WidgetUIHolder {
    zones:          Vec<ActiveZone>,
    hover_zone:     Option<ActiveZone>,
    params:         Box<dyn Parameters>,
    input_mode:     Option<InputMode>,
    needs_redraw:   bool,
}

impl WidgetUI for WidgetUIHolder {
    fn define_active_zone(&mut self, az: ActiveZone) {
        self.zones.push(az);
    }

    fn queue_redraw(&mut self) {
        self.needs_redraw = true;
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

impl UI {
    pub fn new(main: Box<WidgetData>, params: Box<dyn Parameters>, window_size: (f64, f64)) -> Self {
        Self {
            main:           Some(main),
            params:         Some(params),
            zones:          Some(vec![]),
            mouse_pos:      (0.0, 0.0),
            hover_zone:     None,
            input_mode:     None,
            needs_redraw:   true,
            window_size,
            mod_keys: ModifierKeys {
                fine_drag_key: false
            },
        }
    }

    fn queue_redraw(&mut self) {
        self.needs_redraw = true;
    }

    fn get_zone_at(&self, pos: (f64, f64)) -> Option<ActiveZone> {
        if let Some(zones) = self.zones.as_ref() {
            let mut zone : Option<ActiveZone> = None;

            for z in zones.iter().rev() {
                match z.zone_type {
                    ZoneType::HexFieldClick { tile_size, y_offs, .. } => {
                        //d// println!("HEXFIELD! {:?} (mouse@ {:?})", z, pos);
                        if let Some(id) = z.id_if_inside(pos) {
                            let x = pos.0 - z.pos.x as f64;
                            let y = pos.1 - z.pos.y as f64;

                            // https://web.archive.org/web/20161024224848/http://gdreflections.com/2011/02/hexagonal-grid-math.html
                            let side   = ((tile_size * 3.0) / 2.0).floor();
                            let radius = tile_size;
                            let width  = tile_size * 2.0;
                            let height = (tile_size * (3.0_f64).sqrt()).floor();

                            let y = if y_offs { y + 0.5 * height } else { y };

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

                            //d// println!("i={}, j={}", i, j);

                            let mut new_az = *z;
                            new_az.zone_type = ZoneType::HexFieldClick {
                                tile_size,
                                y_offs,
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

    fn dispatch<F>(&mut self, f: F)
        where F: FnOnce(&mut dyn WidgetUI, &mut WidgetData, &dyn WidgetType) {

        let mut data        = self.main.take();
        let mut zones       = self.zones.take();
        let mut params      = self.params.take();
        let mut input_mode  = self.input_mode.take();

        if let Some(mut data) = data {
            let mut zones  = zones.unwrap();
            let mut params = params.unwrap();
            zones.clear();

            let mut wui   =
                WidgetUIHolder {
                    hover_zone: self.hover_zone,
                    params,
                    zones,
                    input_mode,
                    needs_redraw: false,
                };

            let wt = data.widget_type();
            f(&mut wui, &mut data, &*wt);

            if wui.needs_redraw {
                self.queue_redraw();
            }
            self.zones      = Some(wui.zones);
            self.main       = Some(data);
            self.params     = Some(wui.params);
            self.input_mode = wui.input_mode;
        }
    }
}

impl WindowUI for UI {
    fn pre_frame(&mut self) {
    }

    fn post_frame(&mut self) {
        self.needs_redraw = false;
    }

    fn needs_redraw(&mut self) -> bool {
        self.needs_redraw
    }

    fn is_active(&mut self) -> bool {
        true
    }

    fn handle_input_event(&mut self, event: InputEvent) {
        let mut dispatch_event = None;

        match event {
            InputEvent::MousePosition(x, y) => {
                self.mouse_pos = (x, y);

                let prev_hz = self.hover_zone;

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
                    self.queue_redraw();
                }

                self.hover_zone = new_hz;

                if self.hover_zone != prev_hz {
                    self.queue_redraw();
                }
            },
            InputEvent::MouseButtonReleased(btn) => {
                if let Some(input_mode) = self.input_mode.take() {
                    if let Some((id, val)) =
                        input_mode.get_param_change_when_drag(self.mouse_pos) {

                        self.params.as_mut().unwrap().change_end(id, val);
                        self.queue_redraw();
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
                            ZoneType::HexFieldClick { .. } => {
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
                                self.queue_redraw();
                            },
                            _ => {},
                        }
                    }
                }

            },
            _ => {
                println!("UNKNOWN INPUT EVENT: {:?}", event);
            },
        }

        if let Some(event) = dispatch_event {
            self.dispatch(|ui: &mut dyn WidgetUI, data: &mut WidgetData,
                           wt: &dyn WidgetType| {

                wt.event(ui, data, &event);
            });
        }
    }

    fn draw(&mut self, painter: &mut dyn Painter) {
        painter.label(20.0, 0, (1.0, 1.0, 0.0), 10.0, 40.0, 100.0, 20.0, "TEST");
        let win_size = self.window_size;
        self.dispatch(|ui: &mut dyn WidgetUI, data: &mut WidgetData, wt: &dyn WidgetType| {
            wt.draw(ui, data, painter,
                Rect::from(0.0, 0.0, win_size.0, win_size.1));
        });
    }

    fn set_window_size(&mut self, w: f64, h: f64) {
        self.queue_redraw();
    }
}
