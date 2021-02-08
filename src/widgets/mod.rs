// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use super::*;

#[derive(Debug)]
pub struct Container {
    w: f64,
}

impl WidgetType for Container {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {}
    fn size(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData) -> (f64, f64) { (0.0, 0.0) }
    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: UIEvent) {}
}


#[derive(Debug, Clone)]
pub struct Button {
}

#[derive(Debug, Clone)]
pub struct ButtonData {
    pub label: String,
    /// TODO: Buttons don't store the value themself, they should access
    ///       a shared trait object for parameter access!
    ///       The trait will be used:
    ///         - for accessing the string representation of the value.
    ///         - for sending value changes.
    ///       For the matrix we will define a different kind of data model trait.
    pub counter: usize,
}

impl WidgetType for Button {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        ui.define_active_zone(ActiveZone::new_click_zone(10, pos));
        let pos2 = pos.offs(0.0, 20.0);
        ui.define_active_zone(ActiveZone::new_drag_zone(10, pos2, true));

        let hl = ui.hl_style_for(10);

        data.with(|data: &mut ButtonData| {
            let clr =
                match hl {
                    HLStyle::Hover(_) => (1.0, 1.0, 0.0),
                    _                 => (1.0, 0.0, 1.0),
                };

            p.label(20.0, 0, clr, pos.x, pos.y, pos.w, pos.h, &data.label);
            p.label(20.0, 1, clr, pos.x, pos.y + 20.0, pos.w, pos.h, &format!("VL: {}", data.counter));
            p.label(20.0,-1, clr, pos.x, pos.y + 40.0, pos.w, pos.h,
                &format!("V: {:6.4}", ui.params().get(10)));
        });
    }

    fn size(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData) -> (f64, f64) { (0.0, 0.0) }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: UIEvent) {
        println!("DISPATCHED EVENT: {:?}", ev);
        if ev.id() == 10 {
            match ev {
                UIEvent::Click { .. } => {
                    data.with(|data: &mut ButtonData| data.counter += 1);
                },
                _ => {}
            }
        }
    }
}


use femtovg::{
    renderer::OpenGl,
    Canvas,
    FontId,
    ImageId,
    Color,
};

#[derive(Debug, Clone)]
pub struct HexGrid {
}

#[derive(Debug, Clone)]
pub struct HexGridData {
}

impl WidgetType for HexGrid {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let size = 60.0_f64;

        ui.define_active_zone(ActiveZone::new_hex_field(11, pos, size));
//        let pos2 = pos.offs(0.0, 20.0);
//        ui.define_active_zone(ActiveZone::new_drag_zone(10, pos2, true));

//        let hl = ui.hl_style_for(10);
        let pad  = 10.0;
        let w    = 2.0_f64          * size;
        let h    = (3.0_f64).sqrt() * size;
        let w_in = 2.0_f64          * (size - pad);
        let h_in = (3.0_f64).sqrt() * (size - pad);

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

        data.with(|data: &mut HexGridData| {
            p.rect_fill(
                (32.0 / 255.0, 14.0 / 255.0, 31.0 / 255.0),
                pos.x, pos.y,
                pos.w, pos.h);

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

                    let pad  = 12.0;
                    let padw = pad;
                    let padh = 0.5 * (3.0_f64).sqrt() * pad;

                    let xo = pos.x + x * 0.75 * w + size;
                    let yo = pos.y + (1.00 + y) * h;

                    p.path_stroke(
                        line,
                        clr,
                        &mut ([
                            (xo - 0.50 * w_in, yo          ),
                            (xo - 0.25 * w_in, yo - 0.5 * h_in),
                            (xo + 0.25 * w_in, yo - 0.5 * h_in),
                            (xo + 0.50 * w_in, yo          ),
                            (xo + 0.25 * w_in, yo + 0.5 * h_in),
                            (xo - 0.25 * w_in, yo + 0.5 * h_in),
                        ].iter().copied().map(|p| (p.0.floor(), p.1.floor()))), true);
//                    p.rect_fill((1.0, 1.0, 1.0), 
//                        (pos.x + x * 0.75 * w).floor(),
//                        (pos.y + h + y * h).floor(),
//                        2.0, 2.0);
                    let th = 20.0;
                    let tw = w;
                    let center = (xo - 0.5 * tw, yo - 0.5 * th);


                    p.label_rot(
                        20.0, 0, 60.0, (81.0 / 255.0, 162.0 / 255.0, 171.0 / 255.0),
                        center.0, center.1, w, th, "CENTER");


//                    p.label_rot(
//                        20.0, 0, 300.0, (81.0 / 255.0, 162.0 / 255.0, 171.0 / 255.0),
//                        center.0.floor() + (0.75) * size - pad,
//                        center.1.floor() - 0.25 * h,
//                        w, th, "BR");

//                    p.label_rot(
//                        // center
//                        // 20.0, 0, 0.0, (81.0 / 255.0, 162.0 / 255.0, 171.0 / 255.0),
//                        // (pos.x + x * 0.75 * w).floor(),
//                        // (pos.y + h + y * h - th * 0.5).floor(),
//
//                        // top right
//                        20.0, 0, 60.0, (81.0 / 255.0, 162.0 / 255.0, 171.0 / 255.0),
//                        (pos.x + x * 0.75 * w    ).floor(),
//                        (pos.y + y * h        + h - 0.5 * th).floor(),
//
//                        // bottom right
////                         20.0, 0, 300.0, (81.0 / 255.0, 162.0 / 255.0, 171.0 / 255.0),
////                         (pos.x + (w / 3.0) - (th / 2.0) + x * 0.75 * w).floor(),
////                         (pos.y + y * h + h).floor(),
//
////                        (pos.x + x * 0.75 * w).floor(),
////                        (pos.y + h + y * h - 10.0).floor(),
//                        w,
//                        th,
//                        "Test");
                }
            }

//            self.canvas.create_image_empty(
//                800,
//                700,
//                femtovg::PixelFormat::Rgba8,
//                femtovg::ImageFlags::FLIP_Y)
//                .expect("making image buffer for hex text");
//
//            for xi in 0..10 {
//                let x = xi as f64;
//
//                for y in 0..10 {
//                    let y =
//                        if xi % 2 == 0 { y as f64 }
//                        else           { y as f64 - 0.5 };
//
//                    p.label(
//                        10.0, 0, (0.5, 1.0, 0.0),
//                        x * 0.75 * w,
//                        0.5 * h + y * h,
//                        w,
//                        20.0,
//                        "test");
//                }
//            }
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
