// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoTK. See README.md and COPYING for details.

pub fn lighten_clr(depth: u32, clr: (f64, f64, f64)) -> (f64, f64, f64) {
    (clr.0 * (1.2_f64).powf(depth as f64),
     clr.1 * (1.2_f64).powf(depth as f64),
     clr.2 * (1.2_f64).powf(depth as f64))
}

macro_rules! hxclr {
    ($i: expr) => {
        (
            ($i >> 16 & 0xFF) as f64 / 255.0,
            ($i >> 8  & 0xFF) as f64 / 255.0,
            ($i       & 0xFF) as f64 / 255.0,
        )
    }
}

pub const UI_BG_CLR               : (f64, f64, f64) = hxclr!(0x414a51); // 473f49
pub const UI_BG2_CLR              : (f64, f64, f64) = hxclr!(0x4b535a); // 594f5d
pub const UI_BG3_CLR              : (f64, f64, f64) = hxclr!(0x545b61); // 645868
pub const UI_TXT_CLR              : (f64, f64, f64) = hxclr!(0xdcdcf0);
pub const UI_BORDER_CLR           : (f64, f64, f64) = hxclr!(0x163239); // 2b0530);
pub const UI_LBL_BG_CLR           : (f64, f64, f64) = hxclr!(0x111920); // hxclr!(0x16232f); // 1a2733); // 200e1f);
pub const UI_LBL_BG_ALT_CLR       : (f64, f64, f64) = hxclr!(0x2d4d5e); // 323237
pub const UI_ACCENT_CLR           : (f64, f64, f64) = hxclr!(0x922f93); // b314aa);
pub const UI_ACCENT_DARK_CLR      : (f64, f64, f64) = hxclr!(0x1e333d); // 4d184a); // 4d184a);
pub const UI_ACCENT_BG1_CLR       : (f64, f64, f64) = UI_LBL_BG_CLR; // hxclr!(0x111920); // UI_LBL_BG_CLR; // hxclr!(0x27091b); // 381c38); // 200e1f);
pub const UI_ACCENT_BG2_CLR       : (f64, f64, f64) = hxclr!(0x192129); // 2c132a);
pub const UI_PRIM_CLR             : (f64, f64, f64) = hxclr!(0x03fdcb); // 69e8ed
pub const UI_PRIM2_CLR            : (f64, f64, f64) = hxclr!(0x228f9d); // 1aaeb3
pub const UI_HLIGHT_CLR           : (f64, f64, f64) = hxclr!(0xecf9ce); // e9f840
pub const UI_HLIGHT2_CLR          : (f64, f64, f64) = hxclr!(0xbcf9cd); // b5c412
pub const UI_SELECT_CLR           : (f64, f64, f64) = hxclr!(0xd73988); // 0xdc1821);
pub const UI_INACTIVE_CLR         : (f64, f64, f64) = hxclr!(0x6f8782);
pub const UI_INACTIVE2_CLR        : (f64, f64, f64) = hxclr!(0xa6dbd0);

pub const UI_VERSION_FONT_SIZE    : f64 = 10.0;

pub const UI_HELP_FONT_SIZE       : f64 = 16.0;
pub const UI_HELP_TXT_CLR         : (f64, f64, f64) = UI_TXT_CLR;

pub const UI_LBL_TXT_CLR          : (f64, f64, f64) = UI_TXT_CLR;

pub const UI_CONT_FONT_SIZE       : f64 = 14.0;
pub const UI_CONT_FONT_CLR        : (f64, f64, f64) = UI_PRIM_CLR;

pub const UI_BG_KNOB_STROKE       : f64 = 8.0;
pub const UI_MG_KNOB_STROKE       : f64 = 3.0;
pub const UI_FG_KNOB_STROKE       : f64 = 5.0;
pub const UI_BG_KNOB_STROKE_CLR   : (f64, f64, f64) = UI_LBL_BG_CLR;
pub const UI_MG_KNOB_STROKE_CLR   : (f64, f64, f64) = UI_ACCENT_CLR;
pub const UI_FG_KNOB_STROKE_CLR   : (f64, f64, f64) = UI_PRIM_CLR;
pub const UI_TXT_KNOB_CLR         : (f64, f64, f64) = UI_PRIM_CLR;
pub const UI_TXT_KNOB_HOVER_CLR   : (f64, f64, f64) = UI_HLIGHT_CLR;
pub const UI_TXT_KNOB_HLIGHT_CLR  : (f64, f64, f64) = UI_HLIGHT_CLR;
pub const UI_TXT_KNOB_MODPOS_CLR  : (f64, f64, f64) = UI_HLIGHT2_CLR;
pub const UI_TXT_KNOB_MODNEG_CLR  : (f64, f64, f64) = UI_INACTIVE2_CLR;
pub const UI_GUI_BG_CLR           : (f64, f64, f64) = UI_BG_CLR;
pub const UI_GUI_CLEAR_CLR        : (f64, f64, f64) = UI_LBL_BG_CLR;
pub const UI_BORDER_WIDTH         : f64 = 2.0;
pub const UI_KNOB_RADIUS          : f64 = 25.0;
pub const UI_KNOB_SMALL_RADIUS    : f64 = 14.0;
pub const UI_KNOB_FONT_SIZE       : f64 = 11.0;

