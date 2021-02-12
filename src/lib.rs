// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

// trait for WidgetType
// - provides active zones
// - can set "input modes" which forward interaction to the widget
// - can draw themself
// - get a reference to their state data, which is stored externally
//   => need to define how and how to interact with client code!
// => Think if container types can be implemented this way.
pub mod widgets;
mod window;
mod constants;
mod femtovg_painter;

use std::rc::Rc;
use std::cell::RefCell;

use keyboard_types::{Key, KeyboardEvent};

pub use window::open_window;

use std::fmt::Debug;

pub struct WidgetData {
    id:    ParamID,
    wtype: usize,
    data:  Box<dyn std::any::Any>,
}

impl WidgetData {
    pub fn new(wtype: usize, id: ParamID, data: Box<dyn std::any::Any>) -> Self {
        Self { wtype, id, data }
    }

    pub fn id(&self) -> ParamID { self.id }

    pub fn widget_type(&self) -> usize { self.wtype }

    pub fn with<F, T: 'static, R>(&mut self, f: F) -> Option<R>
        where F: FnOnce(&mut T) -> R
    {
        if let Some(data) = self.data.downcast_mut::<T>() {
            Some(f(data))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
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

    pub fn offs(&self, x: f64, y: f64) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
            w: self.w,
            h: self.h,
        }
    }

    pub fn is_inside(&self, x: f64, y: f64) -> bool {
           x >= self.x && x <= (self.x + self.w)
        && y >= self.y && y <= (self.y + self.h)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy)]
pub struct ActiveZone {
    pub id:         ParamID,
    pub pos:        Rect,
    pub zone_type:  ZoneType,
}

impl ActiveZone {
    pub fn new_drag_zone(id: ParamID, pos: Rect, coarse: bool) -> Self {
        if coarse {
            Self { id, pos, zone_type: ZoneType::ValueDragCoarse }
        } else {
            Self { id, pos, zone_type: ZoneType::ValueDragFine }
        }
    }

    pub fn new_hex_field(id: ParamID, pos: Rect, tile_size: f64) -> Self {
        Self { id, pos, zone_type: ZoneType::HexFieldClick { tile_size, pos: (0, 0) } }
    }

    pub fn new_input_zone(id: ParamID, pos: Rect) -> Self {
        Self { id, pos, zone_type: ZoneType::ValueInput }
    }

