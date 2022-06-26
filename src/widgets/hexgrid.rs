// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::{Widget, InputEvent, Event, MButton, EvPayload, Style};

use crate::style::*;

use crate::painter::*;
use crate::rect::*;

use std::rc::Rc;
use std::cell::RefCell;

macro_rules! hxclr {
    ($i: expr) => {
        (
            ($i >> 16 & 0xFF) as f32 / 255.0,
            ($i >> 8  & 0xFF) as f32 / 255.0,
            ($i       & 0xFF) as f32 / 255.0,
        )
    }
}

pub fn hex_color_idx2clr(style: &Style, idx: u8) -> (f32, f32, f32) {
    style.colors[idx as usize % style.colors.len()]
}

pub const UI_GRID_TXT_CENTER_CLR    : (f32, f32, f32) = UI_PRIM_CLR;
pub const UI_GRID_TXT_CENTER_HL_CLR : (f32, f32, f32) = UI_HLIGHT_CLR;
pub const UI_GRID_TXT_CENTER_SL_CLR : (f32, f32, f32) = UI_SELECT_CLR;
pub const UI_GRID_TXT_EDGE_CLR      : (f32, f32, f32) = UI_PRIM_CLR;
//pub const UI_GRID_CELL_BORDER_CLR   : (f32, f32, f32) = UI_ACCENT_CLR;
pub const UI_GRID_EMPTY_BORDER_CLR  : (f32, f32, f32) = UI_ACCENT_DARK_CLR;
pub const UI_GRID_HOVER_BORDER_CLR  : (f32, f32, f32) = UI_SELECT_CLR;
pub const UI_GRID_DRAG_BORDER_CLR   : (f32, f32, f32) = UI_HLIGHT_CLR;
pub const UI_GRID_BG1_CLR           : (f32, f32, f32) = UI_ACCENT_BG1_CLR;
//pub const UI_GRID_BG2_CLR           : (f32, f32, f32) = UI_ACCENT_BG2_CLR;
pub const UI_GRID_SIGNAL_OUT_CLR    : (f32, f32, f32) = UI_HLIGHT_CLR;
pub const UI_GRID_LED_CLR           : (f32, f32, f32) = UI_PRIM_CLR;
pub const UI_GRID_LED_R             : f32             = 5.0;


#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum HexDir {
    TR,
    BR,
    B,
    BL,
    TL,
    T
}

impl HexDir {
    pub fn from(edge: u8) -> Self {
        match edge {
            0 => HexDir::TR,
            1 => HexDir::BR,
            2 => HexDir::B,
            3 => HexDir::BL,
            4 => HexDir::TL,
            5 => HexDir::T,
            _ => HexDir::TR,
        }
    }

    #[inline]
    pub fn is_right_half(&self) -> bool {
        let e = self.as_edge();
        e <= 2
    }

    #[inline]
    pub fn is_left_half(&self) -> bool {
        !self.is_right_half()
    }

    #[inline]
    pub fn as_edge(&self) -> u8 {
        *self as u8
    }
}

use hexodsp::CellDir;

impl From<HexDir> for CellDir {
    fn from(h: HexDir) -> Self {
        CellDir::from(h.as_edge())
    }
}

impl From<CellDir> for HexDir {
    fn from(c: CellDir) -> Self {
        HexDir::from(c.as_edge())
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum HexEdge {
    NoArrow,
    Arrow,
    ArrowValue { value: (f32, f32) },
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum HexHLight {
    Normal,
    Plain,
    Accent,
    HLight,
    Select,
}

#[derive(Debug)]
pub struct HexCell<'a> {
    pub label:      &'a str,
    pub hlight:     HexHLight,
    pub rg_colors:  Option<(f32, f32)>,
}

pub trait HexGridModel {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn cell_visible(&self, x: usize, y: usize) -> bool;
    fn cell_empty(&self, x: usize, y: usize) -> bool;
    fn cell_color(&self, _x: usize, _y: usize) -> u8 { 0 }

    fn cell_label<'a>(&self, x: usize, y: usize, out: &'a mut [u8])
        -> Option<HexCell<'a>>; // (&'a str, HexCell, Option<(f32, f32)>)>;

