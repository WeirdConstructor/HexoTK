// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use crate::constants::*;
use super::*;

#[derive(Debug)]
pub struct Button {
    width:      f64,
    font_size:  f64,
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonMode {
    Toggle,
    ValueDrag
}

#[derive(Debug)]
pub struct ButtonData {
    name:      String,
    mode:      ButtonMode,
    value_buf: [u8; 20],
}

impl ButtonData {
    pub fn new_value_drag(name: &str) -> Box<dyn std::any::Any> {
        Box::new(Self {
            value_buf:  [0; 20],
            mode:       ButtonMode::ValueDrag,
            name:       String::from(name),
        })
    }

    pub fn new_toggle(name: &str) -> Box<dyn std::any::Any> {
        Box::new(Self {
            value_buf:  [0; 20],
            mode:       ButtonMode::Toggle,
            name:       String::from(name),
        })
    }
}


impl Button {
    pub fn new(width: f64, font_size: f64) -> Self {
        Self {
            width,
            font_size,
        }
    }

    fn draw_border(&self, p: &mut dyn Painter, width: f64, clr: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64, fill: bool) {
        let path = &[
            (x,                      y + UI_BTN_BEVEL),
            (x + UI_BTN_BEVEL,       y),
            (x + (w - UI_BTN_BEVEL), y),
            (x + w,                  y + UI_BTN_BEVEL),
            (x + w,                  y + (h - UI_BTN_BEVEL)),
            (x + (w - UI_BTN_BEVEL), y + h),
            (x + UI_BTN_BEVEL,       y + h),
            (x,                      y + (h - UI_BTN_BEVEL)),
        ];

        if fill {
            p.path_fill(clr, &mut path.iter().copied(), true);
        } else {
            p.path_stroke(width, clr, &mut path.iter().copied(), true);
        }
    }

    fn draw_divider(&self, p: &mut dyn Painter, width: f64, color: (f64, f64, f64), x: f64, y: f64) {
        let (x, y) = (
            x + (UI_BTN_BORDER_WIDTH / 2.0).round(),
            y + (UI_BTN_BORDER_WIDTH / 2.0).round(),
        );

        let w = self.width;
        let h = UI_ELEM_TXT_H * 2.0 + UI_BTN_BORDER_WIDTH;

        // divider
        p.path_stroke(
            UI_BTN_BORDER2_WIDTH,
            color,
            &mut [
                (x,     y + (h / 2.0).round()),
                (x + w, y + (h / 2.0).round()),
            ].iter().copied(),
            false);
    }

}

impl WidgetType for Button {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let (x, y) = (pos.x, pos.y);

        let (xo, yo) = (
            x + (UI_BTN_BORDER_WIDTH / 2.0).round(),
            y + (UI_BTN_BORDER_WIDTH / 2.0).round(),
        );

        let w = self.width;
        let h = UI_ELEM_TXT_H * 2.0 + UI_BTN_BORDER_WIDTH;

        let id = data.id();
        let highlight = ui.hl_style_for(id);

        let color =
            match highlight {
                HLStyle::Hover(_) => {
                    self.draw_border(
                        p, UI_BTN_BORDER2_WIDTH, UI_BTN_TXT_HOVER_CLR,
                        xo - (UI_BTN_BORDER2_WIDTH * 0.5).round(),
                        yo - (UI_BTN_BORDER2_WIDTH * 0.5).round(),
                        w + UI_BTN_BORDER2_WIDTH,
                        h + UI_BTN_BORDER2_WIDTH, false);
                    UI_BTN_TXT_HOVER_CLR
                },
                HLStyle::HoverModTarget => {
                    self.draw_border(
                        p, UI_BTN_BORDER_WIDTH, UI_BTN_TXT_HLHOVR_CLR,
                        xo, yo, w, h, false);
                    UI_BTN_TXT_HLHOVR_CLR
                },
                HLStyle::ModTarget => {
                    self.draw_border(
                        p, UI_BTN_BORDER2_WIDTH, UI_BTN_TXT_HLIGHT_CLR,
                        xo, yo, w, h, false);
                    UI_BTN_TXT_HLIGHT_CLR
                },
                HLStyle::Inactive => {
                    self.draw_border(
                        p, UI_BTN_BORDER2_WIDTH, UI_INACTIVE_CLR,
                        xo, yo, w, h, false);
                    self.draw_divider(
                        p, UI_BTN_BORDER2_WIDTH * 1.2, UI_INACTIVE_CLR, x, y);
                    UI_INACTIVE2_CLR
                },
                _ => UI_BTN_TXT_CLR,
            };

        // border
        self.draw_border(
            p, UI_BTN_BORDER_WIDTH, UI_BTN_BORDER_CLR, xo, yo, w, h, false);

        self.draw_border(
            p, UI_BTN_BORDER2_WIDTH, UI_BTN_BORDER2_CLR, xo, yo, w, h, false);

        self.draw_border(
            p, 0.0, UI_BTN_BG_CLR, xo, yo, w, h, true);

        self.draw_divider(p, UI_BTN_BORDER2_WIDTH, UI_BTN_BORDER2_CLR, x, y);

        data.with(|data: &mut ButtonData| {
            let len = ui.params().fmt(id, &mut data.value_buf[..]);
            let val_s = std::str::from_utf8(&data.value_buf[0..len]).unwrap();
            p.label(self.font_size, 0, color,
                xo, yo, w, (h / 2.0).round(), val_s);

            p.label(self.font_size, 0, UI_BTN_TXT_CLR,
                xo,
                yo + UI_ELEM_TXT_H + UI_BTN_BORDER2_WIDTH,
                w, (h / 2.0).round(), &data.name);

            match data.mode {
                ButtonMode::Toggle => {
                    ui.define_active_zone(
                        ActiveZone::new_click_zone(
                            id,
                            Rect::from_tpl((0.0, 0.0, w, h)).offs(xo, yo)));
                },
                ButtonMode::ValueDrag => {
                    ui.define_active_zone(
                        ActiveZone::new_drag_zone(
                            id,
                            Rect::from_tpl((0.0, 0.0, w, h)).offs(xo, yo),
                            true));
                },
            }
        });
    }

    fn size(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, _avail: (f64, f64)) -> (f64, f64) {
        (self.width
         + UI_BTN_BORDER_WIDTH + UI_SAFETY_PAD,
         UI_ELEM_TXT_H + UI_BTN_BORDER_WIDTH + UI_ELEM_TXT_H
         + UI_BTN_BORDER_WIDTH + UI_SAFETY_PAD)
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        match ev {
            UIEvent::Click { id, button, .. } => {
                match button {
                    MButton::Left   => { ui.params_mut().step_next(*id); },
                    MButton::Right  => { ui.params_mut().step_prev(*id); },
                    MButton::Middle => { ui.params_mut().set_default(*id); },
                }
            },
            _ => {},
        }
    }
}
