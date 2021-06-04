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
    ValueDrag,
    SignalSettingInc,
    SignalSettingClick,
    SignalSettingToggle,
    SignalParamClick,
    SignalParamToggle,
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

    pub fn new_setting_inc(name: &str) -> Box<dyn std::any::Any> {
        Box::new(Self {
            value_buf:  [0; 20],
            mode:       ButtonMode::SignalSettingInc,
            name:       String::from(name),
        })
    }

    pub fn new_setting_click(name: &str) -> Box<dyn std::any::Any> {
        Box::new(Self {
            value_buf:  [0; 20],
            mode:       ButtonMode::SignalSettingClick,
            name:       String::from(name),
        })
    }

    pub fn new_setting_toggle(name: &str) -> Box<dyn std::any::Any> {
        Box::new(Self {
            value_buf:  [0; 20],
            mode:       ButtonMode::SignalSettingToggle,
            name:       String::from(name),
        })
    }

    pub fn new_param_click(name: &str) -> Box<dyn std::any::Any> {
        Box::new(Self {
            value_buf:  [0; 20],
            mode:       ButtonMode::SignalParamClick,
            name:       String::from(name),
        })
    }

    pub fn new_param_toggle(name: &str) -> Box<dyn std::any::Any> {
        Box::new(Self {
            value_buf:  [0; 20],
            mode:       ButtonMode::SignalParamToggle,
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

    fn draw_divider(&self, p: &mut dyn Painter, _width: f64, color: (f64, f64, f64), x: f64, y: f64) {
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
        let highlight = ui.hl_style_for(id, None);

        let (view_value, value_set) =
            data.with(|data: &mut ButtonData| {
                match data.mode {
                    | ButtonMode::SignalSettingToggle
                    | ButtonMode::SignalParamToggle => {
                        let val_set =
                            if let Some(at) = ui.atoms().get(id) {
                                at.f() > 0.5
                            } else { false };

                        (false, val_set)
                    },
                      ButtonMode::SignalSettingInc
                    | ButtonMode::SignalSettingClick
                    | ButtonMode::SignalParamClick => {
                        (false, false)
                    },
                    _ => (true, false),
                }
            }).unwrap_or((true, false));

        let mut label_color = UI_BTN_TXT_CLR;

        let (mut color, border_color, mut bg_color) =
            match highlight {
                HLStyle::Hover(_) => {
                    (UI_BTN_TXT_HOVER_CLR, UI_BTN_TXT_HOVER_CLR, UI_BTN_BG_CLR)
                },
                HLStyle::HoverModTarget => {
                    (UI_BTN_TXT_HLHOVR_CLR, UI_BTN_TXT_HLHOVR_CLR, UI_BTN_BG_CLR)
                },
                HLStyle::AtomClick => {
                    label_color = UI_BTN_BG_CLR;
                    (UI_BTN_BG_CLR, UI_BTN_BORDER2_CLR, UI_BTN_TXT_CLR)
                },
                HLStyle::ModTarget => {
                    (UI_BTN_TXT_HLIGHT_CLR, UI_BTN_TXT_HLIGHT_CLR, UI_BTN_BG_CLR)
                },
                HLStyle::Inactive => {
                    (UI_INACTIVE2_CLR, UI_INACTIVE2_CLR, UI_BTN_BG_CLR)
                },
                _ => (UI_BTN_TXT_CLR, UI_BTN_BORDER2_CLR, UI_BTN_BG_CLR)
            };

        if value_set {
            bg_color    = UI_BTN_TXT_CLR;
            color       = UI_BTN_BG_CLR;
            label_color = UI_BTN_BG_CLR;
        }

        // border
        self.draw_border(
            p, UI_BTN_BORDER_WIDTH, UI_BTN_BORDER_CLR, xo, yo, w, h, false);

        self.draw_border(
            p, UI_BTN_BORDER2_WIDTH, border_color, xo, yo, w, h, false);

        self.draw_border(
            p, 0.0, bg_color, xo, yo, w, h, true);

        if view_value {
            self.draw_divider(p, UI_BTN_BORDER2_WIDTH, UI_BTN_BORDER2_CLR, x, y);
        }

        data.with(|data: &mut ButtonData| {
            if view_value {
                let len = ui.atoms().fmt(id, &mut data.value_buf[..]);
                let val_s = std::str::from_utf8(&data.value_buf[0..len]).unwrap();
                p.label(self.font_size, 0, color,
                    xo, yo, w, (h / 2.0).round(), val_s);

                p.label(self.font_size, 0, label_color,
                    xo,
                    yo + UI_ELEM_TXT_H + UI_BTN_BORDER2_WIDTH,
                    w, (h / 2.0).round(), &data.name);

            } else {
                p.label(self.font_size, 0, label_color,
                    xo,
                    (yo
                     + 0.5 * UI_ELEM_TXT_H
                     + UI_BTN_BORDER2_WIDTH)
                    .round(),
                    w, UI_ELEM_TXT_H, &data.name);
            }

            let zone_rect = Rect::from_tpl((0.0, 0.0, w, h)).offs(xo, yo);

            match data.mode {
                ButtonMode::SignalSettingInc => {
                    ui.define_active_zone(
                        ActiveZone::new_atom_inc(id, zone_rect, false));
                },
                ButtonMode::SignalSettingToggle => {
                    ui.define_active_zone(
                        ActiveZone::new_atom_toggle(id, zone_rect, true, false));
                },
                ButtonMode::SignalSettingClick => {
                    ui.define_active_zone(
                        ActiveZone::new_atom_toggle(id, zone_rect, true, true));
                },
                ButtonMode::SignalParamToggle => {
                    ui.define_active_zone(
                        ActiveZone::new_atom_toggle(id, zone_rect, false, false));
                },
                ButtonMode::SignalParamClick => {
                    ui.define_active_zone(
                        ActiveZone::new_atom_toggle(id, zone_rect, false, true));
                },
                ButtonMode::Toggle => {
                    ui.define_active_zone(
                        ActiveZone::new_click_zone(id, zone_rect));
                },
                ButtonMode::ValueDrag => {
                    ui.define_active_zone(
                        ActiveZone::new_drag_zone(id, zone_rect, true));
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
                println!("BTN CHK: {} == {}", *id, data.id());
                if *id == data.id() {
                    data.with(|data: &mut ButtonData| {
                        match button {
                            MButton::Left   => {
                                match data.mode {
                                    ButtonMode::Toggle => {
                                        ui.atoms_mut().step_next(*id);
                                    },
                                    _ => {},
                                }
                            },
                            MButton::Right  => {
                                match data.mode {
                                    ButtonMode::Toggle => {
                                        ui.atoms_mut().step_prev(*id);
                                    },
                                    _ => {},
                                }
                            },
                            MButton::Middle => { ui.atoms_mut().set_default(*id); },
                        }
                    });

                    ui.queue_redraw();
                }
            },
            _ => {},
        }
    }
}
