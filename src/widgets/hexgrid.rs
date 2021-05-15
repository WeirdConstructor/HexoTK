use super::*;
use crate::constants::*;
use std::rc::Rc;

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
        let e = self.to_edge();
        e <= 2
    }

    #[inline]
    pub fn is_left_half(&self) -> bool {
        !self.is_right_half()
    }

    #[inline]
    pub fn to_edge(&self) -> u8 {
        *self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum HexEdge {
    NoArrow,
    Arrow,
    ArrowValue { value: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum HexCell {
    Normal,
    Plain,
    HLight,
    Select,
}

pub trait HexGridModel {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn cell_visible(&self, x: usize, y: usize) -> bool;
    fn cell_empty(&self, x: usize, y: usize) -> bool;
    fn cell_label<'a>(&self, x: usize, y: usize, out: &'a mut [u8]) -> Option<(&'a str, HexCell, Option<(f32, f32)>)>;
    /// Edge: 0 top-right, 1 bottom-right, 2 bottom, 3 bottom-left, 4 top-left, 5 top
    fn cell_edge<'a>(&self, x: usize, y: usize, edge: HexDir, out: &'a mut [u8]) -> Option<(&'a str, HexEdge)>;
    fn cell_click(&self, x: usize, y: usize, btn: MButton, modkey: bool);
    fn cell_hover(&self, _x: usize, _y: usize) { }
}

#[derive(Debug, Clone)]
pub struct HexGrid {
    center_font_size: f64,
    edge_font_size:   f64,
    bg_color:         (f64, f64, f64),
    y_offs:           bool,
    transformable:    bool,
    cell_size:        f64,
}

impl HexGrid {
    pub fn new(center_font_size: f64, edge_font_size: f64, cell_size: f64) -> Self {
        Self {
            center_font_size,
            edge_font_size,
            bg_color:   UI_GRID_BG1_CLR,
            y_offs:     false,
            transformable: true,
            cell_size,
        }
    }

    pub fn new_y_offs_pinned(center_font_size: f64, edge_font_size: f64, cell_size: f64) -> Self {
        Self {
            center_font_size,
            edge_font_size,
            bg_color:       UI_GRID_BG1_CLR,
            y_offs:         true,
            transformable:  false,
            cell_size,
        }
    }

    pub fn bg_color(mut self, clr: (f64, f64, f64)) -> Self {
        self.bg_color = clr;
        self
    }
}

#[derive(Clone)]
pub struct HexGridData {
    model:          Rc<dyn HexGridModel>,
    last_hover_pos: (usize, usize),
    hex_trans:      HexGridTransform,
}

impl HexGridData {
    pub fn new(model: Rc<dyn HexGridModel>) -> Box<Self> {
        Box::new(Self { model, last_hover_pos: (0, 0), hex_trans: HexGridTransform::new() })
    }
}

fn hex_size2wh(size: f64) -> (f64, f64) {
    (2.0_f64 * size, (3.0_f64).sqrt() * size)
}

fn hex_at_is_inside(x: f64, y: f64, w: f64, h: f64, pos: Rect) -> bool {
    let aabb = Rect {
        x: x - 0.5 * w,
        y: y - 0.5 * h,
        w: w,
        h: h,
    };

    pos.aabb_is_inside(aabb)
}

enum HexDecorPos {
    Center(f64, f64),
    Top(f64, f64),
    TopLeft(f64, f64),
    TopRight(f64, f64),
    Bottom(f64, f64),
    BotLeft(f64, f64),
    BotRight(f64, f64),
}

impl HexEdge {
    fn draw(&self, p: &mut dyn Painter, x: f64, y: f64, rot: f64) {
        match self {
            HexEdge::NoArrow => {},
            HexEdge::Arrow => {
                draw_arrow(p, UI_GRID_TXT_EDGE_CLR, x, y, 0.0, 0.0, 10.0, rot);
            },
            HexEdge::ArrowValue { value } => {
                draw_arrow(p, UI_GRID_SIGNAL_OUT_CLR, x, y, 0.0, 0.0, 10.0, rot);
                let clr = (
                    (*value as f64).max(0.0).abs(),
                    (*value as f64).min(0.0).abs(),
                    0.3,
                );
                draw_arrow(p, clr, x, y, 1.0, 0.0, 7.5, rot);
            },
        }
    }
}

fn draw_arrow(p: &mut dyn Painter, clr: (f64, f64, f64), x: f64, y: f64, xo: f64, yo: f64, size: f64, rot: f64) {
    p.path_fill_rot(
        clr,
        rot,
        x, y,
        xo + 1.0, yo + 1.0,
        &mut ([
            (0.0_f64, -0.6_f64),
            (0.0,      0.6),
            (1.4,      0.0),
        ].iter().copied()
         .map(|p| ((size * p.0),
                   (size * p.1)))),
        true);
}

fn draw_hexagon<F: Fn(&mut dyn Painter, HexDecorPos, (f64, f64, f64))>(p: &mut dyn Painter,
    size: f64, line: f64, x: f64, y: f64, clr: (f64, f64, f64), decor_fun: F) {

    let (w, h) = hex_size2wh(size);

    let sz = (w, h, size);

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
}

fn draw_led(p: &mut dyn Painter, x: f64, y: f64, led_value: (f32, f32)) {
    let r = UI_GRID_LED_R;
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
        led_value.0 as f64,
        led_value.1 as f64,
        0.3,
    );
    p.path_fill(led_clr, &mut path.iter().copied(), true);
    p.path_stroke(1.0, led_clr_border, &mut path.iter().copied(), true);
}

impl WidgetType for HexGrid {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let size = self.cell_size;

        let pad     = 10.0;
        let size_in = size - pad;
        let (w, h)  = hex_size2wh(size);

        let drag_source =
            if let Some(drag_az) = ui.drag_zone_for(data.id()) {
                if let ZoneType::HexFieldClick { pos, ..} = drag_az.zone_type {
                    Some(pos)
                } else {
                    None
                }
            } else {
                None
            };

        let marked =
            if let Some(az) = ui.hover_zone_for(data.id()) {
                if let ZoneType::HexFieldClick { pos, ..} = az.zone_type {
                    data.with(|data: &mut HexGridData| {
                        if data.last_hover_pos != pos {
                            data.last_hover_pos = pos;
                            data.model.cell_hover(pos.0, pos.1);
                        }
                    });

                    pos
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            };

        let id = data.id();

        data.with(|data: &mut HexGridData| {
            p.rect_fill(
                self.bg_color,
                pos.x, pos.y,
                pos.w, pos.h);

            let nx = data.model.width();
            let ny = data.model.height();

            if let Some(ht) = ui.get_hex_transform(id) {
                if self.transformable {
                    let scale = ht.scale().clamp(0.5, 2.0);
                    data.hex_trans = ht.set_scale(scale);
                }
            }

            ui.define_active_zone(
                ActiveZone::new_hex_field(
                    id,
                    pos,
                    self.y_offs,
                    data.hex_trans,
                    size));

            let (scroll_x, scroll_y) = (
                data.hex_trans.x_offs(),
                data.hex_trans.y_offs()
            );
            //d// println!("scroll_x={}, scroll_y={}", scroll_x, scroll_y);
            let scale = data.hex_trans.scale();

            p.clip_region(pos.x, pos.y, pos.w, pos.h);
            let (mv_x, mv_y) = (
                pos.w * 0.5 + scroll_x * scale,
                pos.h * 0.5 + scroll_y * scale
            );

            p.move_and_scale(pos.x + mv_x, pos.y + mv_y, 0.0, 0.0, scale);

            let pos = Rect {
                x: - pos.w * 0.5,
                y: - pos.h * 0.5,
                w: pos.w,
                h: pos.h,
            };

            for xi in 0..nx {
                let x = xi as f64;

                for yi in 0..ny {
                    let y =
                        if xi % 2 == 0 { yi as f64 - 0.5 }
                        else           { yi as f64 };

                    let xo = pos.x + x * 0.75 * w + size;
                    let yo = pos.y + (1.00 + y) * h;

                    let yo = if self.y_offs { yo - 0.5 * h } else { yo };

                    let spos = Rect {
                        x: 0.0,
                        y: 0.0,
                        w: pos.w,
                        h: pos.h,
                    };

                    // Assume the tiles are bigger than they are, so we don't miss:
                    let tile_size_check_factor = 0.1;
                    let w_check_pad = w * tile_size_check_factor;
                    let h_check_pad = w * tile_size_check_factor;
                    if !hex_at_is_inside(
                            xo * scale + mv_x - w_check_pad * scale,
                            yo * scale + mv_y - h_check_pad * scale,
                            (w + w_check_pad) * scale,
                            (h + h_check_pad) * scale,
                            spos)
                    {
                        continue;
                    }

                    if !data.model.cell_visible(xi, yi) {
                        continue;
                    }

                    let th  = p.font_height(self.center_font_size as f32, false) as f64;
                    let fs  = self.center_font_size;
                    let th2 = p.font_height(self.edge_font_size as f32, false) as f64;
                    let fs2 = self.edge_font_size;

                    let (line, clr) =
                        if marked.0 == xi && marked.1 == yi {
                            (5.0, UI_GRID_HOVER_BORDER_CLR)
                        } else {
                            if Some((xi, yi)) == drag_source {
                                (3.0, UI_GRID_DRAG_BORDER_CLR)
                            } else if data.model.cell_empty(xi, yi) {
                                (3.0, UI_GRID_EMPTY_BORDER_CLR)
                            } else {
                                (3.0, UI_GRID_CELL_BORDER_CLR)
                            }
                        };

                    // padded outer hex
                    draw_hexagon(p, size_in, line, xo, yo, clr, |p, pos, sz| {
                        let mut label_buf = [0; 20];

                        match pos {
                            HexDecorPos::Center(x, y) => {
                                if let Some((s, hc, led)) = data.model.cell_label(xi, yi, &mut label_buf) {
                                    let (txt_clr, clr) =
                                        match hc {
                                            HexCell::Normal => (UI_GRID_TXT_CENTER_CLR, clr),
                                            HexCell::Plain  => (UI_GRID_TXT_CENTER_CLR, clr),
                                            HexCell::HLight => (UI_GRID_TXT_CENTER_HL_CLR, UI_GRID_TXT_CENTER_HL_CLR),
                                            HexCell::Select => (UI_GRID_TXT_CENTER_SL_CLR, UI_GRID_TXT_CENTER_SL_CLR),
                                        };

                                    let fs =
                                        if hc == HexCell::Plain { fs * 1.4 }
                                        else { fs };

                                    let num_fs = fs * 0.8;
                                    let y_inc = -1.0 + p.font_height(fs as f32, false) as f64;
                                    let mut lbl_it = s.split(" ");

                                    if let Some(name_lbl) = lbl_it.next() {
                                        p.label(
                                            fs, 0, txt_clr,
                                            x - 0.5 * sz.0,
                                            y - 0.5 * th,
                                            sz.0, th, name_lbl);
                                    }

                                    if let Some(num_lbl) = lbl_it.next() {
                                        p.label(
                                            num_fs, 0, txt_clr,
                                            x - 0.5 * sz.0,
                                            y - 0.5 * th + y_inc,
                                            sz.0, th, num_lbl);
                                    }

                                    if let Some(led) = led {
                                        draw_led(p, x, y - th, led);
                                    }

                                    if hc != HexCell::Plain {
                                        draw_hexagon(
                                            p, size * 0.5, line * 0.5, x, y, clr,
                                            |_p, _pos, _sz| ());
                                    }
                                }
                            },
                            HexDecorPos::Top(x, y) => {
                                if let Some((s, _)) = data.model.cell_edge(xi, yi, HexDir::T, &mut label_buf) {
                                    p.label(
                                        fs2, 0, UI_GRID_TXT_EDGE_CLR,
                                        x - 0.5 * sz.0,
                                        y - 1.0,
                                        sz.0, th, s);
                                }
                            },
                            HexDecorPos::Bottom(x, y) => {
                                if let Some((s, et)) = data.model.cell_edge(xi, yi, HexDir::B, &mut label_buf) {
                                    p.label(
                                        fs2, 0, UI_GRID_TXT_EDGE_CLR,
                                        x - 0.5 * sz.0,
                                        y - th,
                                        sz.0, th, s);

                                    et.draw(p, x, y, 90.0);
                                }
                            },
                            HexDecorPos::TopLeft(x, y) => {
                                if let Some((s, _)) = data.model.cell_edge(xi, yi, HexDir::TL, &mut label_buf) {
                                    p.label_rot(
                                        fs2, 0, 300.0, UI_GRID_TXT_EDGE_CLR,
                                        (x - 0.5 * sz.0).floor(),
                                        (y - 0.5 * th2).floor(),
                                        0.0,
                                        (0.5 * th2).floor() + 2.0,
                                        sz.0, th2, s);
                                }
                            },
                            HexDecorPos::TopRight(x, y) => {
                                if let Some((s, et)) = data.model.cell_edge(xi, yi, HexDir::TR, &mut label_buf) {
                                    p.label_rot(
                                        fs2, 0, 60.0, UI_GRID_TXT_EDGE_CLR,
                                        (x - 0.5 * sz.0).floor(),
                                        (y - 0.5 * th2).floor(),
                                        0.0,
                                        (0.5 * th2).floor() + 2.0,
                                        sz.0, th2, s);

                                    et.draw(p, x, y, -30.0);
                                }
                            },
                            HexDecorPos::BotLeft(x, y) => {
                                if let Some((s, _)) = data.model.cell_edge(xi, yi, HexDir::BL, &mut label_buf) {
                                    p.label_rot(
                                        fs2, 0, 60.0, UI_GRID_TXT_EDGE_CLR,
                                        (x - 0.5 * sz.0).floor(),
                                        (y - 0.5 * th2).floor(),
                                        0.0,
                                        -(0.5 * th2).floor() - 2.0,
                                        sz.0, th2, s);
                                }
                            },
                            HexDecorPos::BotRight(x, y) => {
                                if let Some((s, et)) = data.model.cell_edge(xi, yi, HexDir::BR, &mut label_buf) {
                                    p.label_rot(
                                        fs2, 0, 300.0, UI_GRID_TXT_EDGE_CLR,
                                        (x - 0.5 * sz.0).floor(),
                                        (y - 0.5 * th2).floor(),
                                        0.0,
                                        -(0.5 * th2).floor() - 2.0,
                                        sz.0, th2, s);

                                    et.draw(p, x, y, 30.0);
                                }
                            },
                        }
                    });
                }
            }

            p.reset_scale();
            p.reset_clip_region();
        });
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        if let UIEvent::Click { id, button, .. } = ev {
            if let Some(az) = ui.hover_zone_for(data.id()) {
                if az.id == data.id() && *id == data.id() {
                    if let ZoneType::HexFieldClick { pos, .. } = az.zone_type {
                        data.with(|data: &mut HexGridData| {
                            data.model.cell_click(
                                pos.0, pos.1, *button,
                                ui.is_key_pressed(UIKey::Ctrl));
                        });
                    }
                }
            }
        }
    }
}