    pub fn new_click_zone(id: ParamID, pos: Rect) -> Self {
        Self { id, pos, zone_type: ZoneType::Click }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ZoneType {
    ValueDragFine,
    ValueDragCoarse,
    ValueInput,
    HexFieldClick { tile_size: f64, pos: (usize, usize), },
    Click,
}

impl ActiveZone {
    pub fn id_if_inside(&self, pos: (f64, f64)) -> Option<ParamID> {
        if self.pos.is_inside(pos.0, pos.1) {
            Some(self.id)
        } else {
            None
        }
    }

    pub fn get_zone_type(&self) -> ZoneType {
        self.zone_type
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ParamID {
    node_id: u32,
    param_id: u32,
}

impl ParamID {
    pub fn new(node_id: u32, param_id: u32) -> Self {
        Self { node_id, param_id }
    }

    pub fn node_id(&self) -> u32 { self.node_id }
    pub fn param_id(&self) -> u32 { self.param_id }
}

impl std::fmt::Display for ParamID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParamID(n={}, p={})", self.node_id, self.param_id)
    }
}

impl From<usize> for ParamID {
    fn from(p: usize) -> Self {
        Self {
            node_id: 0,
            param_id: p as u32,
        }
    }
}

impl From<(u32, u32)> for ParamID {
    fn from(p: (u32, u32)) -> Self {
        Self {
            node_id: p.0,
            param_id: p.1,
        }
    }
}

impl From<(usize, usize)> for ParamID {
    fn from(p: (usize, usize)) -> Self {
        Self {
            node_id: p.0 as u32,
            param_id: p.1 as u32,
        }
    }
}

pub trait Parameters {
    fn len(&self) -> usize;
    fn get(&self, id: ParamID) -> f32;
    fn get_denorm(&self, id: ParamID) -> f32;
    fn set(&mut self, id: ParamID, v: f32);
    fn fmt<'a>(&self, id: ParamID, buf: &'a mut [u8]) -> usize;
    fn step_next(&mut self, id: ParamID);
    fn step_prev(&mut self, id: ParamID);
    fn set_default(&mut self, id: ParamID);
    fn change_start(&mut self, id: ParamID);
    fn change(&mut self, id: ParamID, v: f32, single: bool);
    fn change_end(&mut self, id: ParamID, v: f32);
}

#[derive(Debug, Clone, Copy)]
pub enum HLStyle {
    None,
    Inactive,
    Hover(ZoneType),
    ModTarget,
    HoverModTarget,
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

pub trait Painter {
//    fn start_imgbuf(&mut self, global_id: usize, w: usize, h: usize);
//    fn stop_imgbuf(&mut self);
//    fn imgbuf(&mut self, global_id: usize, x: f64, y: f64);

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
    fn label(&mut self, size: f64, align: i8, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64, text: &str);
    fn label_rot(&mut self, size: f64, align: i8, rot: f64, color: (f64, f64, f64), x: f64, y: f64, xo: f64, yo: f64, w: f64, h: f64, text: &str);
    fn label_mono(&mut self, size: f64, align: i8, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64, text: &str);
    fn font_height(&mut self, size: f32, mono: bool) -> f32;
}

pub trait WindowUI {
    fn pre_frame(&mut self);
    fn post_frame(&mut self);
    fn needs_redraw(&mut self) -> bool;
    fn is_active(&mut self) -> bool;
    fn handle_input_event(&mut self, event: InputEvent);
    fn draw(&mut self, painter: &mut dyn Painter);
    fn set_window_size(&mut self, w: f64, h: f64);
    fn add_widget_type(&mut self, w_type_id: usize, wtype: Box<dyn WidgetType>);
}

pub trait WidgetUI {
    fn define_active_zone(&mut self, az: ActiveZone);
    fn hl_style_for(&self, id: ParamID) -> HLStyle;
    fn hover_zone_for(&self, id: ParamID) -> Option<ActiveZone>;
    fn draw_widget(&mut self, w_type_id: usize, data: &mut WidgetData, p: &mut dyn Painter, rect: Rect);
    fn grab_focus(&mut self);
    fn release_focus(&mut self);
    fn params(&self) -> &dyn Parameters;
    fn params_mut(&mut self) -> &mut dyn Parameters;
}

#[derive(Debug, Clone)]
pub enum UIEvent {
    ValueDragStart { id: ParamID, },
    ValueDrag      { id: ParamID, steps: f64 },
    ValueDragEnd   { id: ParamID, },
    EnteredValue   { id: ParamID, val: String },
    Click          { id: ParamID, button: MButton, x: f64, y: f64 },
}

impl UIEvent {
    pub fn id(&self) -> ParamID {
        match self {
            UIEvent::ValueDragStart { id, .. } => *id,
            UIEvent::ValueDrag      { id, .. } => *id,
            UIEvent::ValueDragEnd   { id, .. } => *id,
            UIEvent::EnteredValue   { id, .. } => *id,
            UIEvent::Click          { id, .. } => *id,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DummyWidget { }

impl DummyWidget {
    pub fn new() -> Self { Self { } }
}

impl WidgetType for DummyWidget {
    fn draw(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _p: &mut dyn Painter, _pos: Rect) { }
    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData) -> (f64, f64) { (0.0, 0.0) }
    fn event(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _ev: UIEvent) { }
}

pub trait WidgetType: Debug {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect);
    fn size(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData) -> (f64, f64);
    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: UIEvent);
}
