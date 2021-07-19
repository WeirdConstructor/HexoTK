// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

use crate::constants::*;
use super::*;

#[derive(Debug)]
pub struct Knob {
    sbottom:        (f64, f64),
    s:              [(f64, f64); 7],
    arc_len:        [f64; 5],
    full_len:       f64,
    s1_len:         f64,
    s2_len:         f64,
    radius:         f64,
    font_size_lbl:  f64,
    font_size_data: f64,
}

impl Knob {
    pub fn new(radius: f64, font_size_lbl: f64, font_size_data: f64) -> Self {
        let init_rot : f64 = 90.;

        let mut s       = [(0.0_f64, 0.0_f64); 7];
        let mut arc_len = [0.0_f64; 5];

        let sbottom = circle_point(radius, init_rot.to_radians());

        s[0] = circle_point(radius, (init_rot + 10.0_f64).to_radians());
        s[1] = circle_point(radius, (init_rot + 60.0_f64).to_radians());
        s[2] = circle_point(radius, (init_rot + 120.0_f64).to_radians());
        s[3] = circle_point(radius, (init_rot + 180.0_f64).to_radians());
        s[4] = circle_point(radius, (init_rot + 240.0_f64).to_radians());
        s[5] = circle_point(radius, (init_rot + 300.0_f64).to_radians());
        s[6] = circle_point(radius, (init_rot + 350.0_f64).to_radians());

        let s1_len  = ((s[0].0 - s[1].1).powf(2.0) + (s[0].0 - s[1].1).powf(2.0)).sqrt();
        let s2_len  = ((s[1].0 - s[2].1).powf(2.0) + (s[1].0 - s[2].1).powf(2.0)).sqrt();

        // TODO: If I stumble across this the next time, simplify this.
        let full_len = s2_len * 2.0 + s2_len * 4.0;

        arc_len[0] = s2_len                  / full_len;
        arc_len[1] = (s2_len + s2_len)       / full_len;
        arc_len[2] = (s2_len + 2.0 * s2_len) / full_len;
        arc_len[3] = (s2_len + 3.0 * s2_len) / full_len;
        arc_len[4] = (s2_len + 4.0 * s2_len) / full_len;

        Self {
            sbottom,
            s,
            arc_len,
            full_len,
            s1_len,
            s2_len,
            radius,
            font_size_lbl,
            font_size_data,
        }
    }

    pub fn get_center_offset(&self, line_width: f64) -> (f64, f64) {
        ((self.get_label_rect().2 / 2.0).ceil() + UI_SAFETY_PAD,
         self.radius + (line_width / 2.0).ceil() + UI_SAFETY_PAD)
    }

    pub fn get_fine_adjustment_mark(&self) -> (f64, f64, f64, f64) {
        let mut r = self.get_fine_adjustment_rect();
        r.1 = (r.1 - UI_ELEM_TXT_H * 0.5).round();
        r.3 = (r.3 + UI_ELEM_TXT_H * 0.5).round();

        let mut size = (self.font_size_lbl * 0.25).round();
        if (size as i32) % 2 != 0 {
            size += 1.0;
        }
        ((r.0 + size * 1.0).round(),
         r.1 + (r.3 * 0.5 + size * 0.5).round(),
         size,
         size)
    }

    pub fn get_fine_adjustment_rect(&self) -> (f64, f64, f64, f64) {
        self.get_label_rect()
    }

    pub fn get_coarse_adjustment_rect(&self) -> (f64, f64, f64, f64) {
        let width = self.radius * 2.0;
        ((self.sbottom.0 - self.radius).round(),
         -self.radius,
         width.round(),
         (self.radius * 2.0).round())
    }

    pub fn get_value_rect(&self, double: bool) -> (f64, f64, f64, f64) {
        let width = self.radius * 2.0;
        if double {
            ((self.sbottom.0 - self.radius).round(),
             (self.sbottom.1 - (self.radius + UI_ELEM_TXT_H)).round(),
             width.round(),
             2.0 * UI_ELEM_TXT_H)
        } else {
            ((self.sbottom.0 - self.radius).round(),
             (self.sbottom.1 - (self.radius + UI_ELEM_TXT_H * 0.5)).round(),
             width.round(),
             UI_ELEM_TXT_H)
        }
    }

