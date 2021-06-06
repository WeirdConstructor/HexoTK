use crate::*;
use std::collections::HashMap;

use keyboard_types::{Key};

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
///         fn change(&mut self, id: AtomId, v: f32, single: bool, _res: ChangeRes) {
///             self.set(id, v);
///         }
///         fn change_end(&mut self, id: AtomId, v: f32, _res: ChangeRes) {
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
    /// The dialog widget.
    dialog:         Option<Box<WidgetData>>,
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
    /// Holds the currently pressed keys for the widgets to query:
    pressed_keys:   Option<Box<HashMap<UIKey, bool>>>,
    /// Is set, when a widget requires a redraw after some event.
    /// See also `queue_redraw`.
    needs_redraw:   bool,
    /// The window size the UI should be drawn as.
    window_size:    (f64, f64),
    /// The currently adjusted scroll offset, can be queried
    /// by any widget.
    cur_hex_trans:  Option<(AtomId, HexGridTransform)>,
}

/// A temporary holder of the UI state.
///
/// Please refer to `UI` about the documentation of the members.
struct WidgetUIHolder {
    zones:           Vec<ActiveZone>,
    hover_zone:      Option<ActiveZone>,
    atoms:           Box<dyn AtomDataModel>,
    input_mode:      Option<InputMode>,
    needs_redraw:    bool,
    pressed_keys:    Box<HashMap<UIKey, bool>>,
    cur_hex_trans:   Option<(AtomId, HexGridTransform)>,
}

impl WidgetUI for WidgetUIHolder {
    fn define_active_zone(&mut self, az: ActiveZone) {
        self.zones.push(az);
    }

    fn get_hex_transform(&self, at_id: AtomId) -> Option<HexGridTransform> {
        if let Some((id, ht)) = &self.cur_hex_trans {
            if *id == at_id {
                return Some(*ht);
            }
        }

        None
    }

    fn queue_redraw(&mut self) {
        self.needs_redraw = true;
    }

    fn grab_focus(&mut self) {
    }

    fn release_focus(&mut self) {
    }

    fn is_key_pressed(&self, key: UIKey) -> bool {
        if let Some(flag) = self.pressed_keys.get(&key) {
            *flag
        } else {
            false
        }
    }

    fn is_input_value_for(&self, az_id: AtomId) -> bool {
        if let Some(InputMode::InputValue { zone }) = self.input_mode {
            return zone.id == az_id;
        }

        false
    }

    fn drag_zone_for(&self, az_id: AtomId) -> Option<ActiveZone> {
        if let Some(InputMode::HexFieldDrag { zone }) = self.input_mode {
            if zone.id == az_id {
                return Some(zone);
            }
        }

        None
    }

    fn hover_atom_id(&self) -> Option<AtomId> {
        if let Some(hz) = self.hover_zone {
            Some(hz.id)
        } else {
            None
        }
    }

    fn hover_zone_for(&self, az_id: AtomId) -> Option<ActiveZone> {
        if let Some(hz) = self.hover_zone {
            if hz.id == az_id {
                return Some(hz);
            }
        }
        None
    }