pub const UI_BTN_BORDER_CLR       : (f64, f64, f64) = UI_BG_KNOB_STROKE_CLR;
pub const UI_BTN_BORDER2_CLR      : (f64, f64, f64) = UI_ACCENT_CLR;
pub const UI_BTN_BG_CLR           : (f64, f64, f64) = UI_BG_KNOB_STROKE_CLR;
pub const UI_BTN_TXT_CLR          : (f64, f64, f64) = UI_TXT_KNOB_CLR;
pub const UI_BTN_TXT_HOVER_CLR    : (f64, f64, f64) = UI_TXT_KNOB_HOVER_CLR;
pub const UI_BTN_TXT_HLIGHT_CLR   : (f64, f64, f64) = UI_TXT_KNOB_HLIGHT_CLR;
pub const UI_BTN_WIDTH            : f64 = 3.0 * UI_KNOB_RADIUS;
pub const UI_BTN_BORDER_WIDTH     : f64 = 6.0;
pub const UI_BTN_BORDER2_WIDTH    : f64 = 2.0;
pub const UI_BTN_BEVEL            : f64 = UI_ELEM_TXT_H / 4.0;

pub const UI_GRPH_BORDER          : f64 = 2.0;
pub const UI_GRPH_BORDER_CLR      : (f64, f64, f64) = UI_ACCENT_CLR;
pub const UI_GRPH_BORDER_HOVER_CLR: (f64, f64, f64) = UI_HLIGHT_CLR;
pub const UI_GRPH_TEXT_CLR        : (f64, f64, f64) = UI_TXT_KNOB_CLR;
pub const UI_GRPH_LINE_CLR        : (f64, f64, f64) = UI_PRIM_CLR;
pub const UI_GRPH_PHASE_CLR       : (f64, f64, f64) = UI_HLIGHT_CLR;
pub const UI_GRPH_PHASE_BG_CLR    : (f64, f64, f64) = UI_HLIGHT2_CLR;
pub const UI_GRPH_BG              : (f64, f64, f64) = UI_LBL_BG_CLR;
pub const UI_GRPH_FONT_SIZE       : f64 = UI_KNOB_FONT_SIZE;

pub const UI_TAB_WIDTH            : f64 = 90.0;
pub const UI_TAB_HEIGHT           : f64 = 26.0;
pub const UI_TAB_FONT_SIZE        : f64 = 12.0;
pub const UI_TAB_BG_CLR           : (f64, f64, f64) = UI_LBL_BG_CLR;
pub const UI_TAB_BORDER_WIDTH     : f64 = 1.0;
pub const UI_TAB_BORDER_CLR       : (f64, f64, f64) = UI_BORDER_CLR;
pub const UI_TAB_PAD_WIDTH        : f64 = 2.0;
pub const UI_TAB_DIV_CLR          : (f64, f64, f64) = UI_PRIM_CLR;
pub const UI_TAB_TXT_CLR          : (f64, f64, f64) = UI_PRIM_CLR;
pub const UI_TAB_TXT2_CLR         : (f64, f64, f64) = UI_PRIM2_CLR;
pub const UI_TAB_TXT_HOVER_CLR    : (f64, f64, f64) = UI_BTN_TXT_HOVER_CLR;

pub const UI_BOX_H          : f64 = 200.0;
pub const UI_BOX_BORD       : f64 =   3.0;
pub const UI_MARGIN         : f64 =   4.0;
pub const UI_PADDING        : f64 =   6.0;
pub const UI_ELEM_TXT_H     : f64 =  16.0;
pub const UI_SAFETY_PAD     : f64 =   1.0;

pub const UI_INPUT_W            : f64 = 90.0;
pub const UI_INPUT_FONT_SIZE    : f64 = 16.0;
pub const UI_INPUT_BORDER_CLR   : (f64, f64, f64) = UI_BTN_BORDER2_CLR;
pub const UI_INPUT_BG_CLR       : (f64, f64, f64) = UI_BTN_BG_CLR;
pub const UI_INPUT_BORDER_WIDTH : f64 = UI_BTN_BORDER2_WIDTH;

pub const UI_DRAG_INFO_W         : f64 = 70.0;
pub const UI_DRAG_INFO_FONT_SIZE : f64 = 10.0;