    pub fn get_label_rect(&self) -> (f64, f64, f64, f64) {
        let width = self.radius * 2.25;
        ((self.sbottom.0 - width * 0.5).round(),
         (self.sbottom.1 + 0.5 * UI_BG_KNOB_STROKE).round(),
         width.round(),
         UI_ELEM_TXT_H)
    }

    pub fn get_decor_rect1(&self) -> (f64, f64, f64, f64) {
        ((self.s[0].0      - 0.25 * UI_BG_KNOB_STROKE).round(),
         (self.sbottom.1    - 0.5 * UI_BG_KNOB_STROKE).round(),
         ((self.s[6].0 - self.s[0].0).abs()
                           + 0.5 * UI_BG_KNOB_STROKE).round(),
         UI_BG_KNOB_STROKE * 1.0)
    }

    pub fn draw_name(&self, p: &mut dyn Painter, x: f64, y: f64, s: &str) {
        let r = self.get_label_rect();
        p.label(
            self.font_size_lbl, 0, UI_TXT_KNOB_CLR,
            x + r.0, y + r.1, r.2, r.3, s, DBGID_KNOB_NAME);
    }

    pub fn draw_value_label(&self, double: bool, first: bool, p: &mut dyn Painter, x: f64, y: f64, highlight: HLStyle, s: &str) {
        let r = self.get_value_rect(double);

        let r =
            if double {
                if first {
                    (r.0, r.1 + 1.0, r.2, UI_ELEM_TXT_H)
                } else {
                    (r.0, r.1 + UI_ELEM_TXT_H - 1.0, r.2, UI_ELEM_TXT_H)
                }
            } else {
                r
            };

        let color =
            match highlight {
                HLStyle::Hover(_subtype) => { UI_TXT_KNOB_HOVER_CLR },
                HLStyle::Inactive        => { UI_INACTIVE_CLR },
                HLStyle::EditModAmt      => { UI_TXT_KNOB_MOD_CLR },
                _                        => { UI_TXT_KNOB_CLR },
            };

        let some_right_padding = 6.0;
        let light_font_offs    = 4.0;

        p.label(
            self.font_size_data, 0, color,
            x + r.0 + light_font_offs,
            y + r.1,
            r.2 - some_right_padding,
            r.3, s,
            if double {
                if first { DBGID_KNOB_VALUE } else { DBGID_KNOB_MODAMT }
            } else { DBGID_KNOB_VALUE });
    }

    pub fn draw_mod_arc(
        &self, p: &mut dyn Painter, xo: f64, yo: f64,
        value: f64, modamt: Option<f64>,
        fg_clr: (f64, f64, f64))
    {
        if let Some(modamt) = modamt {
            if modamt > 0.0 {
                self.draw_oct_arc(
                    p, xo, yo,
                    UI_MG_KNOB_STROKE,
                    UI_FG_KNOB_MODPOS_CLR,
                    None,
                    (value + modamt).clamp(0.0, 1.0));
                self.draw_oct_arc(
                    p, xo, yo,
                    UI_MG_KNOB_STROKE,
                    fg_clr,
                    Some(UI_FG_KNOB_MODPOS_CLR),
                    value);
            } else {
                self.draw_oct_arc(
                    p, xo, yo,
                    UI_MG_KNOB_STROKE,
                    UI_FG_KNOB_MODNEG_CLR,
                    Some(UI_FG_KNOB_MODNEG_CLR),
                    value);
                self.draw_oct_arc(
                    p, xo, yo,
                    UI_MG_KNOB_STROKE,
                    fg_clr,
                    None,
                    (value + modamt).clamp(0.0, 1.0));
            }
        } else {
            self.draw_oct_arc(
                p, xo, yo,
                UI_MG_KNOB_STROKE,
                fg_clr,
                Some(fg_clr),
                value);
        }
    }

