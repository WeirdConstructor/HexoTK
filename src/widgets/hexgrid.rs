use super::*;
use crate::constants::*;
use std::sync::Arc;

pub trait HexGridModel {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn cell_visible(&self, x: usize, y: usize) -> bool;
    fn cell_empty(&self, x: usize, y: usize) -> bool;
    fn cell_label<'a>(&self, x: usize, y: usize, out: &'a mut [u8]) -> Option<&'a str>;
    // Edge: 0 top, 1 top right, ... 3 bottom, 5 top left
    fn cell_edge<'a>(&self, x: usize, y: usize, edge: u8, out: &'a mut [u8]) -> Option<&'a str>;
    fn cell_click(&self, x: usize, y: usize, btn: MButton);
}

#[derive(Debug, Clone)]
pub struct HexGrid {
    center_font_size: f64,
    edge_font_size:   f64,
}

impl HexGrid {
    pub fn new(center_font_size: f64, edge_font_size: f64) -> Self {
        Self { center_font_size, edge_font_size }
    }
}

#[derive(Clone)]
pub struct HexGridData {
    model: Arc<dyn HexGridModel>,
}

impl HexGridData {
    pub fn new(model: Arc<dyn HexGridModel>) -> Box<Self> {
        Box::new(Self { model })
    }
}

