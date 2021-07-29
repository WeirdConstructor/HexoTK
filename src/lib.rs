// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

pub mod widgets;
pub mod components;
pub mod constants;

mod driver;
mod ui;
#[allow(clippy::type_complexity)]
mod window;
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
mod femtovg_painter;

use std::rc::Rc;

use keyboard_types::{KeyboardEvent, Key};

pub use window::open_window;
pub use ui::*;
pub use driver::*;

use std::fmt::Debug;

// TODO: Define default margin/padding between grid cells
#[derive(Debug, Clone, Copy)]
pub struct UIPos {
    pub col_size: u8,
    pub row_size: u8,
    pub align:    i8,
    pub valign:   i8,
}

impl UIPos {
    pub fn center(col_size: u8, row_size: u8)  -> Self { UIPos { col_size, row_size, align:  0, valign: 0 } }
    pub fn left(col_size: u8, row_size: u8)    -> Self { UIPos { col_size, row_size, align: -1, valign: 0 } }
    pub fn right(col_size: u8, row_size: u8)   -> Self { UIPos { col_size, row_size, align:  1, valign: 0 } }
    pub fn middle(mut self) -> Self { self.valign = 0;  self }
    pub fn top(mut self) -> Self    { self.valign = -1; self }
    pub fn bottom(mut self) -> Self { self.valign = 1;  self }
    pub fn alignment(&self) -> (i8, i8) {
        (self.align, self.valign)
    }
}

pub struct WidgetData {
    id:    AtomId,
    wtype: Rc<dyn WidgetType>,
    pos:   UIPos,
    data:  Box<dyn std::any::Any>,
}

impl WidgetData {
    pub fn new_box(wtype: Rc<dyn WidgetType>, id: AtomId, pos: UIPos, data: Box<dyn std::any::Any>) -> Box<Self> {
        Box::new(Self { wtype, id, data, pos })
    }

    pub fn new_tl_box(wtype: Rc<dyn WidgetType>, id: AtomId, data: Box<dyn std::any::Any>) -> Box<Self> {
        Box::new(Self { wtype, id, data, pos: UIPos::center(12, 12) })
    }

    pub fn new(wtype: Rc<dyn WidgetType>, id: AtomId, pos: UIPos, data: Box<dyn std::any::Any>) -> Self {
        Self { wtype, id, data, pos }
    }

    pub fn pos(&self) -> UIPos { self.pos }

    pub fn id(&self) -> AtomId { self.id }

    pub fn widget_type(&self) -> Rc<dyn WidgetType> { self.wtype.clone() }

    pub fn with<F, T: 'static, R>(&mut self, f: F) -> Option<R>
        where F: FnOnce(&mut T) -> R
    {
        self.data.downcast_mut::<T>().map(|data| f(data))
    }

    pub fn event(&mut self, ui: &mut dyn WidgetUI, ev: &UIEvent) {
        let wt = self.widget_type();
        wt.event(ui, self, ev);
    }

    pub fn size(&mut self, ui: &mut dyn WidgetUI, avail: (f64, f64)) -> (f64, f64) {
        let wt = self.widget_type();
        wt.size(ui, self, avail)
    }

