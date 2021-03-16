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
///         atoms: [f32; 100],
///     }
///
///     let wt_btn = Rc::new(Button::new(80.0, 10.0));
///
///     let mut ui = Box::new(UI::new(
///         WidgetData::new_box(
///             wt_btn, 0.into(), UIPos::center(12, 12),
///             ButtonData::new_toggle("Test Btn")),
///         Box::new(SomeParameters { atoms: [0.0; 100] }),
///         (window_w as f64, window_h as f64),
///     ));
///
///
///     impl AtomDataModel for SomeParameters {
///         fn len(&self) -> usize { self.atoms.len() }
///         fn get(&self, id: AtomId) -> Atom { self.atoms[id.param_id() as usize] }
///         fn get_denorm(&self, id: AtomId) -> Atom { self.atoms[id.param_id() as usize] }
///         fn set(&mut self, id: AtomId, v: Atom) { self.atoms[id.param_id() as usize] = v; }
///         fn set_default(&mut self, id: AtomId) { self.set(id, 0.0); }
///         fn change_start(&mut self, id: AtomId) { }
///         fn change(&mut self, id: AtomId, v: f32, single: bool) {
///             self.set(id, v);
///         }
///         fn change_end(&mut self, id: AtomId, v: f32) {
///             self.set(id, v);
///         }
///         fn step_next(&mut self, id: AtomId) {
///             self.set(id, (self.get(id) + 0.2).fract());
///         }
///         fn step_prev(&mut self, id: AtomId) {
///             self.set(id, ((self.get(id) - 0.2) + 1.0).fract());
///         }
///         fn fmt<'a>(&self, id: AtomId, buf: &'a mut [u8]) -> usize {
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
    /// The root widget.
    main:           Option<Box<WidgetData>>,
    /// The active zones that were defined by the last draw call.
    zones:          Option<Vec<ActiveZone>>,
    /// The current active zone that is hovered.
    hover_zone:     Option<ActiveZone>,
    /// The current mouse position.
    mouse_pos:      (f64, f64),
    /// A pointer to the parameters that are modified by the UI.
    atoms:          Option<Box<dyn AtomDataModel>>,
    /// The current input mode.
    input_mode:     Option<InputMode>,
    /// Holds the pressed modifier keys.
    mod_keys:       ModifierKeys,
    /// Is set, when a widget requires a redraw after some event.
    /// See also `queue_redraw`.
    needs_redraw:   bool,
    /// The window size the UI should be drawn as.
    window_size:    (f64, f64),
}

/// A temporary holder of the UI state.
///
/// Please refer to `UI` about the documentation of the members.
struct WidgetUIHolder {
    zones:          Vec<ActiveZone>,
    hover_zone:     Option<ActiveZone>,
    atoms:          Box<dyn AtomDataModel>,
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

    fn drag_zone_for(&self, az_id: AtomId) -> Option<ActiveZone> {
        if let Some(InputMode::HexFieldDrag { zone }) = self.input_mode {
            if zone.id == az_id {
                return Some(zone);
            }
        }

        None
    }

    fn hover_zone_for(&self, az_id: AtomId) -> Option<ActiveZone> {
        if let Some(hz) = self.hover_zone {
            if hz.id == az_id {
                return Some(hz);
            }
        }
        None
    }

    fn hl_style_for(&self, az_id: AtomId) -> HLStyle {
        if let Some(hz) = self.hover_zone_for(az_id) {
            HLStyle::Hover(hz.zone_type)
        } else {
            HLStyle::None
        }
    }

    fn atoms_mut(&mut self) -> &mut dyn AtomDataModel {
        &mut *self.atoms
    }

    fn atoms(&self) -> &dyn AtomDataModel {
        &*self.atoms
    }
//    fn emit_event(&self, event: UIEvent) {
//    }
}

/// The input mode is a modal mode that is enabled/disabled
/// by certain input events.
#[allow(dead_code)]
#[derive(Debug, Clone)]
enum InputMode {
    /// No input mode active.
    None,
    /// The value drag mode is enabled by clicking inside a
    /// `ZoneType::ValueDragCoarse` or `ZoneType::ValueDragFine`
    /// and holding down the mouse button while moving the mouse.
    /// A mouse up event ends the drag mode.
    ValueDrag {
        /// The original value of the parameter that was initially clicked on.
        value:          f32,
        /// The modification step, a parameter that will define how coarse/fine
        /// the change of the paramter is for N pixels of mouse movement.
        step_dt:        f32,
        /// The `ActiveZone` the current drag action belongs to.
        zone:           ActiveZone,
        /// The original position the mouse cursor was on when pressing mouse
        /// button down.
        orig_pos:       (f64, f64),
        /// A delta value that is set when the user hits the Shift key.
        pre_fine_delta: f32,
        /// Whether the Shift key was pressed.
        fine_key:       bool
    },
    /// If a hexfield is dragged, this input mode stores the
    /// origin of the dragging.
    HexFieldDrag {
        zone: ActiveZone,
    },
    /// Modulation Selection mode.
    SelectMod  {
        /// The zone the modulation is selected for.
        zone: ActiveZone
    },
    /// Direct value input via keyboard.
    InputValue {
        /// The zone for which the value is entered.
        zone:   ActiveZone,
        /// The original input string.
        value:  String,
        /// The currently edited text.
        input:  Rc<RefCell<BufWriter<Vec<u8>>>>
    },
}