    pub fn draw_oct_arc(&self, p: &mut dyn Painter, x: f64, y: f64, line_w: f64, color: (f64, f64, f64), dot_color: Option<(f64, f64, f64)>, value: f64) {
        let arc_len = &self.arc_len;

        let (next_idx, prev_arc_len) =
            if value > arc_len[4] {
                (6, arc_len[4])
            } else if value > arc_len[3] {
                (5, arc_len[3])
            } else if value > arc_len[2] {
                (4, arc_len[2])
            } else if value > arc_len[1] {
                (3, arc_len[1])
            } else if value > arc_len[0] {
                (2, arc_len[0])
            } else {
                (1, 0.0)
            };

        let mut s : [(f64, f64); 7] = self.s;
        for p in s.iter_mut() {
            p.0 += x;
            p.1 += y;
        }

        // The segment len is used to calculate the ratio of the traveled
        // total length.
        let segment_len = self.s2_len;
        let prev       = s[next_idx - 1];
        let last       = s[next_idx];
        let rest_len   = value - prev_arc_len;
        let rest_ratio = rest_len / (segment_len / self.full_len);
//        println!("i[{}] prev_arc_len={:1.3}, rest_len={:1.3}, value={:1.3}, seglen={:1.3}",
//                 next_idx, prev_arc_len, rest_len, value,
//                 segment_len / self.full_len);
        let partial =
            ((last.0 - prev.0) * rest_ratio,
             (last.1 - prev.1) * rest_ratio);

        s[next_idx] = (
            prev.0 + partial.0,
            prev.1 + partial.1
        );

        if let Some(clr) = dot_color {
            p.arc_stroke(
                0.9 * line_w * 0.5,
                clr,
                0.9 * line_w * 1.5,
                0.0, 2.0 * std::f64::consts::PI,
                prev.0 + partial.0,
                prev.1 + partial.1);
        }

        p.path_stroke(line_w, color, &mut s.iter().copied().take(next_idx + 1), false);
    }
}

fn circle_point(r: f64, angle: f64) -> (f64, f64) {
    let (y, x) = angle.sin_cos();
    (x * r, y * r)
}

#[derive(Debug, Clone)]
pub struct KnobData {
    lbl_buf:    [u8; 15],
    name:       String,
}

impl KnobData {
	pub fn new(name: &str) -> Box<dyn std::any::Any> {
        Box::new(Self {
            lbl_buf:    [0; 15],
            name:       name.to_string(),
        })
    }
}

impl WidgetType for Knob {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
		let (x, y) = (pos.x, pos.y);

        let (knob_xo, knob_yo) =
            self.get_center_offset(UI_BG_KNOB_STROKE);
        // let (knob_w, knob_h) = self.size(ui, data, (pos.w, pos.h));
        let (xo, yo) = (x + knob_xo, y + knob_yo);

        let id     = data.id();
        let modamt = ui.atoms().get_ui_mod_amt(id).map(|v| v as f64);

        self.draw_oct_arc(
            p, xo, yo,
            UI_BG_KNOB_STROKE,
            UI_BG_KNOB_STROKE_CLR,
            None,
            1.0);

        let dc1 = self.get_decor_rect1();
        p.rect_fill(
            UI_BG_KNOB_STROKE_CLR,
            xo + dc1.0, yo + dc1.1, dc1.2, dc1.3);

        let valrect = self.get_value_rect(modamt.is_some());
        p.rect_fill(
            UI_BG_KNOB_STROKE_CLR,
            valrect.0 + xo, valrect.1 + yo, valrect.2, valrect.3);

        let lblrect = self.get_label_rect();
        p.rect_fill(
            UI_BG_KNOB_STROKE_CLR,
            lblrect.0 + xo, lblrect.1 + yo, lblrect.2, lblrect.3);

        let r = self.get_fine_adjustment_mark();
        p.rect_fill(
            UI_BG_KNOB_STROKE_CLR,
            xo + r.0, yo + r.1, r.2, r.3);

        let highlight = ui.hl_style_for(id, None);
        let value =
            if let Some(v) = ui.atoms().get_ui_range(id) {
                (v as f64).clamp(0.0, 1.0)
            } else { 0.0 };

        let mut hover_fine_adj = false;