    pub fn draw(&mut self, ui: &mut dyn WidgetUI, p: &mut dyn Painter, rect: Rect) {
        let wt = self.widget_type();
        #[cfg(feature = "driver")]
        { p.start_widget(self.id()); }
        wt.draw(ui, self, p, rect);
        #[cfg(feature = "driver")]
        { p.end_widget(self.id()); }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn from_tpl(t: (f64, f64, f64, f64)) -> Self {
        Self { x: t.0, y: t.1, w: t.2, h: t.3 }
    }

    pub fn from(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    pub fn resize(&self, w: f64, h: f64) -> Self {
        Self {
            x: self.x,
            y: self.y,
            w,
            h,
        }
    }

    pub fn scale(&self, factor: f64) -> Self {
        Self {
            x: self.x,
            y: self.y,
            w: self.w * factor,
            h: self.h * factor,
        }
    }

    pub fn center(&self) -> Self {
        Self {
            x: self.x + self.w * 0.5,
            y: self.y + self.h * 0.5,
            w: 1.0,
            h: 1.0,
        }
    }

    pub fn crop_left(&self, delta: f64) -> Self {
        Self {
            x: self.x + delta,
            y: self.y,
            w: self.w - delta,
            h: self.h,
        }
    }

    pub fn crop_right(&self, delta: f64) -> Self {
        Self {
            x: self.x,
            y: self.y,
            w: self.w - delta,
            h: self.h,
        }
    }

    pub fn crop_bottom(&self, delta: f64) -> Self {
        Self {
            x: self.x,
            y: self.y,
            w: self.w,
            h: self.h - delta,
        }
    }

    pub fn crop_top(&self, delta: f64) -> Self {
        Self {
            x: self.x,
            y: self.y + delta,
            w: self.w,
            h: self.h - delta,
        }
    }

    pub fn shrink(&self, delta_x: f64, delta_y: f64) -> Self {
        Self {
            x: self.x + delta_x,
            y: self.y + delta_y,
            w: self.w - 2.0 * delta_x,
            h: self.h - 2.0 * delta_y,
        }
    }

    pub fn grow(&self, delta_x: f64, delta_y: f64) -> Self {
        Self {
            x: self.x - delta_x,
            y: self.y - delta_y,
            w: self.w + 2.0 * delta_x,
            h: self.h + 2.0 * delta_y,
        }
    }

    pub fn offs(&self, x: f64, y: f64) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
            w: self.w,
            h: self.h,
        }
    }

    pub fn move_into(mut self, pos: &Rect) -> Self {
        if self.x < pos.x { self.x = pos.x; }
        if self.y < pos.y { self.y = pos.y; }

        if (self.x + self.w) > (pos.x + pos.w) {
            self.x = (pos.x + pos.w) - self.w;
        }

        if (self.y + self.h) > (pos.y + pos.h) {
            self.y = (pos.y + pos.h) - self.h;
        }

        self
    }

    pub fn aabb_is_inside(&self, aabb: Rect) -> bool {
        if self.is_inside(aabb.x,          aabb.y)          { return true; }
        if self.is_inside(aabb.x + aabb.w, aabb.y)          { return true; }
        if self.is_inside(aabb.x,          aabb.y + aabb.h) { return true; }
        if self.is_inside(aabb.x + aabb.w, aabb.y + aabb.h) { return true; }
        false
    }

    pub fn is_inside(&self, x: f64, y: f64) -> bool {
           x >= self.x && x <= (self.x + self.w)
        && y >= self.y && y <= (self.y + self.h)
    }

