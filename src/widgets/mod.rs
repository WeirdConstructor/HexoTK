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
}

impl WidgetType for Button {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        data.with(|data: &mut ButtonData| {
            p.label(20.0, 0, (1.0, 0.0, 1.0), pos.x, pos.y, pos.w, pos.h, &data.label);
        });
    }
    fn size(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData) -> (f64, f64) { (0.0, 0.0) }
    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: UIEvent) {}
}