    fn hl_style_for(&self, az_id: AtomId, idx: Option<usize>) -> HLStyle {
        if let Some(hz) = self.hover_zone_for(az_id) {
            if let Some(input_mode) = &self.input_mode {
                match input_mode {
                    InputMode::AtomClick { .. } => {
                        return HLStyle::AtomClick;
                    },
                    _ => {},
                }
            }

            if let Some(idx) = idx {
                match hz.zone_type {
                    ZoneType::Click { index } => {
                        if idx == index { HLStyle::Hover(hz.zone_type) }
                        else            { HLStyle::None }
                    },
                    _ => HLStyle::None,
                }
            } else {
                HLStyle::Hover(hz.zone_type)
            }
        } else {
            if let Some(input_mode) = &self.input_mode {
                match input_mode {
                    InputMode::Keyboard { zone } => {
                        if zone.id == az_id {
                            return HLStyle::Hover(zone.zone_type);
                        }
                    },
                    _ => ()
                }
            }

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
        fine_key:       bool,
        /// The change resolution, used by the client to round the values.
        res:            ChangeRes,
    },
    HexFieldZoom {
        zone:        ActiveZone,
        hex_trans:   HexGridTransform,
        orig_pos:    (f64, f64),
    },
    HexFieldScroll {
        zone:        ActiveZone,
        hex_trans:   HexGridTransform,
        orig_pos:    (f64, f64),
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
        zone: ActiveZone,
    },
    /// Directly clicked [Atom] values.
    AtomClick {
        zone:       ActiveZone,
        prev_value: f64,
    },
    /// Dragging the mouse while a button is pressed.
    Drag {
        button:     MButton,
        zone:       ActiveZone,
        start_pos:  (f64, f64),
    },
    /// When inside a keyboard controlled zone.
    Keyboard {
        zone:       ActiveZone,
    },
}

impl InputMode {
    fn calc_value_delta(
        orig_pos: (f64, f64), mouse_pos: (f64, f64), fine_key: bool, step_dt: f32)
        -> f32
    {
        let distance = (orig_pos.1 - mouse_pos.1) as f32;

        let steps =
            if fine_key { distance / 100.0 }
            else        { distance / 10.0 };

        steps as f32 * step_dt
    }