    /// Edge: 0 top-right, 1 bottom-right, 2 bottom, 3 bottom-left, 4 top-left, 5 top
    fn cell_edge<'a>(&self, x: usize, y: usize, edge: HexDir, out: &'a mut [u8])
        -> Option<(&'a str, HexEdge)>;

    fn get_generation(&self) -> u64;
}

fn hex_size2wh(size: f32) -> (f32, f32) {
    (2.0_f32 * size, (3.0_f32).sqrt() * size)
}

enum HexDecorPos {
    Center(f32, f32),
    Top(f32, f32),
    TopLeft(f32, f32),
    TopRight(f32, f32),
    Bottom(f32, f32),
    BotLeft(f32, f32),
    BotRight(f32, f32),
}

impl HexEdge {
    fn draw(&self, p: &mut Painter, scale: f32, x: f32, y: f32, rot: f32) {
        match self {
            HexEdge::NoArrow => {},
            HexEdge::Arrow => {
                draw_arrow(p, UI_GRID_TXT_EDGE_CLR, x, y, 0.0, 0.0, 10.0 * scale, rot);
            },
            HexEdge::ArrowValue { value } => {
                draw_arrow(p, UI_GRID_SIGNAL_OUT_CLR, x, y, 0.0, 0.0, 10.0 * scale, rot);
                let clr = (
                    value.0,
                    value.1,
                    0.3,
                );
                draw_arrow(p, clr, x, y, 1.0, 0.0, 7.0 * scale, rot);
            },
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_arrow(p: &mut Painter, clr: (f32, f32, f32), x: f32, y: f32, xo: f32, yo: f32, size: f32, rot: f32) {
    p.path_fill_rot(
        clr,
        rot,
        x, y,
        xo + 1.0, yo + 1.0,
        &mut ([
            (0.0_f32, -0.6_f32),
            (0.0,      0.6),
            (1.4,      0.0),
        ].iter().copied()
         .map(|p| ((size * p.0),
                   (size * p.1)))),
        true);
}

fn draw_hexagon<F: FnMut(&mut Painter, HexDecorPos, (f32, f32, f32))>(p: &mut Painter,
    size: f32, line: f32, x: f32, y: f32, clr: (f32, f32, f32), mut decor_fun: F) {

    let (w, h) = hex_size2wh(size);

    let sz = (w, h, size);

    p.path_stroke(
        line,
        clr,
        &mut ([
            (x - 0.50 * w, y          ),
            (x - 0.25 * w, y - 0.5 * h),
            (x + 0.25 * w, y - 0.5 * h),
            (x + 0.50 * w, y          ),
            (x + 0.25 * w, y + 0.5 * h),
            (x - 0.25 * w, y + 0.5 * h),
        ].iter().copied().map(|p| (p.0.floor(), p.1.floor()))), true);

    decor_fun(p,
        HexDecorPos::Center(x.floor(), y.floor()), sz);

    decor_fun(p,
        HexDecorPos::Top(
            x.floor(),
            (y - 0.5 * h).floor(),
        ), sz);

    decor_fun(p,
        HexDecorPos::TopRight(
            (x + 0.75 * size).floor(),
            (y - 0.25 * h   ).floor(),
        ), sz);

    decor_fun(p,
        HexDecorPos::TopLeft(
            (x - 0.75 * size).floor(),
            (y - 0.25 * h   ).floor(),
        ), sz);

    decor_fun(p,
        HexDecorPos::Bottom(
            x.floor(),
            (y + 0.5 * h).floor(),
        ), sz);

    decor_fun(p,
        HexDecorPos::BotRight(
            (x + 0.75 * size).floor(),
            (y + 0.25 * h   ).floor(),
        ), sz);

    decor_fun(p,
        HexDecorPos::BotLeft(
            (x - 0.75 * size).floor(),
            (y + 0.25 * h   ).floor(),
        ), sz);
}

fn draw_led(p: &mut Painter, scale: f32, x: f32, y: f32, led_value: (f32, f32)) {
    let r = UI_GRID_LED_R * scale;
    /*
          ____
         /    \
        /      \
        |  *   |
        |  xy  |
        \      /
         \____/
    */
    let path = &[
        (x - r,                  y - (r * 0.5)),
        (x - (r * 0.5),          y - r),
        (x + (r * 0.5),          y - r),
        (x + r,                  y - (r * 0.5)),

        (x + r,                  y + (r * 0.5)),
        (x + (r * 0.5),          y + r),
        (x - (r * 0.5),          y + r),
        (x - r,                  y + (r * 0.5)),
    ];

    let led_clr_border = (
        UI_GRID_LED_CLR.0 * 0.3,
        UI_GRID_LED_CLR.1 * 0.3,
        UI_GRID_LED_CLR.2 * 0.3,
    );
    let led_clr = (
        led_value.0 as f32,
        led_value.1 as f32,
        0.3,
    );
    p.path_fill(led_clr, &mut path.iter().copied(), true);
    p.path_stroke(1.0 * scale, led_clr_border, &mut path.iter().copied(), true);
}

pub struct HexGrid {
    tile_size: f32,
    scale:     f32,
    scale_step:i32,
    model:     Rc<RefCell<dyn HexGridModel>>,
    center_font_size: f32,
    edge_font_size:   f32,
    y_offs:           bool,

    drag_source_pos:  Option<(i32, i32)>,
    shift_offs:       (f32, f32),
    tmp_shift_offs:   Option<(f32, f32)>,

    start_tile_pos:   Option<(i32, i32)>,
    hover_pos:        (i32, i32),

    real_pos:       Rect,
    mouse:          (f32, f32),
    mouse_state:    Option<(MButton, f32, f32)>,
}

impl HexGrid {
    pub fn new(model: Rc<RefCell<dyn HexGridModel>>) -> Self {
        let tile_size = 54.0_f32;
        let scale = tile_size / 54.0;
        HexGrid {
            center_font_size:   (14.0 * scale).round(),
            edge_font_size:     (10.0 * scale).round(),
            y_offs:             false,
            scale:              1.0,
            scale_step:         0,
            tile_size,
            drag_source_pos:    None,
            shift_offs:         (0.0, 0.0),
            tmp_shift_offs:     None,
            start_tile_pos:     None,
            hover_pos:          (1000, 1000),
            real_pos:           Rect::from(0.0, 0.0, 0.0, 0.0),
            mouse:              (0.0, 0.0),
            mouse_state:        None,
            model,
        }
    }

}

impl HexGrid {
    pub fn mouse_to_tile(&self, x: f32, y: f32) -> (i32, i32) {
        // https://web.archive.org/web/20161024224848/http://gdreflections.com/2011/02/hexagonal-grid-math.html
        let tile_size = self.tile_size * self.scale;
        let side   = ((tile_size * 3.0) / 2.0).floor();
        let radius = tile_size;
        let _width = tile_size * 2.0;
        let height = (tile_size * (3.0_f32).sqrt()).floor();

        let y = if self.y_offs { y + 0.5 * height } else { y };

        let ci = (x / side).floor();
        let cx = x - side * ci;

        let ty = (y - (ci as usize % 2) as f32 * height / 2.0).floor();
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
        (i as i32, j as i32)
    }

    pub fn get_mouse_tile_pos(&self, x: f32, y: f32) -> (i32, i32) {
        let pos = self.real_pos;

        let shift_x =
            (self.shift_offs.0
             + self.tmp_shift_offs.map(|o| o.0).unwrap_or(0.0)).round();
        let shift_y =
            (self.shift_offs.1
             + self.tmp_shift_offs.map(|o| o.1).unwrap_or(0.0)).round();

        self.mouse_to_tile(x - pos.x - shift_x, y - pos.y - shift_y)
    }
}

#[derive(Clone)]
pub enum HexGridMessage {
    SetModel(Rc<RefCell<dyn HexGridModel>>),
}

impl HexGrid {
    pub fn handle(
        &mut self, w: &Widget, event: &InputEvent,
        out_events: &mut Vec<(usize, Event)>)
    {
        let is_hovered = w.is_hovered();

        match event {
            InputEvent::MouseButtonPressed(btn) => {
                if is_hovered {
                    self.mouse_state = Some((*btn, self.mouse.0, self.mouse.1));
                    self.start_tile_pos =
                        Some(self.get_mouse_tile_pos(self.mouse.0, self.mouse.1));

                    w.activate();
                }
            },
            InputEvent::MouseButtonReleased(btn) => {
                if let Some((pres_btn, mx, my)) = self.mouse_state.take() {
                    // Ignore mouse button ups that don't belong
                    if pres_btn != *btn {
                        self.mouse_state = Some((pres_btn, mx, my));
                        return;
                    }
                } else {
                    return;
                }

//                    self.mouse_down_pos = None;
                if *btn == MButton::Middle {
                    if let Some(tmp_shift_offs) = self.tmp_shift_offs.take() {
                        self.shift_offs.0 += tmp_shift_offs.0;
                        self.shift_offs.1 += tmp_shift_offs.1;
                    }
                } else {
                    let cur_tile_pos =
                        self.get_mouse_tile_pos(self.mouse.0, self.mouse.1);

                    if let Some(start_tile_pos) = self.start_tile_pos {
                        if cur_tile_pos == start_tile_pos {
                            if    cur_tile_pos.0 >= 0
                               && cur_tile_pos.1 >= 0
                            {
                                out_events.push((w.id(), Event {
                                    name: "click".to_string(),
                                    data: EvPayload::HexGridClick {
                                        x: cur_tile_pos.0 as usize,
                                        y: cur_tile_pos.1 as usize,
                                        button: *btn,
                                    } }));
                            }

                        } else {
                            if    cur_tile_pos.0 >= 0
                               && cur_tile_pos.1 >= 0
                               && start_tile_pos.0 >= 0
                               && start_tile_pos.1 >= 0
                            {
                                out_events.push((w.id(), Event {
                                    name: "hex_drag".to_string(),
                                    data: EvPayload::HexGridDrag {
                                        x_src: start_tile_pos.0 as usize,
                                        y_src: start_tile_pos.1 as usize,
                                        x_dst: cur_tile_pos.0 as usize,
                                        y_dst: cur_tile_pos.1 as usize,
                                        button: *btn,
                                    } }));
                            }
                        }

                        w.emit_redraw_required();
                    }

                    self.start_tile_pos = None;
                    self.drag_source_pos = None;
                }

                if w.is_active() {
                    w.deactivate();
                }
            },
            InputEvent::MousePosition(x, y) => {
                self.mouse = (*x, *y);

                if let Some((MButton::Middle, mx, my)) = self.mouse_state {
                    self.tmp_shift_offs =
                        Some((
                            *x - mx,
                            *y - my
                        ));

                    w.emit_redraw_required();

                } else {
                    let old_hover_pos = self.hover_pos;

                    self.hover_pos = self.get_mouse_tile_pos(*x, *y);

                    // For left & right mouse clicks:
                    if let Some((_, _mx, _my)) = self.mouse_state {
                        let cur_tile_pos = self.get_mouse_tile_pos(*x, *y);

                        if let Some(start_tile_pos) = self.start_tile_pos {
                            if cur_tile_pos != start_tile_pos {
                                self.drag_source_pos = Some(start_tile_pos);
                            } else {
                                self.drag_source_pos = None;
                            }
                        }
                    }

                    if old_hover_pos != self.hover_pos {
                        w.emit_redraw_required();
                    }
                }
            },
            InputEvent::MouseWheel(y) => {
                if is_hovered {
                    if *y < 0.0 {
                        self.scale_step += 1;
                    } else {
                        self.scale_step -= 1;
                    }

                    let old_shift = self.shift_offs;
                    let old_shift = (
                        old_shift.0 / self.scale,
                        old_shift.1 / self.scale
                    );

                    self.scale = 1.0 + self.scale_step as f32 * 0.25;

                    if self.scale <= 0.001 {
                        self.scale = 0.1;
                    }

                    self.shift_offs = (
                        old_shift.0 * self.scale,
                        old_shift.1 * self.scale
                    );

                    w.emit_redraw_required();
                }
            },
            _ => {},
        }
    }

    pub fn get_generation(&mut self) -> u64 {
        self.model.borrow().get_generation()
    }

//    fn on_draw(&mut self, state: &mut State, entity: Entity, canvas: &mut Canvas) {
    pub fn draw(&mut self, w: &Widget, style: &Style, pos: Rect,
                real_pos: Rect, p: &mut Painter)
    {
        let is_hovered = w.is_hovered();

        let mut dbg = LblDebugTag::from_id(w.id());
        dbg.set_offs((real_pos.x - pos.x, real_pos.y - pos.y));

        self.real_pos = real_pos;

        let size = self.tile_size * self.scale;

        let pad     = 10.0 * self.scale;
        let size_in = size - pad;
        let (w, h)  = hex_size2wh(size);

        p.clip_region(pos.x, pos.y, pos.w, pos.h);
        p.rect_fill_r(UI_GRID_BG1_CLR, pos);

        let model = self.model.borrow();

        let nx = model.width();
        let ny = model.height();

        for xi in 0..nx {
            let x = xi as f32;

            for yi in 0..ny {
                let y =
                    if xi % 2 == 0 { yi as f32 - 0.5 }
                    else           { yi as f32 };

                dbg.set_logic_pos(xi as i32, yi as i32);

                let xo = x * 0.75 * w + size;
                let yo = (1.00 + y) * h;

                let yo = if self.y_offs { yo - 0.5 * h } else { yo };

                let test_pos = Rect {
                    x: -0.5 * w,
                    y: -0.5 * h,
                    w: pos.w + 1.0 * w,
                    h: pos.h + 1.0 * h,
                };
                let shift_x = (self.shift_offs.0 + self.tmp_shift_offs.map(|o| o.0).unwrap_or(0.0)).round();
                let shift_y = (self.shift_offs.1 + self.tmp_shift_offs.map(|o| o.1).unwrap_or(0.0)).round();

//                let test_pos = test_pos.offs(shift_x, shift_y);

                // Assume the tiles are bigger than they are, so we don't miss:
                let tile_size_check_factor = 0.1;
                let w_check_pad = w * tile_size_check_factor;
                let h_check_pad = h * tile_size_check_factor;
                if !test_pos.aabb_is_inside(Rect {
                        x: xo + shift_x - w_check_pad,
                        y: yo + shift_y - h_check_pad,
                        w: w + w_check_pad,
                        h: h + h_check_pad
                    })
                {
//                println!("NOT HEXINSODE {:?} IN {:?}", Rect {
//                    x: xo + shift_x - w_check_pad,
//                    y: yo + shift_y - h_check_pad,
//                    w: w + w_check_pad,
//                    h: h + h_check_pad,
//                }, test_pos);

                    continue;
                }

                if !model.cell_visible(xi, yi) {
                    continue;
                }

                let th  = p.font_height(self.center_font_size * self.scale, false);
                let fs  = self.center_font_size * self.scale;
                let th2 = p.font_height(self.edge_font_size * self.scale, false);
                let fs2 = self.edge_font_size * self.scale;

                let (line, clr) =
                    if is_hovered
                       && self.hover_pos.0 == (xi as i32)
                       && self.hover_pos.1 == (yi as i32)
                    {
                        (5.0, UI_GRID_HOVER_BORDER_CLR)
                    } else  if Some((xi as i32, yi as i32)) == self.drag_source_pos {
                        (3.0, UI_GRID_DRAG_BORDER_CLR)
                    } else if model.cell_empty(xi, yi) {
                        (3.0, UI_GRID_EMPTY_BORDER_CLR)
                    } else {
                        (3.0, hex_color_idx2clr(style, model.cell_color(xi, yi)))
                    };

                p.translate(shift_x, shift_y);

                // padded outer hex
                draw_hexagon(p, size_in, line * self.scale, pos.x + xo, pos.y + yo, clr, |p, pos, sz| {
                    let mut label_buf = [0; 20];

                    match pos {
                        HexDecorPos::Center(x, y) => {
                            p.define_debug_area(Rect {
                                    x: x, y: y,
                                    w: (sz.0 / 3.0).round(),
                                    h: (sz.1 / 3.0).round(),
                                }, || {
                                    (*(dbg.source("hexcell")),
                                     format!("hexcell_{}_{}", xi, yi))
                                });

                            if let Some(cell_vis) = model.cell_label(xi, yi, &mut label_buf) {
                                let (s, hc, led) = (
                                    cell_vis.label,
                                    cell_vis.hlight,
                                    cell_vis.rg_colors
                                );

                                let (txt_clr, clr) =
                                    match hc {
                                        HexHLight::Normal => (UI_GRID_TXT_CENTER_CLR, clr),
                                        HexHLight::Plain  => (UI_GRID_TXT_CENTER_CLR, clr),
                                        HexHLight::Accent => (UI_GRID_TXT_CENTER_CLR, UI_GRID_TXT_CENTER_CLR),
                                        HexHLight::HLight => (UI_GRID_TXT_CENTER_HL_CLR, UI_GRID_TXT_CENTER_HL_CLR),
                                        HexHLight::Select => (UI_GRID_TXT_CENTER_SL_CLR, UI_GRID_TXT_CENTER_SL_CLR),
                                    };

                                let fs =
                                    if hc == HexHLight::Plain { fs * 1.4 }
                                    else { fs };

                                let num_fs = fs * 0.8;
                                let y_inc = -1.0 + p.font_height(fs as f32, false);
                                let mut lbl_it = s.split(' ');

                                if let Some(name_lbl) = lbl_it.next() {
                                    let maxwidth =
                                        if hc == HexHLight::Plain {
                                            (size * 1.3) as f32
                                        } else { (size * 0.82) as f32 };

                                    let fs =
                                        calc_font_size_from_text(
                                            p, name_lbl, fs, maxwidth);

                                    p.label(
                                        fs, 0, txt_clr,
                                        x - 0.5 * sz.0,
                                        y - 0.5 * th,
                                        sz.0, th, name_lbl,
                                        dbg.source("cell_name"));
                                }

                                if let Some(num_lbl) = lbl_it.next() {
                                    p.label(
                                        num_fs, 0, txt_clr,
                                        x - 0.5 * sz.0,
                                        y - 0.5 * th + y_inc,
                                        sz.0, th, num_lbl,
                                        dbg.source("cell_num"));
                                }

                                if let Some(led) = led {
                                    draw_led(p, self.scale, x, y - th, led);
                                }

                                if hc != HexHLight::Plain {
                                    draw_hexagon(
                                        p, size * 0.5, line * 0.5 * self.scale, x, y, clr,
                                        |_p, _pos, _sz| ());
                                }
                            }
                        },
                        HexDecorPos::Top(x, y) => {
                            if let Some((s, _)) = model.cell_edge(xi, yi, HexDir::T, &mut label_buf) {
                                p.label(
                                    fs2, 0, UI_GRID_TXT_EDGE_CLR,
                                    x - 0.5 * sz.0,
                                    y - 1.0,
                                    sz.0, th, s,
                                    dbg.source("cell_top"));
                            }
                        },
                        HexDecorPos::Bottom(x, y) => {
                            if let Some((s, et)) = model.cell_edge(xi, yi, HexDir::B, &mut label_buf) {
                                p.label(
                                    fs2, 0, UI_GRID_TXT_EDGE_CLR,
                                    x - 0.5 * sz.0,
                                    y - th,
                                    sz.0, th, s,
                                    dbg.source("cell_bottom"));

                                et.draw(p, self.scale, x, y, 90.0);
                            }
                        },
                        HexDecorPos::TopLeft(x, y) => {
                            if let Some((s, _)) = model.cell_edge(xi, yi, HexDir::TL, &mut label_buf) {
                                p.label_rot(
                                    fs2, 0, 300.0, UI_GRID_TXT_EDGE_CLR,
                                    (x - 0.5 * sz.0).floor(),
                                    (y - 0.5 * th2).floor(),
                                    0.0,
                                    (0.5 * th2).floor() + 2.0,
                                    sz.0, th2, s,
                                    dbg.source("cell_top_left"));
                            }
                        },
                        HexDecorPos::TopRight(x, y) => {
                            if let Some((s, et)) = model.cell_edge(xi, yi, HexDir::TR, &mut label_buf) {
                                p.label_rot(
                                    fs2, 0, 60.0, UI_GRID_TXT_EDGE_CLR,
                                    (x - 0.5 * sz.0).floor(),
                                    (y - 0.5 * th2).floor(),
                                    0.0,
                                    (0.5 * th2).floor() + 2.0,
                                    sz.0, th2, s,
                                    dbg.source("cell_top_right"));

                                et.draw(p, self.scale, x, y, -30.0);
                            }
                        },
                        HexDecorPos::BotLeft(x, y) => {
                            if let Some((s, _)) = model.cell_edge(xi, yi, HexDir::BL, &mut label_buf) {
                                p.label_rot(
                                    fs2, 0, 60.0, UI_GRID_TXT_EDGE_CLR,
                                    (x - 0.5 * sz.0).floor(),
                                    (y - 0.5 * th2).floor(),
                                    0.0,
                                    -(0.5 * th2).floor() - 2.0,
                                    sz.0, th2, s,
                                    dbg.source("cell_bottom_left"));
                            }
                        },
                        HexDecorPos::BotRight(x, y) => {
                            if let Some((s, et)) = model.cell_edge(xi, yi, HexDir::BR, &mut label_buf) {
                                p.label_rot(
                                    fs2, 0, 300.0, UI_GRID_TXT_EDGE_CLR,
                                    (x - 0.5 * sz.0).floor(),
                                    (y - 0.5 * th2).floor(),
                                    0.0,
                                    -(0.5 * th2).floor() - 2.0,
                                    sz.0, th2, s,
                                    dbg.source("cell_bottom_right"));

                                et.draw(p, self.scale, x, y, 30.0);
                            }
                        },
                    }
                });

                p.restore();
            }
        }

        p.reset_clip_region();
    }

    pub fn annotate_drop_event(&mut self, mouse_pos: (f32, f32), ev: Event) -> Event {
        let cur_tile_pos = self.get_mouse_tile_pos(mouse_pos.0, mouse_pos.1);
        if let EvPayload::UserData(data) = ev.data {
            Event {
                name: ev.name,
                data: EvPayload::HexGridDropData {
                    x: cur_tile_pos.0 as usize,
                    y: cur_tile_pos.1 as usize,
                    data: data,
                },
            }
        } else {
            ev
        }
    }
}
