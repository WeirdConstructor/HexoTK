// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

pub mod widgets;
pub mod components;
pub mod constants;

mod ui;
mod window;
mod femtovg_painter;

use std::rc::Rc;

use keyboard_types::KeyboardEvent;

pub use window::open_window;
pub use ui::*;

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
    id:    ParamID,
    wtype: Rc<dyn WidgetType>,
    pos:   UIPos,
    data:  Box<dyn std::any::Any>,
}

impl WidgetData {
    pub fn new_box(wtype: Rc<dyn WidgetType>, id: ParamID, pos: UIPos, data: Box<dyn std::any::Any>) -> Box<Self> {
        Box::new(Self { wtype, id, data, pos })
    }

    pub fn new_tl_box(wtype: Rc<dyn WidgetType>, id: ParamID, data: Box<dyn std::any::Any>) -> Box<Self> {
        Box::new(Self { wtype, id, data, pos: UIPos::center(12, 12) })
    }

    pub fn new(wtype: Rc<dyn WidgetType>, id: ParamID, pos: UIPos, data: Box<dyn std::any::Any>) -> Self {
        Self { wtype, id, data, pos }
    }

    pub fn pos(&self) -> UIPos { self.pos }

    pub fn id(&self) -> ParamID { self.id }

    pub fn widget_type(&self) -> Rc<dyn WidgetType> { self.wtype.clone() }

    pub fn with<F, T: 'static, R>(&mut self, f: F) -> Option<R>
        where F: FnOnce(&mut T) -> R
    {
        if let Some(data) = self.data.downcast_mut::<T>() {
            Some(f(data))
        } else {
            None
        }
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
        wt.draw(ui, self, p, rect);
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

    pub fn new_hex_field(id: ParamID, pos: Rect, y_offs: bool, tile_size: f64) -> Self {
        Self { id, pos, zone_type: ZoneType::HexFieldClick { tile_size, y_offs, pos: (0, 0) } }
    }

    pub fn new_input_zone(id: ParamID, pos: Rect) -> Self {
        Self { id, pos, zone_type: ZoneType::ValueInput }
    }

    pub fn new_click_zone(id: ParamID, pos: Rect) -> Self {
        Self { id, pos, zone_type: ZoneType::Click }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ZoneType {
    ValueDragFine,
    ValueDragCoarse,
    ValueInput,
    HexFieldClick {
        tile_size: f64,
        y_offs:    bool,
        pos:       (usize, usize),
    },
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

#[derive(Debug, Clone, Copy, PartialEq)]
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
}

pub trait WidgetUI {
    /// Defines the active zone that is currently drawn by the widget.
    ///
    /// ```
    /// use hexotk::*;
    ///
    /// ui.define_active_zone(
    ///     ActiveZone::new_click_zone(
    ///         ParamID::new(0, 10),
    ///         Rect::from_tpl((0.0, 0.0, 40.0, 40.0))
    ///             .offs(10.0, 10.0)));
    /// ```
    fn define_active_zone(&mut self, az: ActiveZone);
    fn hl_style_for(&self, id: ParamID) -> HLStyle;
    fn hover_zone_for(&self, id: ParamID) -> Option<ActiveZone>;
    fn queue_redraw(&mut self);
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
    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, avail: (f64, f64)) -> (f64, f64) { avail }
    fn event(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _ev: &UIEvent) { }
}

pub trait WidgetType: Debug {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect);
    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, avail: (f64, f64)) -> (f64, f64) { avail }
    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent);
}