    pub fn get_param_change_when_drag(&self, mouse_pos: (f64, f64))
        -> Option<(AtomId, f32, ChangeRes)>
    {
        match self {
            InputMode::ValueDrag { value, zone, step_dt, pre_fine_delta,
                                   fine_key, orig_pos, res, .. } => {

                let value_delta =
                    InputMode::calc_value_delta(
                        *orig_pos, mouse_pos, *fine_key, *step_dt);

                Some((
                    zone.id,
                    (value + value_delta + pre_fine_delta),
                    *res
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
                                index:  0,
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
    pub fn new(main: Box<WidgetData>, dialog: Box<WidgetData>,
               atoms: Box<dyn AtomDataModel>,
               window_size: (f64, f64))
        -> Self
    {
        Self {
            main:               Some(main),
            dialog:             Some(dialog),
            atoms:              Some(atoms),
            zones:              Some(vec![]),
            mouse_pos:          (0.0, 0.0),
            hover_zone:         None,
            input_mode:         None,
            needs_redraw:       true,
            window_size,
            pressed_keys:       Some(Box::new(HashMap::new())),
            cur_hex_trans:      None,
        }
    }

    fn queue_redraw(&mut self) {
        self.needs_redraw = true;
    }

    fn is_key_pressed(&self, key: UIKey) -> bool {
        if let Some(pressed_keys) = &self.pressed_keys {
            if let Some(flag) = pressed_keys.get(&key) {
                return *flag;
            }
        }

        false
    }

    fn get_zone_at(&self, pos: (f64, f64)) -> Option<ActiveZone> {
        if let Some(zones) = self.zones.as_ref() {
            let mut zone : Option<ActiveZone> = None;

            for z in zones.iter().rev() {
                match z.zone_type {
                    ZoneType::HexFieldClick { tile_size, y_offs, hex_trans, .. } => {
                        //d// println!("HEXFIELD! {:?} (mouse@ {:?})", z, pos);
                        if let Some(_id) = z.id_if_inside(pos) {
                            let scale = hex_trans.scale();

                            // move mouse relative to widget rectangle
                            let rel_mouse_x = pos.0 - z.pos.x as f64;
                            let rel_mouse_y = pos.1 - z.pos.y as f64;

                            // move mouse to center and apply the offset
                            let x = rel_mouse_x - z.pos.w * 0.5 - hex_trans.x_offs() * scale;
                            let y = rel_mouse_y - z.pos.h * 0.5 - hex_trans.y_offs() * scale;

                            // scale mouse position
                            // and move the grid top/left by half the window width
                            let x = (x / scale) + z.pos.w * 0.5;
                            let y = (y / scale) + z.pos.h * 0.5;

                            // Tiles are assumed to have 1.0 scale
                            // This means, we have to inversely scale the mouse position
                            // to simulate bigger tiles!

                            let tile_size = tile_size;

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
                                hex_trans,
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

    fn handle_atom_mouse_released(&mut self, az: ActiveZone, prev_value: f64) {
        if let ZoneType::AtomClick {
                atom_type_setting,
                momentary,
                ..
            } = az.zone_type
        {
            if momentary {
                if atom_type_setting {
                    self.atoms.as_mut().unwrap().set(
                        az.id,
                        Atom::setting(prev_value.round() as i64));
                } else {
                    self.atoms.as_mut().unwrap().set(
                        az.id, Atom::param(prev_value as f32));
                }
            }
        }
    }

    fn handle_atom_mouse_pressed(&mut self, az: ActiveZone) {
        if let ZoneType::AtomClick {
                atom_type_setting,
                increment,
                ..
            } = az.zone_type
        {
            let atom =
                self.atoms.as_ref().unwrap().get(az.id);
            let val =
                if let Some(atom) = atom { atom.f() }
                else                     { 0.0 };

            self.input_mode =
                Some(InputMode::AtomClick {
                    zone:       az,
                    prev_value: val as f64,
                });

            if atom_type_setting {
                if increment {
                    self.atoms.as_mut().unwrap().set(
                        az.id,
                        Atom::setting(
                            (val + 1.0).round() as i64));
                } else {
                    self.atoms.as_mut().unwrap().set(
                        az.id,
                        Atom::setting(
                            if val > 0.5 { 0 }
                            else         { 1 }));
                }

            } else {
                if increment {
                    self.atoms.as_mut().unwrap().set(
                        az.id,
                        Atom::param((val + 0.1) % 1.0));
                } else {
                    self.atoms.as_mut().unwrap().set(
                        az.id,
                        Atom::param(
                            if val > 0.5 { 0.0 }
                            else         { 1.0 }));
                }
            }
        }
    }



    fn dispatch<F>(&mut self, f: F)
        where F: FnOnce(&mut dyn WidgetUI, &mut WidgetData, &dyn WidgetType) {

        let data         = self.main.take();
        let zones        = self.zones.take();
        let atoms        = self.atoms.take();
        let input_mode   = self.input_mode.take();
        let pressed_keys = self.pressed_keys.take();

        if let Some(mut data) = data {
            let zones        = zones.unwrap();
            let atoms        = atoms.unwrap();
            let pressed_keys = pressed_keys.unwrap();

            let mut wui   =
                WidgetUIHolder {
                    hover_zone: self.hover_zone,
                    atoms,
                    zones,
                    input_mode,
                    pressed_keys,
                    cur_hex_trans: self.cur_hex_trans,
                    needs_redraw: false,
                };

            let wt = data.widget_type();
            f(&mut wui, &mut data, &*wt);

            if wui.needs_redraw {
                self.queue_redraw();
            }

            self.zones        = Some(wui.zones);
            self.main         = Some(data);
            self.atoms        = Some(wui.atoms);
            self.input_mode   = wui.input_mode;
            self.pressed_keys = Some(wui.pressed_keys);
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

                let mut param_change = None;

                let new_hz = self.get_zone_at(self.mouse_pos);

                if let Some(input_mode) = &self.input_mode {
                    if let Some(pc) =
                        input_mode.get_param_change_when_drag(self.mouse_pos)
                    {
                        param_change = Some(pc);
                    } else {
                        match input_mode {
                            InputMode::HexFieldZoom { orig_pos, hex_trans, zone } => {
                                self.cur_hex_trans =
                                    Some((
                                        zone.id,
                                        hex_trans.set_scale(
                                            hex_trans.scale()
                                            + (self.mouse_pos.1 - orig_pos.1)
                                              / 100.0)));
                            },
                            InputMode::HexFieldScroll { orig_pos, hex_trans, zone } => {
                                self.cur_hex_trans =
                                    Some((
                                        zone.id,
                                        hex_trans.add_offs(
                                            self.mouse_pos.0 - orig_pos.0,
                                            self.mouse_pos.1 - orig_pos.1)));
                            },
                            InputMode::Drag { button, zone, start_pos } => {
                                if let ZoneType::Drag { index } = zone.zone_type {
                                    dispatch_event =
                                        Some(UIEvent::Drag {
                                            id: zone.id,
                                            button: *button,
                                            index,
                                            x: self.mouse_pos.0 - zone.pos.x,
                                            y: self.mouse_pos.1 - zone.pos.y,
                                            start_x: start_pos.0,
                                            start_y: start_pos.1,
                                        });
                                }
                            },
                            _ => { },
                        }
                    };
                }

                if let Some((id, val, res)) = param_change {
                    let res =
                        if self.is_key_pressed(UIKey::Ctrl) {
                            ChangeRes::Free
                        } else { res };

                    self.atoms.as_mut().unwrap().change(id, val, false, res);
                    self.queue_redraw();
                }

                self.hover_zone = new_hz;

                if self.hover_zone != prev_hz {
                    self.queue_redraw();
                }
            },
            InputEvent::MouseWheel(amt) => {
                let az = self.get_zone_at(self.mouse_pos);

                if let Some(az) = az {
                    dispatch_event =
                        Some(UIEvent::Scroll {
                            id:     az.id,
                            amt,
                            x:      self.mouse_pos.0,
                            y:      self.mouse_pos.1,
                        });
                    self.queue_redraw();
                }
            },
            InputEvent::MouseButtonReleased(btn) => {
                let mut reset_input_mode = true;

                if let Some(input_mode) = self.input_mode.take() {
                    if let Some((id, val, res)) =
                        input_mode.get_param_change_when_drag(self.mouse_pos) {

                        let res =
                            if self.is_key_pressed(UIKey::Ctrl) {
                                ChangeRes::Free
                            } else { res };

                        self.atoms.as_mut().unwrap().change_end(id, val, res);
                        self.queue_redraw();
                    } else {
                        match input_mode {
                            InputMode::AtomClick { zone, prev_value } => {
                                self.handle_atom_mouse_released(
                                    zone, prev_value);
                                self.queue_redraw();
                            },
                            InputMode::Keyboard { zone } => {
                                dispatch_event =
                                    Some(UIEvent::Click {
                                        id:     zone.id,
                                        button: btn,
                                        x:      self.mouse_pos.0 - zone.pos.x,
                                        y:      self.mouse_pos.1 - zone.pos.y,
                                        index:  0,
                                    });

                                self.input_mode = Some(input_mode);
                                reset_input_mode = false;
                            },
                            _ => {
                                if let Some(az) =
                                    self.get_zone_at(self.mouse_pos)
                                {
                                    match az.zone_type {
                                        ZoneType::HexFieldClick { .. } => {
                                            dispatch_event =
                                                input_mode.check_hex_field_click(
                                                    az, btn, self.mouse_pos);
                                            self.queue_redraw();
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }

                } else {
                    let az = self.get_zone_at(self.mouse_pos);

                    if let Some(az) = az {
                        match az.zone_type {
                            ZoneType::Click { index } => {
                                dispatch_event =
                                    Some(UIEvent::Click {
                                        id:     az.id,
                                        button: btn,
                                        x:      self.mouse_pos.0 - az.pos.x,
                                        y:      self.mouse_pos.1 - az.pos.y,
                                        index,
                                    });
                                self.queue_redraw();
                            },
                            ZoneType::TextInput => {
                                self.input_mode =
                                    Some(InputMode::InputValue {
                                        zone: az,
                                    });
                                self.queue_redraw();
                                reset_input_mode = false;
                            },
                            _ => {},
                        }
                    }
                }

                if reset_input_mode {
                    self.input_mode = None;
                }
            },
            InputEvent::MouseButtonPressed(btn) => {
                let az = self.get_zone_at(self.mouse_pos);

                if let Some(input_mode) = self.input_mode.take() {
                    match input_mode {
                        InputMode::Keyboard { .. } => {
                            // drop mode.
                        },
                        _ => {
                            // Keep other modes
                            self.input_mode = Some(input_mode);
                        },
                    }
                };

                if let Some(az) = az {
                    match az.zone_type {
                        ZoneType::ValueDragCoarse | ZoneType::ValueDragFine => {
                            match btn {
                                MButton::Left => {
                                    let steps =
                                        self.atoms.as_ref().unwrap()
                                            .get_ui_steps(az.id)
                                            .unwrap_or(
                                                (DEFAULT_COARSE_STEP,
                                                 DEFAULT_FINE_STEP));

                                    let (step_dt, res) =
                                        if let ZoneType::ValueDragCoarse = az.zone_type {
                                            (steps.0, ChangeRes::Coarse)
                                        } else {
                                            (steps.1, ChangeRes::Fine)
                                        };

                                    let v =
                                        if let Some(v) =
                                            self.atoms.as_ref().unwrap().get(az.id)
                                        {
                                            v.f()
                                        } else {
                                            0.0
                                        };

                                    let fine_key =
                                        self.is_key_pressed(UIKey::Shift);

                                    self.input_mode =
                                        Some(InputMode::ValueDrag {
                                            step_dt,
                                            res,
                                            fine_key,
                                            value:          v,
                                            orig_pos:       self.mouse_pos,
                                            zone:           az,
                                            pre_fine_delta: 0.0,
                                        });

                                    self.atoms.as_mut().unwrap().change_start(az.id);
                                    self.queue_redraw();
                                },
                                MButton::Middle => {
                                    self.atoms.as_mut().unwrap().set_default(az.id);
                                    self.queue_redraw();
                                },
                                MButton::Right => {
                                    // TODO: Enter value input mode!
                                }
                            }
                        },
                        ZoneType::HexFieldClick { hex_trans, .. } => {
                            if self.is_key_pressed(UIKey::Shift) {
                                match btn {
                                    MButton::Left => {
                                        self.input_mode =
                                            Some(InputMode::HexFieldScroll {
                                                orig_pos:    self.mouse_pos,
                                                zone:        az,
                                                hex_trans,
                                            });
                                    },
                                    MButton::Right => {
                                        self.input_mode =
                                            Some(InputMode::HexFieldZoom {
                                                orig_pos:    self.mouse_pos,
                                                zone:        az,
                                                hex_trans,
                                            });
                                    },
                                    _ => {}
                                }

                            } else {
                                self.input_mode =
                                    Some(InputMode::HexFieldDrag {
                                        zone: az
                                    });
                            }
                        },
                        ZoneType::AtomClick { ..  } => {
                            self.handle_atom_mouse_pressed(az);
                            self.queue_redraw();
                        },
                        ZoneType::Keyboard => {
                            self.input_mode =
                                Some(InputMode::Keyboard { zone: az });
                            self.queue_redraw();
                        },
                        ZoneType::Drag { .. } => {
                            self.input_mode =
                                Some(InputMode::Drag {
                                    button:     btn,
                                    zone:       az,
                                    start_pos:  (
                                        self.mouse_pos.0 - az.pos.x,
                                        self.mouse_pos.1 - az.pos.y,
                                    ),
                                });

                            if let ZoneType::Drag { index } = az.zone_type {
                                dispatch_event =
                                    Some(UIEvent::Drag {
                                        id: az.id,
                                        button: btn,
                                        index,
                                        x: self.mouse_pos.0 - az.pos.x,
                                        y: self.mouse_pos.1 - az.pos.y,
                                        start_x: self.mouse_pos.0 - az.pos.x,
                                        start_y: self.mouse_pos.1 - az.pos.y,
                                    });
                            }
                        },
                        _ => {},
                    }
                }

            },
            InputEvent::KeyPressed(key) => {
                println!("KEY PRESSED {:?}", key);
                if let Some(InputMode::Keyboard { zone }) = self.input_mode {
                    dispatch_event =
                        Some(UIEvent::Key { id: zone.id, key: key.key.clone() });
                }

                match key.key {
                    Key::Backspace => {
                        if let Some(InputMode::InputValue { zone }) = self.input_mode {
                            if let Some(atoms) = self.atoms.as_mut() {
                                let new_str =
                                    if let Some(at) = atoms.get(zone.id) {
                                        let s = at.str_ref().unwrap_or("");
                                        let len = s.chars().count();
                                        let s : String =
                                            s.chars().take(len - 1).collect();
                                        s
                                    } else {
                                        String::from("")
                                    };
                                atoms.set(zone.id, Atom::str_mv(new_str));
                            }
                        }
                    },
                    Key::Character(c) => {
                        if let Some(InputMode::InputValue { zone }) = self.input_mode {
                            if let Some(atoms) = self.atoms.as_mut() {
                                let new_str =
                                    if let Some(at) = atoms.get(zone.id) {
                                        let s = at.str_ref().unwrap_or("");
                                        format!("{}{}", s, c)
                                    } else {
                                        format!("{}", c)
                                    };
                                atoms.set(zone.id, Atom::str_mv(new_str));
                            }
                        }
                    },
                    _ => {
                        if key.key == Key::Shift {
                            if let Some(InputMode::ValueDrag {
                                value, pre_fine_delta, fine_key,
                                step_dt, orig_pos, ..
                            }) = &mut self.input_mode
                            {
                                if !*fine_key {
                                    *pre_fine_delta =
                                        InputMode::calc_value_delta(
                                            *orig_pos, self.mouse_pos,
                                            false, *step_dt);
                                    *orig_pos       = self.mouse_pos;
                                    *fine_key       = true;
                                }
                            }
                        }

                        if let Some(ui_key) = UIKey::from(key.key.clone()) {
                            self.pressed_keys.as_mut().unwrap()
                                .insert(ui_key, true);

                        } else if dispatch_event.is_none() {
                            if let Some(main) = &self.main {
                                dispatch_event =
                                    Some(UIEvent::Key {
                                        id:  main.id(),
                                        key: key.key.clone()
                                    });
                            }
                        }
                    },
                }
            },
            InputEvent::KeyReleased(key) => {
                if let Some(ui_key) = UIKey::from(key.key) {
                    self.pressed_keys.as_mut().unwrap().remove(&ui_key);
                }
            },
            _ => {
                println!("UNKNOWN INPUT EVENT: {:?}", event);
            },
        }

        if let Some(event) = dispatch_event {
            let mut dialog = self.dialog.take();

            self.dispatch(|ui: &mut dyn WidgetUI, data: &mut WidgetData,
                           wt: &dyn WidgetType| {

                wt.event(ui, data, &event);

                if let Some(dialog) = &mut dialog {
                    let wt = dialog.widget_type();
                    wt.event(ui, dialog, &event);
                }
            });

            self.dialog = dialog;
        }
    }

    fn draw(&mut self, painter: &mut dyn Painter) {
        let win_size = self.window_size;

        self.zones.as_mut().map(|zones| zones.clear());
        let mut dialog = self.dialog.take();

        self.dispatch(|ui: &mut dyn WidgetUI, data: &mut WidgetData, wt: &dyn WidgetType| {
            let main_pos = Rect::from(0.0, 0.0, win_size.0, win_size.1);
            wt.draw(ui, data, painter, main_pos);

            if let Some(dialog) = &mut dialog {
                let dialog_pos =
                    main_pos.shrink(main_pos.w * 0.25, main_pos.h * 0.25);

                let wt = dialog.widget_type();

                wt.draw(ui, dialog, painter, dialog_pos);
            }
        });

        self.dialog = dialog;
    }

    fn set_window_size(&mut self, _w: f64, _h: f64) {
        // XXX: The window content is resized by scaling!
        //      So we ignore the real window size here.
        self.queue_redraw();
    }
}