impl InputMode {
    pub fn get_param_change_when_drag(&self, mouse_pos: (f64, f64)) -> Option<(AtomId, f32)> {
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

    pub fn check_hex_field_click(&self, release_az: ActiveZone, btn: MButton, mouse_pos: (f64, f64))
        -> Option<UIEvent>
    {
        match release_az.zone_type {
            ZoneType::HexFieldClick { pos: dst_pos, .. } => {

                match self {
                    InputMode::HexFieldDrag { zone } => {

                        if let ZoneType::HexFieldClick { pos, .. } = zone.zone_type {
                            if dst_pos != pos {
                                return
                                    Some(UIEvent::FieldDrag {
                                        id:     release_az.id,
                                        button: btn,
                                        src:    pos,
                                        dst:    dst_pos,
                                    });
                            }
                        }

                        return
                            Some(UIEvent::Click {
                                id:     release_az.id,
                                button: btn,
                                x:      mouse_pos.0,
                                y:      mouse_pos.1,
                            });
                    },
                    _ => None,
                }
            },
            _ => None,
        }
    }
}

impl UI {
    /// Creates a new UI instance.
    /// Please refer to the documentation of the UI data structure above
    /// about a comprehensive example.
    ///
    /// The window size is only the initial window size.
    pub fn new(main: Box<WidgetData>, atoms: Box<dyn AtomDataModel>, window_size: (f64, f64)) -> Self {
        Self {
            main:           Some(main),
            atoms:          Some(atoms),
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
                        if let Some(_id) = z.id_if_inside(pos) {
                            let x = pos.0 - z.pos.x as f64;
                            let y = pos.1 - z.pos.y as f64;

                            // https://web.archive.org/web/20161024224848/http://gdreflections.com/2011/02/hexagonal-grid-math.html
                            let side   = ((tile_size * 3.0) / 2.0).floor();
                            let radius = tile_size;
                            let _width = tile_size * 2.0;
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

                            //d// println!("   *HEX: i={}, j={}", i, j);

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
                        if let Some(_id) = z.id_if_inside(pos) {
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

        let data        = self.main.take();
        let zones       = self.zones.take();
        let atoms       = self.atoms.take();
        let input_mode  = self.input_mode.take();

        if let Some(mut data) = data {
            let mut zones = zones.unwrap();
            let atoms     = atoms.unwrap();
            zones.clear();

            let mut wui   =
                WidgetUIHolder {
                    hover_zone: self.hover_zone,
                    atoms,
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
            self.atoms      = Some(wui.atoms);
            self.input_mode = wui.input_mode;
        }
    }
}

impl WindowUI for UI {
    fn pre_frame(&mut self) {
        if let Some(atoms) = self.atoms.as_mut() {
            atoms.check_sync();
        }
    }

    fn post_frame(&mut self) {
        if self.needs_redraw {
//            self.needs_redraw = false;

            let old_hz = self.hover_zone;
            self.hover_zone = self.get_zone_at(self.mouse_pos);
            if old_hz != self.hover_zone {
                self.queue_redraw();
            }
        }
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
                    self.atoms.as_mut().unwrap().change(id, val, false);
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

                        self.atoms.as_mut().unwrap().change_end(id, val);
                        self.queue_redraw();
                    } else {
                        if let Some(az) = self.get_zone_at(self.mouse_pos) {
                            dispatch_event =
                                input_mode.check_hex_field_click(
                                    az, btn, self.mouse_pos);
                            self.queue_redraw();
                        }
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
                                self.queue_redraw();
                            },
                            _ => {},
                        }
                    }
                }

                self.input_mode = None;
//                println!("UPDATE HOVER ZONE AFTER RELEASE!");
//                self.hover_zone = self.get_zone_at(self.mouse_pos);
            },
            InputEvent::MouseButtonPressed(btn) => {
                let az = self.get_zone_at(self.mouse_pos);

                if let Some(az) = az {
                    match az.zone_type {
                        ZoneType::ValueDragCoarse | ZoneType::ValueDragFine => {
                            if let MButton::Left = btn {
                                let step_dt =
                                    if let ZoneType::ValueDragCoarse = az.zone_type {
                                        DEFAULT_COARSE_STEP
                                    } else {
                                        DEFAULT_FINE_STEP
                                    };

                                let v =
                                    if let Some(v) =
                                        self.atoms.as_ref().unwrap().get(az.id)
                                    {
                                        v.f()
                                    } else {
                                        0.0
                                    };

                                self.input_mode =
                                    Some(InputMode::ValueDrag {
                                        step_dt,
                                        value:          v,
                                        orig_pos:       self.mouse_pos,
                                        zone:           az,
                                        fine_key:       self.mod_keys.fine_drag_key,
                                        pre_fine_delta: 0.0,
                                    });

                                self.atoms.as_mut().unwrap().change_start(az.id);
                                self.queue_redraw();
                            }
                        },
                        ZoneType::HexFieldClick { .. } => {
                            self.input_mode =
                                Some(InputMode::HexFieldDrag {
                                    zone: az
                                });
                        },
                        _ => {},
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
        let win_size = self.window_size;
        self.dispatch(|ui: &mut dyn WidgetUI, data: &mut WidgetData, wt: &dyn WidgetType| {
            wt.draw(ui, data, painter,
                Rect::from(0.0, 0.0, win_size.0, win_size.1));
        });
    }

    fn set_window_size(&mut self, _w: f64, _h: f64) {
        // XXX: The window content is resized by scaling!
        //      So we ignore the real window size here.
        self.queue_redraw();
    }
}
