use crate::hxclr;

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
pub struct Style {
    pub bg_color:       (f32, f32, f32),
    pub border_color:   (f32, f32, f32),
    pub color:          (f32, f32, f32),
    pub border:         f32,
}

impl Style {
    pub fn new() -> Self {
        Self {
            bg_color:       UI_BG_CLR,
            border_color:   UI_BORDER_CLR,
            color:          UI_TXT_CLR,
            border:         UI_BOX_BORD,
        }
    }
}