fn hex_size2wh(size: f64) -> (f64, f64) {
    (2.0_f64 * size, (3.0_f64).sqrt() * size)
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

fn draw_arrow(p: &mut dyn Painter, clr: (f64, f64, f64), x: f64, y: f64, size: f64, rot: f64) {
    p.path_fill_rot(
        clr,
        rot,
        x, y,
        1.0, 1.0,
        &mut ([
            (0.0_f64, -0.6_f64),
            (0.0,      0.6),
            (1.4,      0.0),
        ].iter().copied()
         .map(|p| ((size * p.0).floor(),
                   (size * p.1).floor()))),
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

impl WidgetType for HexGrid {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let size = 54.0_f64;

        ui.define_active_zone(ActiveZone::new_hex_field(data.id(), pos, size));

        let pad     = 10.0;
        let size_in = size - pad;
        let (w, h)  = hex_size2wh(size);

        let marked =
            if let Some(az) = ui.hover_zone_for(data.id()) {
                if let ZoneType::HexFieldClick { pos, ..} = az.zone_type {
                    pos
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            };

        data.with(|data: &mut HexGridData| {
            p.rect_fill(
                (32.0 / 255.0, 14.0 / 255.0, 31.0 / 255.0),
                pos.x, pos.y,
                pos.w, pos.h);

            //// Calculate the number of hexagons fitting into the pos Rect:
            //let nx = ((pos.w - (0.5 * w)) / (0.75 * w)).floor() as usize;
            //let ny = ((pos.h - (0.5 * h)) / h).floor() as usize;
            let nx = data.model.width();
            let ny = data.model.height();

            for xi in 0..nx {
                let x = xi as f64;

                for yi in 0..ny {
                    let y =
                        if xi % 2 == 0 { yi as f64 - 0.5 }
                        else           { yi as f64 };

                    if !data.model.cell_visible(xi, yi) {
                        continue;
                    }

                    let (line, clr) =
                        if marked.0 == xi && marked.1 == yi {
                            (5.0, UI_GRID_HOVER_BORDER_CLR)
                        } else {
                            if data.model.cell_empty(xi, yi) {
                                (3.0, UI_GRID_EMPTY_BORDER_CLR)
                            } else {
                                (3.0, UI_GRID_CELL_BORDER_CLR)
                            }
                        };

                    let xo = pos.x + x * 0.75 * w + size;
                    let yo = pos.y + (1.00 + y) * h;

                    let th  = p.font_height(self.center_font_size as f32, false) as f64;
                    let fs  = self.center_font_size;
                    let th2 = p.font_height(self.edge_font_size as f32, false) as f64;
                    let fs2 = self.edge_font_size;

                    // padded outer hex
                    draw_hexagon(p, size_in, line, xo, yo, clr, |p, pos, sz| {
                        let mut label_buf = [0; 20];

                        match pos {
                            HexDecorPos::Center(x, y) => {
                                if let Some(s) = data.model.cell_label(xi, yi, &mut label_buf) {
                                    p.label(
                                        fs, 0, UI_GRID_TXT_CENTER_CLR,
                                        x - 0.5 * sz.0,
                                        y - 0.5 * th,
                                        sz.0, th, s);

                                    draw_hexagon(
                                        p, size * 0.5, line * 0.5, x, y, clr,
                                        |_p, _pos, _sz| ());
                                }
                            },
                            HexDecorPos::Top(x, y) => {
                                if let Some(s) = data.model.cell_edge(xi, yi, 0, &mut label_buf) {
                                    p.label(
                                        fs2, 0, UI_GRID_TXT_EDGE_CLR,
                                        x - 0.5 * sz.0,
                                        y - 2.0,
                                        sz.0, th, s);
                                }
                            },
                            HexDecorPos::Bottom(x, y) => {
                                if let Some(s) = data.model.cell_edge(xi, yi, 3, &mut label_buf) {
                                    p.label(
                                        fs2, 0, UI_GRID_TXT_EDGE_CLR,
                                        x - 0.5 * sz.0,
                                        y - th + 1.0,
                                        sz.0, th, s);

                                    draw_arrow(p, UI_GRID_TXT_EDGE_CLR, x, y, fs2, 90.0);
                                }
                            },
                            HexDecorPos::TopLeft(x, y) => {
                                if let Some(s) = data.model.cell_edge(xi, yi, 5, &mut label_buf) {
                                    p.label_rot(
                                        fs2, 0, 300.0, UI_GRID_TXT_EDGE_CLR,
                                        (x - 0.5 * sz.0).floor(),
                                        (y - 0.5 * th2).floor(),
                                        0.0,
                                        (0.5 * th2).floor() + 1.0,
                                        sz.0, th2, s);
                                }
                            },
                            HexDecorPos::TopRight(x, y) => {
                                if let Some(s) = data.model.cell_edge(xi, yi, 1, &mut label_buf) {
                                    p.label_rot(
                                        fs2, 0, 60.0, UI_GRID_TXT_EDGE_CLR,
                                        (x - 0.5 * sz.0).floor(),
                                        (y - 0.5 * th2).floor(),
                                        0.0,
                                        (0.5 * th2).floor() + 1.0,
                                        sz.0, th2, s);

                                    draw_arrow(p, UI_GRID_TXT_EDGE_CLR, x, y, 10.0, -30.0);
                                }
                            },
                            HexDecorPos::BotLeft(x, y) => {
                                if let Some(s) = data.model.cell_edge(xi, yi, 4, &mut label_buf) {
                                    p.label_rot(
                                        fs2, 0, 60.0, UI_GRID_TXT_EDGE_CLR,
                                        (x - 0.5 * sz.0).floor(),
                                        (y - 0.5 * th2).floor(),
                                        0.0,
                                        -(0.5 * th2).floor() + 1.0,
                                        sz.0, th2, s);
                                }
                            },
                            HexDecorPos::BotRight(x, y) => {
                                if let Some(s) = data.model.cell_edge(xi, yi, 2, &mut label_buf) {
                                    p.label_rot(
                                        fs2, 0, 300.0, UI_GRID_TXT_EDGE_CLR,
                                        (x - 0.5 * sz.0).floor(),
                                        (y - 0.5 * th2).floor(),
                                        0.0,
                                        -(0.5 * th2).floor(),
                                        sz.0, th2, s);

                                    draw_arrow(p, UI_GRID_TXT_EDGE_CLR, x, y, 10.0, 30.0);
                                }
                            },
                        }
                    });
                }
            }
        });
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        println!("HEX GRID DISPATCHED EVENT: {:?}", ev);
        if let Some(az) = ui.hover_zone_for(data.id()) {
            println!("X {:?}", az);
        }
    }
}