    fn calc_widget_rect(&self, row_offs: u8, col_offs: u8, pos: UIPos) -> (Rect, u8, u8) {
        let x = self.x + ((self.w * (col_offs     as f64)) / 12.0).round();
        let y = self.y + ((self.h * (row_offs     as f64)) / 12.0).round();
        let w =          ((self.w * (pos.col_size as f64)) / 12.0).round();
        let h =          ((self.h * (pos.row_size as f64)) / 12.0).round();

        let new_row_offs = row_offs + pos.row_size;
        let new_col_offs = col_offs + pos.col_size;

        (Rect { x, y, w, h }, new_row_offs, new_col_offs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActiveZone {
    pub id:         AtomId,
    pub pos:        Rect,
    pub zone_type:  ZoneType,
    pub dbgid:      usize,
}

impl ActiveZone {
    pub fn new_drag_zone(id: AtomId, pos: Rect, coarse: bool) -> Self {
        if coarse {
            Self { id, pos, zone_type: ZoneType::ValueDragCoarse, dbgid: 0 }
        } else {
            Self { id, pos, zone_type: ZoneType::ValueDragFine, dbgid: 0 }
        }
    }

    pub fn new_hex_field(id: AtomId, pos: Rect, y_offs: bool,
        hex_trans: HexGridTransform, tile_size: f64) -> Self
    {
        Self {
            id, pos,
            zone_type: ZoneType::HexFieldClick {
                tile_size,
                y_offs,
                hex_trans,
                pos: (0, 0),
            },
            dbgid: 0,
        }
    }

    pub fn new_input_zone(id: AtomId, pos: Rect) -> Self {
        Self { id, pos, zone_type: ZoneType::TextInput, dbgid: 0 }
    }

    pub fn new_click_zone(id: AtomId, pos: Rect) -> Self {
        Self { id, pos, zone_type: ZoneType::Click { index: 0 }, dbgid: 0 }
    }

    pub fn new_toggle_zone(id: AtomId, pos: Rect) -> Self {
        Self { id, pos, zone_type: ZoneType::Click { index: 0 }, dbgid: 0 }
    }

    pub fn new_indexed_click_zone(id: AtomId, pos: Rect, index: usize) -> Self {
        Self { id, pos, zone_type: ZoneType::Click { index }, dbgid: 0 }
    }

    pub fn new_indexed_drag_zone(id: AtomId, pos: Rect, index: usize) -> Self {
        Self { id, pos, zone_type: ZoneType::Drag { index }, dbgid: 0 }
    }

    pub fn new_keyboard_zone(id: AtomId, pos: Rect) -> Self {
        Self { id, pos, zone_type: ZoneType::Keyboard, dbgid: 0 }
    }

    pub fn new_atom_toggle(id: AtomId, pos: Rect, atom_type_setting: bool, momentary: bool) -> Self {
        Self {
            id, pos,
            zone_type: ZoneType::AtomClick {
                atom_type_setting,
                increment: false,
                momentary,
            },
            dbgid: 0,
        }
    }

    pub fn new_atom_inc(id: AtomId, pos: Rect, momentary: bool) -> Self {
        Self {
            id, pos,
            zone_type: ZoneType::AtomClick {
                atom_type_setting: true,
                increment: true,
                momentary,
            },
            dbgid: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HexGridTransform {
    offs:       (f64, f64),
    scale:      f64,
    scale_mid:  (f64, f64),
}

impl HexGridTransform {
    pub fn new() -> Self {
        Self {
            offs:      (0.0, 0.0),
            scale:     1.0,
            scale_mid: (0.0, 0.0),
        }
    }

    pub fn add_offs(&self, xo: f64, yo: f64) -> Self {
        Self {
            offs: (
                self.offs.0 + xo / self.scale,
                self.offs.1 + yo / self.scale
            ),
            scale:     self.scale,
            scale_mid: self.scale_mid,
        }
    }

    pub fn set_offs(&self, offs: (f64, f64)) -> Self {
        Self {
            offs,
            scale:     self.scale,
            scale_mid: self.scale_mid,
        }
    }

    pub fn set_scale(&self, scale: f64) -> Self {
        Self {
            scale,
            offs:      self.offs,
            scale_mid: self.scale_mid,
        }
    }

    pub fn scale(&self) -> f64 { self.scale }

    pub fn x_offs(&self) -> f64 { self.offs.0 }
    pub fn y_offs(&self) -> f64 { self.offs.1 }
}

impl Default for HexGridTransform { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ZoneType {
    ValueDragFine,
    ValueDragCoarse,
    TextInput,
    HexFieldClick {
        tile_size:   f64,
        y_offs:      bool,
        pos:         (usize, usize),
        hex_trans:   HexGridTransform,
    },
    Click {
        index: usize,
    },
    Drag {
        index: usize,
    },
    Keyboard,
    AtomClick {
        /// Whether this is an [Atom::Setting] (`true`) or [Atom::Param] (`false`).
        atom_type_setting: bool,
        /// Whether to increment the value or toggle between 1 and 0.
        increment:         bool,
        /// Whether the toggling/increment is only while the mouse button
        /// is held down.
        momentary:         bool,
    },
}

impl ActiveZone {
    pub fn id_if_inside(&self, pos: (f64, f64)) -> Option<AtomId> {
        if self.pos.is_inside(pos.0, pos.1) {
            Some(self.id)
        } else {
            None
        }
    }

    pub fn dbgid(mut self, dbgid: usize) -> Self {
        self.dbgid = dbgid;
        self
    }

    pub fn get_zone_type(&self) -> ZoneType {
        self.zone_type
    }
}

#[derive(Debug, Clone)]
pub enum Atom {
    Str(String),
    MicroSample(Vec<f32>),
    AudioSample((String, Option<std::sync::Arc<Vec<f32>>>)),
    Setting(i64),
    Param(f32),
}

impl Atom {
    pub fn str(s: &str)         -> Self { Atom::Str(s.to_string()) }
    pub fn str_mv(s: String)    -> Self { Atom::Str(s) }
    pub fn setting(s: i64)      -> Self { Atom::Setting(s) }
    pub fn param(p: f32)        -> Self { Atom::Param(p) }
    pub fn micro(m: &[f32])  -> Self { Atom::MicroSample(m.to_vec()) }
    pub fn audio(s: &str, m: std::sync::Arc<Vec<f32>>) -> Self {
        Atom::AudioSample((s.to_string(), Some(m)))
    }

    pub fn audio_unloaded(s: &str) -> Self {
        Atom::AudioSample((s.to_string(), None))
    }

    pub fn default_of(&self) -> Self {
        match self {
            Atom::Str(_)         => Atom::Str("".to_string()),
            Atom::MicroSample(_) => Atom::MicroSample(vec![]),
            Atom::AudioSample(_) => Atom::AudioSample(("".to_string(), None)),
            Atom::Setting(_)     => Atom::Setting(0),
            Atom::Param(_)       => Atom::Param(0.0),
        }
    }

    pub fn is_continous(&self) -> bool {
        matches!(self, Atom::Param(_))
    }

    pub fn str_ref(&self) -> Option<&str> {
        match self {
            Atom::Str(s)                => Some(&s),
            Atom::AudioSample((s, _))   => {
                if let Some(idx) = s.rfind('/') {
                    Some(&s[(idx + 1)..])
                } else if let Some(idx) = s.rfind('\\') {
                    Some(&s[(idx + 1)..])
                } else {
                    Some(&s)
                }
            },
            _ => None,
        }
    }

    pub fn i(&self) -> i64 {
        match self {
            Atom::Setting(i) => *i,
            Atom::Param(i)   => *i as i64,
            _                => 0,
        }
    }

    pub fn f(&self) -> f32 {
        match self {
            Atom::Setting(i) => *i as f32,
            Atom::Param(i)   => *i,
            _                => 0.0,
        }
    }

    pub fn set_v_idx_micro(&self, idx: usize, v: f32) -> Option<Self> {
        let data = self.v_ref()?;

        if idx >= data.len() {
            return None;
        }

        let mut new_vec = data.to_vec();
        new_vec[idx] = v;

        Some(Atom::MicroSample(new_vec))
    }

    pub fn v_ref(&self) -> Option<&[f32]> {
        match self {
            Atom::MicroSample(v)            => Some(&v[..]),
            Atom::AudioSample((_, Some(v))) => Some(&v[..]),
            _                               => None,
        }
    }
}

impl From<f32> for Atom {
    fn from(n: f32) -> Self { Atom::Param(n) }
}
//impl Into<SAtom> for Atom {
//    fn into(self) -> SAtom {
//        match self {
//            Atom::Str(s)         => SAtom::Str(s),
//            Atom::MicroSample(s) => SAtom::MicroSample(s),
//            Atom::AudioSample(s) => SAtom::AudioSample(s),
//            Atom::Setting(s)     => SAtom::Setting(s),
//            Atom::Param(s)       => SAtom::Param(s),
//        }
//    }
//}
//
//impl Into<Atom> for SAtom {
//    fn into(self) -> Atom {
//        match self {
//            SAtom::Str(s)         => Atom::Str(s),
//            SAtom::MicroSample(s) => Atom::MicroSample(s),
//            SAtom::AudioSample(s) => Atom::AudioSample(s),
//            SAtom::Setting(s)     => Atom::Setting(s),
//            SAtom::Param(s)       => Atom::Param(s),
//        }
//    }
//}

use hexodsp::SAtom;

impl From<Atom> for SAtom {
    fn from(n: Atom) -> Self {
        match n {
            Atom::Str(s)         => SAtom::Str(s),
            Atom::MicroSample(s) => SAtom::MicroSample(s),
            Atom::AudioSample(s) => SAtom::AudioSample(s),
            Atom::Setting(s)     => SAtom::Setting(s),
            Atom::Param(s)       => SAtom::Param(s),
        }
    }
}

impl From<SAtom> for Atom {
    fn from(n: SAtom) -> Atom {
        match n {
            SAtom::Str(s)         => Atom::Str(s),
            SAtom::MicroSample(s) => Atom::MicroSample(s),
            SAtom::AudioSample(s) => Atom::AudioSample(s),
            SAtom::Setting(s)     => Atom::Setting(s),
            SAtom::Param(s)       => Atom::Param(s),
        }
    }
}

impl std::default::Default for Atom {
    fn default() -> Self { Atom::Param(0.0) }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AtomId {
    node_id: u32,
    atom_id: u32,
}

impl AtomId {
    pub fn new(node_id: u32, atom_id: u32) -> Self {
        Self { node_id, atom_id }
    }

    pub fn node_id(&self) -> u32 { self.node_id }
    pub fn atom_id(&self) -> u32 { self.atom_id }
}

impl std::fmt::Display for AtomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AtomId(n={}, a={})", self.node_id, self.atom_id)
    }
}

impl From<usize> for AtomId {
    fn from(a: usize) -> Self {
        Self {
            node_id: 0,
            atom_id: a as u32,
        }
    }
}

impl From<(u32, u32)> for AtomId {
    fn from(a: (u32, u32)) -> Self {
        Self {
            node_id: a.0,
            atom_id: a.1,
        }
    }
}

impl From<(usize, usize)> for AtomId {
    fn from(a: (usize, usize)) -> Self {
        Self {
            node_id: a.0 as u32,
            atom_id: a.1 as u32,
        }
    }
}

/// This specifies the granularity or resultion of the change.
/// The client of this API can then round the given changed values
/// to a fine/coarse step, or no step at all.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum ChangeRes {
    Free,
    Fine,
    Coarse,
}

pub trait AtomDataModel {
    fn check_sync(&mut self);
    fn get_phase_value(&self, id: AtomId) -> Option<f32>;
    fn get_led_value(&self, id: AtomId) -> Option<f32>;
    /// Should return true if the UI for the parameter can be changed
    /// by the user. In HexoSynth this might return false if the
    /// corresponding input is controlled by an output port.
    fn enabled(&self, id: AtomId) -> bool;
    fn get(&self, id: AtomId) -> Option<&Atom>;
    /// Should return a value in the range 0.0 to 1.0 for displayed knob position.
    /// For instance: a normalized value in the range -1.0 to 1.0 needs to be mapped
    /// to 0.0 to 1.0 by: `(x + 1.0) * 0.5`
    fn get_ui_range(&self, id: AtomId) -> Option<f32>;
    /// Should return a coarse step and a fine step for the normalized value.
    /// If none are returned, the UI will assume default steps of:
    ///
    /// * Default coarse: 0.05
    /// * Default fine: 0.01
    fn get_ui_steps(&self, id: AtomId) -> Option<(f32, f32)>;
    /// Should return the modulation amount like it will be applied to the
    /// inputs.
    fn get_mod_amt(&self, id: AtomId) -> Option<f32>;
    /// Should return the modulation amount for the 0..1 UI knob range.
    /// Internally you should transform that into the appropriate
    /// modulation amount in relation to what [get_ui_range] returns.
    fn get_ui_mod_amt(&self, id: AtomId) -> Option<f32>;
    /// Set the UI modulation amount like it will be used in the
    /// modulation later and be returned from [get_mod_amt].
    fn set_mod_amt(&mut self, id: AtomId, amt: Option<f32>);
    fn get_denorm(&self, id: AtomId) -> Option<f32>;
    fn set(&mut self, id: AtomId, v: Atom);
    fn set_denorm(&mut self, id: AtomId, v: f32);
    fn fmt(&self, id: AtomId, buf: &mut [u8]) -> usize;
    fn fmt_mod(&self, id: AtomId, buf: &mut [u8]) -> usize;
    fn fmt_norm(&self, id: AtomId, buf: &mut [u8]) -> usize;
    fn step_next(&mut self, id: AtomId);
    fn step_prev(&mut self, id: AtomId);
    fn set_default(&mut self, id: AtomId);
    fn change_start(&mut self, id: AtomId);
    fn change(&mut self, id: AtomId, v: f32, single: bool, res: ChangeRes);
    fn change_end(&mut self, id: AtomId, v: f32, res: ChangeRes);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HLStyle {
    None,
    Inactive,
    Hover(ZoneType),
    AtomClick,
    EditModAmt,
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    MousePosition(f64, f64),
    MouseButtonPressed(MButton),
    MouseButtonReleased(MButton),
    MouseWheel(f64),
    KeyPressed(KeyboardEvent),
    KeyReleased(KeyboardEvent),
    WindowClose,
}

#[derive(Debug, Clone)]
pub enum WidgetEvent {
    Clicked,
}

#[allow(clippy::too_many_arguments)]
pub trait Painter {
//    fn start_imgbuf(&mut self, global_id: usize, w: usize, h: usize);
//    fn stop_imgbuf(&mut self);
//    fn imgbuf(&mut self, global_id: usize, x: f64, y: f64);

    fn start_widget(&mut self, _id: AtomId) { }
    fn end_widget(&mut self, _id: AtomId) { }

    fn path_fill(&mut self, color: (f64, f64, f64),
                 segments: &mut dyn std::iter::Iterator<Item = (f64, f64)>,
                 closed: bool);
    fn path_stroke(&mut self, width: f64, color: (f64, f64, f64),
                   segments: &mut dyn std::iter::Iterator<Item = (f64, f64)>,
                   closed: bool);

    fn path_fill_rot(&mut self, color: (f64, f64, f64),
                     rot: f64, x: f64, y: f64, xo: f64, yo: f64,
                     segments: &mut dyn std::iter::Iterator<Item = (f64, f64)>,
                     closed: bool);
    fn path_stroke_rot(&mut self, width: f64, color: (f64, f64, f64),
                       rot: f64, x: f64, y: f64, xo: f64, yo: f64,
                       segments: &mut dyn std::iter::Iterator<Item = (f64, f64)>,
                       closed: bool);

    fn arc_stroke(&mut self, width: f64, color: (f64, f64, f64), radius: f64,
                  from_rad: f64, to_rad: f64, x: f64, y: f64);
    fn rect_fill(&mut self, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64);
    fn rect_stroke(&mut self, width: f64, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64);
    fn label(&mut self, size: f64, align: i8, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64, text: &str, lblid: usize);
    fn label_rot(&mut self, size: f64, align: i8, rot: f64, color: (f64, f64, f64), x: f64, y: f64, xo: f64, yo: f64, w: f64, h: f64, text: &str, lblid: usize);
    fn label_mono(&mut self, size: f64, align: i8, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64, text: &str, lblid: usize);
    fn font_height(&mut self, size: f32, mono: bool) -> f32;
    fn clip_region(&mut self, x: f64, y: f64, w: f64, h: f64);
    fn reset_clip_region(&mut self);
    fn move_and_scale(&mut self, x: f64, y: f64, x2: f64, y2: f64, factor: f64);
    fn reset_scale(&mut self);
}

pub struct UIState {
    zones:      Vec<ActiveZone>,
    mouse_pos:  (f64, f64),
    hover:      Option<ActiveZone>,
}

pub trait WindowUI {
    fn pre_frame(&mut self);
    fn post_frame(&mut self);
    fn needs_redraw(&mut self) -> bool;
    fn is_active(&mut self) -> bool;
    fn handle_input_event(&mut self, event: InputEvent);
    fn draw(&mut self, painter: &mut dyn Painter);
    fn set_window_size(&mut self, w: f64, h: f64);
    /// This breaks abstraction a bit, but is used for the [crate::Driver] to
    /// get it's data for testing purposes.
    fn query_state(&self) -> UIState;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
pub enum UIKey {
    Shift,
    Ctrl,
}

impl UIKey {
    fn from(key: Key) -> Option<Self> {
        match key {
            Key::Shift   => Some(UIKey::Shift),
            Key::Control => Some(UIKey::Ctrl),
            _ => { None }
        }
    }
}

pub trait WidgetUI {
    /// Defines the active zone that is currently drawn by the widget.
    ///
    /// ```
    /// use hexotk::*;
    ///
    /// ui.define_active_zone(
    ///     ActiveZone::new_click_zone(
    ///         AtomId::new(0, 10),
    ///         Rect::from_tpl((0.0, 0.0, 40.0, 40.0))
    ///             .offs(10.0, 10.0)));
    /// ```
    fn define_active_zone(&mut self, az: ActiveZone);
    fn get_hex_transform(&self, at_id: AtomId) -> Option<HexGridTransform>;
    fn hl_style_for(&self, id: AtomId, idx: Option<usize>) -> HLStyle;
    fn hover_zone_for(&self, id: AtomId) -> Option<ActiveZone>;
    fn is_input_value_for(&self, az_id: AtomId) -> bool;
    fn hover_atom_id(&self) -> Option<AtomId>;
    fn drag_zone_for(&self, id: AtomId) -> Option<ActiveZone>;
    fn queue_redraw(&mut self);
    fn grab_focus(&mut self);
    fn release_focus(&mut self);
    fn is_key_pressed(&self, key: UIKey) -> bool;
    fn atoms(&self) -> &dyn AtomDataModel;
    fn atoms_mut(&mut self) -> &mut dyn AtomDataModel;
}

#[derive(Debug, Clone)]
pub enum UIEvent {
    ValueDragStart { id: AtomId, },
    ValueDrag      { id: AtomId, steps: f64 },
    ValueDragEnd   { id: AtomId, },
    Click          { id: AtomId, button: MButton, index: usize, x: f64, y: f64 },
    Drag           { id: AtomId, button: MButton, index: usize, x: f64, y: f64, start_x: f64, start_y: f64 },
    Scroll         { id: AtomId, amt: f64, x: f64, y: f64 },
    FieldDrag      { id: AtomId, button: MButton, src: (usize, usize), dst: (usize, usize), mouse_pos: (f64, f64) },
    Key            { id: AtomId, key: Key, mouse_pos: (f64, f64) },
}

impl UIEvent {
    pub fn id(&self) -> AtomId {
        match self {
            UIEvent::ValueDragStart { id, .. } => *id,
            UIEvent::ValueDrag      { id, .. } => *id,
            UIEvent::ValueDragEnd   { id, .. } => *id,
            UIEvent::Click          { id, .. } => *id,
            UIEvent::Drag           { id, .. } => *id,
            UIEvent::Scroll         { id, .. } => *id,
            UIEvent::FieldDrag      { id, .. } => *id,
            UIEvent::Key            { id, .. } => *id,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DummyWidget { }

#[allow(clippy::new_without_default)]
impl DummyWidget {
    pub fn new() -> Self { Self { } }
}

impl WidgetType for DummyWidget {
    fn draw(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _p: &mut dyn Painter, _pos: Rect) { }
    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, avail: (f64, f64)) -> (f64, f64) { avail }
    fn event(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _ev: &UIEvent) { }
}

#[macro_export]
macro_rules! wbox {
    ($wd: expr, $pid: expr, $pos: ident($v: expr, $h: expr), $data: expr) => {
        WidgetData::new($wd.clone(), $pid, UIPos::$pos($v, $h), $data)
    }
}

pub trait WidgetType: Debug {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect);
    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, avail: (f64, f64)) -> (f64, f64) { avail }
    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent);
}

#[macro_export]
macro_rules! define_containing_widget {
    ($type: ident, $data_type: ident) => {
        #[derive(Debug)]
        pub struct $type;

        impl $type {
            pub fn new_ref() -> Rc<$type> { Rc::new(Self) }
        }

        impl WidgetType for $type {
            fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData,
                    p: &mut dyn Painter, pos: Rect)
            {
                data.with(|data: &mut $data_type| {
                    data.check_cont_update(ui);
                    data.cont.draw(ui, p, pos);
                });
            }

            fn event(&self, ui: &mut dyn WidgetUI,
                     data: &mut WidgetData, ev: &UIEvent)
            {
                data.with(|data: &mut $data_type| {
                    data.cont.event(ui, ev);
                });
            }
        }
    }
}

#[macro_export]
macro_rules! define_containing_opt_shared_widget {
    ($type: ident, $data_type: ident) => {
        #[derive(Debug)]
        pub struct $type;

        impl $type {
            pub fn new_ref() -> Rc<$type> { Rc::new(Self) }
        }

        impl WidgetType for $type {
            fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData,
                    p: &mut dyn Painter, pos: Rect)
            {
                data.with(|data: &mut $data_type| {
                    data.check_cont_update(ui);
                    let mut bor = data.cont.borrow_mut();
                    if let Some(wid) = bor.as_mut() {
                        wid.draw(ui, p, pos);
                    }
                });
            }

            fn event(&self, ui: &mut dyn WidgetUI,
                     data: &mut WidgetData, ev: &UIEvent)
            {
                data.with(|data: &mut $data_type| {
                    let mut bor = data.cont.borrow_mut();
                    if let Some(wid) = bor.as_mut() {
                        wid.event(ui, ev);
                    }
                });
            }
        }
    }
}


#[macro_export]
macro_rules! define_containing_widget_v_split {
    ($type: ident, $data_type: ident, $first_height: expr) => {
        #[derive(Debug)]
        pub struct $type;

        impl $type {
            pub fn new_ref() -> Rc<$type> { Rc::new(Self) }
        }

        impl WidgetType for $type {
            fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData,
                    p: &mut dyn Painter, pos: Rect)
            {
                data.with(|data: &mut $data_type| {
                    data.check_cont_update(ui);
                    let pos_top    = pos.crop_bottom(pos.h - $first_height);
                    let pos_bottom = pos.crop_top($first_height);
                    data.cont_top.draw(ui, p, pos_top);
                    data.cont_bottom.draw(ui, p, pos_bottom);
                });
            }

            fn event(&self, ui: &mut dyn WidgetUI,
                     data: &mut WidgetData, ev: &UIEvent)
            {
                data.with(|data: &mut $data_type| {
                    data.cont_top.event(ui, ev);
                    data.cont_bottom.event(ui, ev);
                });
            }
        }
    }
}