        // TODO MOD AMOUNT:
        // double click enables mod mode, which highlights the outer
        // ring of the buttons differently.
        // Dragging on the fine/coarse zones then goes into ValueDragMod
        // mode, which sets the mod amount.
        // after dragging the mod mode is resetted.
        // Right click after double click removes the modulation!
        //
        // We get the displayed mod value from ui.atoms().mod_amount(id)
        // (modamt + value) is then the position. if modamt < 0.0 we
        // draw different ring colors in layers.
        match highlight {
            HLStyle::EditModAmt => {
                self.draw_oct_arc(
                    p, xo, yo,
                    UI_MG_KNOB_STROKE,
                    UI_MG_KNOB_STROKE_CLR,
                    None,
                    1.0);

                self.draw_mod_arc(
                    p, xo, yo, value, modamt,
                    UI_FG_KNOB_STROKE_CLR);
            },
//            HLStyle::HoverModTarget => {
//                self.draw_oct_arc(
//                    p, xo, yo,
//                    UI_MG_KNOB_STROKE * 2.0,
//                    UI_TXT_KNOB_MODPOS_CLR,
//                    false,
//                    1.0);
//            },
            HLStyle::Hover(subtype) => {
                if let ZoneType::ValueDragFine = subtype {
                    hover_fine_adj = true;

                    let r = self.get_fine_adjustment_mark();
                    p.rect_fill(
                        UI_TXT_KNOB_HOVER_CLR,
                        xo + r.0, yo + r.1, r.2, r.3);
                }

                self.draw_oct_arc(
                    p, xo, yo,
                    UI_MG_KNOB_STROKE,
                    UI_MG_KNOB_STROKE_CLR,
                    None,
                    1.0);

                self.draw_mod_arc(
                    p, xo, yo, value, modamt,
                    UI_FG_KNOB_STROKE_CLR);
            },
            HLStyle::Inactive => {
                self.draw_oct_arc(
                    p, xo, yo,
                    UI_MG_KNOB_STROKE,
                    UI_INACTIVE_CLR,
                    None,
                    1.0);

                self.draw_mod_arc(
                    p, xo, yo, value, modamt,
                    UI_INACTIVE2_CLR);
            },
              HLStyle::None
            | HLStyle::AtomClick
            => {
                self.draw_oct_arc(
                    p, xo, yo,
                    UI_MG_KNOB_STROKE,
                    UI_MG_KNOB_STROKE_CLR,
                    None,
                    1.0);

                self.draw_mod_arc(
                    p, xo, yo, value, modamt,
                    UI_FG_KNOB_STROKE_CLR);
            }
        }

        data.with(|data: &mut KnobData| {
            let len = ui.atoms().fmt(id, &mut data.lbl_buf[..]);
            let val_s = std::str::from_utf8(&data.lbl_buf[0..len]).unwrap();
            self.draw_value_label(modamt.is_some(), true, p, xo, yo, highlight, val_s);


            if let Some(_) = modamt {
                let len = ui.atoms().fmt_mod(id, &mut data.lbl_buf[..]);
                let val_s = std::str::from_utf8(&data.lbl_buf[0..len]).unwrap();
                self.draw_value_label(true, false, p, xo, yo, highlight, val_s);
            }

            if hover_fine_adj {
                let len = ui.atoms().fmt_norm(id, &mut data.lbl_buf[..]);
                let val_s = std::str::from_utf8(&data.lbl_buf[0..len]).unwrap();
                // + 2.0 for the marker cube, to space it from the minus sign.
                self.draw_name(p, xo + 2.0, yo, &val_s);
            } else {
                self.draw_name(p, xo, yo, &data.name);
            }
        });

        ui.define_active_zone(
            ActiveZone::new_drag_zone(
                id,
                Rect::from_tpl(self.get_coarse_adjustment_rect()).offs(xo, yo), true)
            .dbgid(DBGID_KNOB_COARSE));
        ui.define_active_zone(
            ActiveZone::new_drag_zone(
                id,
                Rect::from_tpl(self.get_fine_adjustment_rect()).offs(xo, yo), false)
            .dbgid(DBGID_KNOB_FINE));
    }

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _avail: (f64, f64)) -> (f64, f64) {
        let (_lbl_x, lbl_y, lbl_w, lbl_h) = self.get_label_rect();

        (lbl_w + 2.0 * UI_SAFETY_PAD,
         (self.radius + lbl_y + lbl_h + 0.5 * UI_BG_KNOB_STROKE).round()
         + UI_SAFETY_PAD)
    }

    fn event(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _ev: &UIEvent) {
    }
}
