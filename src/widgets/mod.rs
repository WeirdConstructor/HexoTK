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
