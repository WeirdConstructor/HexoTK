// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use super::*;

#[derive(Debug)]
struct Container {
    w: f64,
}

impl WidgetType for Container {
    fn draw(&self, ui: &dyn WidgetUI, data: &mut WidgetData, pos: Rect) {}
    fn size(&self, ui: &dyn WidgetUI, data: &mut WidgetData) {}
    fn event(&self, ui: &dyn WidgetUI, data: &mut WidgetData, ev: UIEvent) {}
}
