// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use crate::constants::*;
use super::*;

#[derive(Debug)]
pub struct Text {
    font_size:  f64,
}

#[derive(Debug)]
pub struct TextData {
    title:     String,
    lines:     Vec<String>,
}

fn split_text(text: &str) -> (String, Vec<String>) {
    let mut title = None;
    let mut lines = vec![];

    for line in text.split("\n") {
        if title.is_none() {
            title = Some(line.to_string());
            continue;
        }

        lines.push(line.to_string());
    }

    (title.unwrap_or_else(|| "".to_string()), lines)
}

impl TextData {
    pub fn new(text: &str) -> Box<dyn std::any::Any> {
        let (title, lines) = split_text(text);

        Box::new(Self { title, lines })
    }

    pub fn set(&mut self, text: &str) {
        let (title, lines) = split_text(text);

        self.title = title;
        self.lines = lines;
    }
}


impl Text {
    pub fn new(font_size: f64) -> Self {
        Self {
            font_size,
        }
    }
}

impl WidgetType for Text {
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

        data.with(|data: &mut TextData| {
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

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _avail: (f64, f64)) -> (f64, f64) {
        (self.width
         + UI_BTN_BORDER_WIDTH + UI_SAFETY_PAD,
         UI_ELEM_TXT_H + UI_BTN_BORDER_WIDTH + UI_ELEM_TXT_H
         + UI_BTN_BORDER_WIDTH + UI_SAFETY_PAD)
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        match ev {
            UIEvent::Click { id, button, .. } => {
                if *id == data.id() {
                    match button {
                        MButton::Left   => { ui.params_mut().step_next(*id); },
                        MButton::Right  => { ui.params_mut().step_prev(*id); },
                        MButton::Middle => { ui.params_mut().set_default(*id); },
                    }

                    ui.queue_redraw();
                }
            },
            _ => {},
        }
    }
}