pub const UI_GRID_TXT_CENTER_CLR    : (f64, f64, f64) = UI_PRIM_CLR;
pub const UI_GRID_TXT_CENTER_HL_CLR : (f64, f64, f64) = UI_HLIGHT_CLR;
pub const UI_GRID_TXT_CENTER_SL_CLR : (f64, f64, f64) = UI_SELECT_CLR;
pub const UI_GRID_TXT_EDGE_CLR      : (f64, f64, f64) = UI_PRIM_CLR;
pub const UI_GRID_CELL_BORDER_CLR   : (f64, f64, f64) = UI_ACCENT_CLR;
pub const UI_GRID_EMPTY_BORDER_CLR  : (f64, f64, f64) = UI_ACCENT_DARK_CLR;
pub const UI_GRID_HOVER_BORDER_CLR  : (f64, f64, f64) = UI_ACCENT_CLR;
pub const UI_GRID_DRAG_BORDER_CLR   : (f64, f64, f64) = UI_HLIGHT_CLR;
pub const UI_GRID_BG1_CLR           : (f64, f64, f64) = UI_ACCENT_BG1_CLR;
pub const UI_GRID_BG2_CLR           : (f64, f64, f64) = UI_ACCENT_BG2_CLR;
pub const UI_GRID_SIGNAL_OUT_CLR    : (f64, f64, f64) = UI_PRIM2_CLR;
pub const UI_GRID_SIGNAL_IN_CLR     : (f64, f64, f64) = UI_HLIGHT_CLR;
pub const UI_GRID_LED_CLR           : (f64, f64, f64) = UI_PRIM_CLR;
pub const UI_GRID_LED_R             : f64             = 5.0;

pub const UI_ENTRY_BORDER_WIDTH     : f64 = UI_BTN_BORDER2_WIDTH;
pub const UI_ENTRY_BORDER_CLR       : (f64, f64, f64) = UI_ACCENT_CLR;
pub const UI_ENTRY_TXT_CLR          : (f64, f64, f64) = UI_PRIM_CLR;

pub const UI_LIST_BORDER_WIDTH     : f64 = UI_ENTRY_BORDER_WIDTH;
pub const UI_LIST_BTN_WIDTH        : f64 = 20.0;
pub const UI_LIST_BTN_POINTER_SIZE : f64 = 6.0;
pub const UI_LIST_BTN_BORDER_WIDTH : f64 = 1.0;
pub const UI_LIST_BORDER_CLR       : (f64, f64, f64) = UI_ENTRY_BORDER_CLR;
pub const UI_LIST_SEP_CLR          : (f64, f64, f64) = UI_PRIM2_CLR;
pub const UI_LIST_TXT_CLR          : (f64, f64, f64) = UI_PRIM_CLR;
pub const UI_LIST_TXT_HOVER_CLR    : (f64, f64, f64) = UI_BTN_TXT_HOVER_CLR;

pub const UI_TRK_ROW_HEIGHT        : f64 = 14.0;
pub const UI_TRK_COL_WIDTH         : f64 = 38.0;
pub const UI_TRK_FONT_SIZE         : f64 = 12.0;
pub const UI_TRK_BORDER            : f64 = 1.0;
pub const UI_TRK_COL_DIV_PAD       : f64 = 3.0;
pub const UI_TRK_BG_CLR            : (f64, f64, f64) = UI_LBL_BG_CLR;
pub const UI_TRK_BG_ALT_CLR        : (f64, f64, f64) = UI_LBL_BG_ALT_CLR;
pub const UI_TRK_COL_DIV_CLR       : (f64, f64, f64) = UI_LIST_SEP_CLR;
pub const UI_TRK_BORDER_CLR        : (f64, f64, f64) = UI_ACCENT_CLR;
pub const UI_TRK_BORDER_HOVER_CLR  : (f64, f64, f64) = UI_HLIGHT_CLR;
pub const UI_TRK_BORDER_EDIT_CLR   : (f64, f64, f64) = UI_SELECT_CLR;
pub const UI_TRK_BORDER_INACT_CLR  : (f64, f64, f64) = UI_INACTIVE_CLR;
pub const UI_TRK_TEXT_CLR          : (f64, f64, f64) = UI_TXT_CLR;
pub const UI_TRK_CURSOR_BG_CLR     : (f64, f64, f64) = UI_PRIM_CLR;
pub const UI_TRK_CURSOR_FG_CLR     : (f64, f64, f64) = UI_LBL_BG_CLR;
pub const UI_TRK_PHASEROW_BG_CLR   : (f64, f64, f64) = UI_HLIGHT_CLR;
pub const UI_TRK_PHASEROW_FG_CLR   : (f64, f64, f64) = UI_LBL_BG_CLR;
