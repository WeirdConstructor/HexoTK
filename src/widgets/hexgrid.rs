use super::*;
use std::sync::Arc;

use femtovg::{
    renderer::OpenGl,
    Canvas,
    FontId,
    ImageId,
    Color,
};

#[derive(Debug, Clone)]
pub enum CellEdge {
    Top,
    TopRight,
    BotRight,
    Bottom,
    BotLeft,
    TopLeft
}

pub trait HexGridModel {
    fn cell_visible(&self, x: usize, y: usize) -> bool;
    fn cell_label(&self, x: usize, y: usize, out: &mut [u8]);
    fn cell_edge_connection(&self, x: usize, y: usize, edge: u8, out: &mut [u8]) -> bool;
}

#[derive(Debug, Clone)]
pub struct HexGrid {
}

#[derive(Clone)]
pub struct HexGridData {
    model: Arc<dyn HexGridModel>,
}

impl HexGridData {
    pub fn new(model: Arc<dyn HexGridModel>) -> Self {
        Self { model }
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

// TODO: Make the HexGrid use a trait to determine the contents of the hex cells
// TODO: Develop a menu eg. from HexGrid (limiting the visible cells by the trait)
impl WidgetType for HexGrid {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let size = 54.0_f64;

        ui.define_active_zone(ActiveZone::new_hex_field(11, pos, size));

        let pad     = 10.0;
        let size_in = size - pad;
        let (w, h)  = hex_size2wh(size);

        let marked =
            if let Some(az) = ui.hover_zone_for(11) {
                if let ZoneType::HexFieldClick { pos, ..} = az.zone_type {
                    pos
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            };

        let txt_clr = (81.0 / 255.0, 162.0 / 255.0, 171.0 / 255.0);

        data.with(|data: &mut HexGridData| {
            p.rect_fill(
                (32.0 / 255.0, 14.0 / 255.0, 31.0 / 255.0),
                pos.x, pos.y,
                pos.w, pos.h);

            // Calculate the number of hexagons fitting into the pos Rect:
            let nx = ((pos.w - (0.5 * w)) / (0.75 * w)).floor() as usize;
            let ny = ((pos.h - (0.5 * h)) / h).floor() as usize;

            for xi in 0..nx {
                let x = xi as f64;

                for yi in 0..ny {
                    let y =
                        if xi % 2 == 0 { yi as f64 - 0.5 }
                        else           { yi as f64 };

                    let (line, clr) =
                        if marked.0 == xi && marked.1 == yi {
                            (5.0, (120.0 / 255.0, 13.0 / 255.0, 114.0 / 255.0))
                        } else {
                            (3.0, (77.0 / 255.0, 24.0 / 255.0, 74.0 / 255.0))
                        };

                    let xo = pos.x + x * 0.75 * w + size;
                    let yo = pos.y + (1.00 + y) * h;

                    let tw = w;
                    let th  = 15.0;
                    let th2 = 14.0;

                    // padded outer hex
                    draw_hexagon(p, size_in, line, xo, yo, clr, |p, pos, sz| {
                        match pos {
                            HexDecorPos::Center(x, y) => {
                                p.label(
                                    15.0, 0, txt_clr,
                                    x - 0.5 * sz.0,
                                    y - 0.5 * th,
                                    sz.0, th, "Env 1");

                                draw_hexagon(p, size * 0.5, line * 0.5, x, y, clr, |p, pos, sz| ());
                            },
                            HexDecorPos::Top(x, y) => {
                                p.label(
                                    10.0, 0, txt_clr,
                                    x - 0.5 * sz.0,
                                    y - 1.0,
                                    sz.0, th, "Top");
                            },
                            HexDecorPos::Bottom(x, y) => {
                                p.label(
                                    10.0, 0, txt_clr,
                                    x - 0.5 * sz.0,
                                    y - th,
                                    sz.0, th, "Bot");

                                draw_arrow(p, txt_clr, x, y, 10.0, 90.0);
                            },
                            HexDecorPos::TopLeft(x, y) => {
                                p.label_rot(
                                    10.0, 0, 300.0, txt_clr,
                                    (x - 0.5 * sz.0).floor(),
                                    (y - 0.5 * th2).floor(),
                                    0.0,
                                    (0.5 * th2).floor() - 1.0,
                                    sz.0, th2, "TL");
                            },
                            HexDecorPos::TopRight(x, y) => {
                                p.label_rot(
                                    10.0, 0, 60.0, txt_clr,
                                    (x - 0.5 * sz.0).floor(),
                                    (y - 0.5 * th2).floor(),
                                    0.0,
                                    (0.5 * th2).floor(),
                                    sz.0, th2, "TR");

                                draw_arrow(p, txt_clr, x, y, 10.0, -30.0);
                            },
                            HexDecorPos::BotLeft(x, y) => {
                                p.label_rot(
                                    10.0, 0, 60.0, txt_clr,
                                    (x - 0.5 * sz.0).floor(),
                                    (y - 0.5 * th2).floor(),
                                    0.0,
                                    -(0.5 * th2).floor() + 1.0,
                                    sz.0, th2, "BL");
                            },
                            HexDecorPos::BotRight(x, y) => {
                                p.label_rot(
                                    10.0, 0, 300.0, txt_clr,
                                    (x - 0.5 * sz.0).floor(),
                                    (y - 0.5 * th2).floor(),
                                    0.0,
                                    -(0.5 * th2).floor(),
                                    sz.0, th2, "BR");

                                draw_arrow(p, txt_clr, x, y, 10.0, 30.0);
                            },
                            _ => {},
                        }
                    });
                }
            }
        });
    }

    fn size(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData) -> (f64, f64) { (0.0, 0.0) }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: UIEvent) {
        println!("HEX GRID DISPATCHED EVENT: {:?}", ev);
//        if ev.id() == 10 {
//            match ev {
//                UIEvent::Click { .. } => {
//                    data.with(|data: &mut ButtonData| data.counter += 1);
//                },
//                _ => {}
//            }
//        }
    }
}
