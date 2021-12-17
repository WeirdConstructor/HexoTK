use crate::hxclr;
use crate::Rect;

use std::rc::Rc;

pub const UI_BOX_H          : f32 = 200.0;
pub const UI_BOX_BORD       : f32 =   3.0;
pub const UI_MARGIN         : f32 =   4.0;
pub const UI_PADDING        : f32 =   6.0;
pub const UI_ELEM_TXT_H     : f32 =  16.0;
pub const UI_SAFETY_PAD     : f32 =   1.0;

pub const UI_BG_CLR               : (f32, f32, f32) = hxclr!(0x414a51); // 473f49
pub const UI_BG2_CLR              : (f32, f32, f32) = hxclr!(0x4b535a); // 594f5d
pub const UI_BG3_CLR              : (f32, f32, f32) = hxclr!(0x545b61); // 645868
pub const UI_TXT_CLR              : (f32, f32, f32) = hxclr!(0xdcdcf0);
pub const UI_BORDER_CLR           : (f32, f32, f32) = hxclr!(0x163239); // 2b0530);
pub const UI_LBL_BG_CLR           : (f32, f32, f32) = hxclr!(0x111920); // hxclr!(0x16232f); // 1a2733); // 200e1f);
pub const UI_LBL_BG_ALT_CLR       : (f32, f32, f32) = hxclr!(0x2d4d5e); // 323237
pub const UI_ACCENT_CLR           : (f32, f32, f32) = hxclr!(0x922f93); // b314aa);
pub const UI_ACCENT_DARK_CLR      : (f32, f32, f32) = hxclr!(0x1e333d); // 4d184a); // 4d184a);
pub const UI_ACCENT_BG1_CLR       : (f32, f32, f32) = hxclr!(0x111920); // UI_LBL_BG_CLR; // hxclr!(0x111920); // UI_LBL_BG_CLR; // hxclr!(0x27091b); // 381c38); // 200e1f);
pub const UI_ACCENT_BG2_CLR       : (f32, f32, f32) = hxclr!(0x192129); // 2c132a);
pub const UI_PRIM_CLR             : (f32, f32, f32) = hxclr!(0x03fdcb); // 69e8ed
pub const UI_PRIM2_CLR            : (f32, f32, f32) = hxclr!(0x228f9d); // 1aaeb3
pub const UI_HLIGHT_CLR           : (f32, f32, f32) = hxclr!(0xecf9ce); // e9f840
pub const UI_HLIGHT2_CLR          : (f32, f32, f32) = hxclr!(0xbcf9cd); // b5c412
pub const UI_SELECT_CLR           : (f32, f32, f32) = hxclr!(0xd73988); // 0xdc1821);
pub const UI_INACTIVE_CLR         : (f32, f32, f32) = hxclr!(0x6f8782);
pub const UI_INACTIVE2_CLR        : (f32, f32, f32) = hxclr!(0xa6dbd0);

#[derive(Debug, Clone)]
pub enum Align {
    Center,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub enum VAlign {
    Middle,
    Top,
    Bottom,
}

#[derive(Debug, Clone)]
pub struct Style {
    pub bg_color:               (f32, f32, f32),
    pub border_color:           (f32, f32, f32),
    pub color:                  (f32, f32, f32),
    pub border:                 f32,
    pub pad_left:               f32,
    pub pad_right:              f32,
    pub pad_top:                f32,
    pub pad_bottom:             f32,
    pub shadow_offs:            (f32, f32),
    pub shadow_color:           (f32, f32, f32),
    pub hover_shadow_color:     (f32, f32, f32),
    pub hover_border_color:     (f32, f32, f32),
    pub hover_color:            (f32, f32, f32),
    pub active_shadow_color:    (f32, f32, f32),
    pub active_border_color:    (f32, f32, f32),
    pub active_color:           (f32, f32, f32),
    pub text_align:             Align,
    pub text_valign:            VAlign,
    pub font_size:              f32,
    pub colors:                 Vec<(f32, f32, f32)>,
}

impl Style {
    pub fn new() -> Self {
        let colors = vec![
            hxclr!(0x922f93), // 0
            hxclr!(0x862b37),
            hxclr!(0xb45745),
            hxclr!(0x835933),
            hxclr!(0xa69b64),
            hxclr!(0xbec8a6),
            hxclr!(0x346c38), // 6
            hxclr!(0x1fb349),
            hxclr!(0x4cdb80),
            hxclr!(0x59bca3),
            hxclr!(0x228f9d),
            hxclr!(0x03b5e7),
            hxclr!(0x3b5eca), // 12
            hxclr!(0x594fa1),
            hxclr!(0xc2b2eb),
            hxclr!(0xac70fa),
            hxclr!(0x9850a9),
            hxclr!(0xdc4fc1), // 17
            hxclr!(0x03fdcb), // 18
        ];

        Self {
            bg_color:               UI_BG_CLR,
            border_color:           UI_BORDER_CLR,
            color:                  UI_PRIM_CLR,
            border:                 UI_BOX_BORD,
            pad_left:               0.0,
            pad_right:              0.0,
            pad_top:                0.0,
            pad_bottom:             0.0,
            shadow_offs:            (0.0, 0.0),
            shadow_color:           UI_PRIM_CLR,
            hover_shadow_color:     UI_SELECT_CLR,
            hover_border_color:     UI_HLIGHT_CLR,
            hover_color:            UI_HLIGHT_CLR,
            active_shadow_color:    UI_HLIGHT_CLR,
            active_border_color:    UI_SELECT_CLR,
            active_color:           UI_HLIGHT_CLR,
            text_align:             Align::Center,
            text_valign:            VAlign::Middle,
            font_size:              12.0,
            colors,
        }
    }

    pub fn apply_padding(&self, pos: Rect) -> Rect {
        Rect {
            x: pos.x + self.pad_left,
            y: pos.y + self.pad_top,
            w: pos.w - (self.pad_left + self.pad_right),
            h: pos.h - (self.pad_top  + self.pad_bottom),
        }
    }

    pub fn with_style_clone<F: FnOnce(&mut Style)>(&self, f: F) -> Rc<Self> {
        let mut clone = self.clone();
        f(&mut clone);
        Rc::new(clone)
    }

    pub fn color_by_idx(&self, idx: usize) -> (f32, f32, f32) {
        self.colors[idx % self.colors.len()]
    }
}
