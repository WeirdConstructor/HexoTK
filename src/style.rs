// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::hxclr;
use crate::Rect;

use std::rc::Rc;

pub const UI_BOX_H: f32 = 200.0;
pub const UI_BOX_BORD: f32 = 2.0;
pub const UI_MARGIN: f32 = 4.0;
pub const UI_PADDING: f32 = 6.0;
pub const UI_ELEM_TXT_H: f32 = 16.0;
pub const UI_SAFETY_PAD: f32 = 1.0;

pub const UI_BG_CLR: (f32, f32, f32) = hxclr!(0x414a51); // 473f49
pub const UI_BG2_CLR: (f32, f32, f32) = hxclr!(0x4b535a); // 594f5d
pub const UI_BG3_CLR: (f32, f32, f32) = hxclr!(0x545b61); // 645868
pub const UI_TXT_CLR: (f32, f32, f32) = hxclr!(0xdcdcf0);
pub const UI_BORDER_CLR: (f32, f32, f32) = hxclr!(0x163239); // 2b0530);
pub const UI_LBL_BG_CLR: (f32, f32, f32) = hxclr!(0x111920); // hxclr!(0x16232f); // 1a2733); // 200e1f);
pub const UI_LBL_BG_ALT_CLR: (f32, f32, f32) = hxclr!(0x2d4d5e); // 323237
pub const UI_ACCENT_CLR: (f32, f32, f32) = hxclr!(0x922f93); // b314aa);
pub const UI_ACCENT_DARK_CLR: (f32, f32, f32) = hxclr!(0x1e333d); // 4d184a); // 4d184a);
pub const UI_ACCENT_BG1_CLR: (f32, f32, f32) = hxclr!(0x111920); // UI_LBL_BG_CLR; // hxclr!(0x111920); // UI_LBL_BG_CLR; // hxclr!(0x27091b); // 381c38); // 200e1f);
pub const UI_ACCENT_BG2_CLR: (f32, f32, f32) = hxclr!(0x192129); // 2c132a);
pub const UI_PRIM_CLR: (f32, f32, f32) = hxclr!(0x03fdcb); // 69e8ed
pub const UI_PRIM2_CLR: (f32, f32, f32) = hxclr!(0x228f9d); // 1aaeb3
pub const UI_HLIGHT_CLR: (f32, f32, f32) = hxclr!(0xecf9ce); // e9f840
pub const UI_HLIGHT2_CLR: (f32, f32, f32) = hxclr!(0xbcf9cd); // b5c412
pub const UI_SELECT_CLR: (f32, f32, f32) = hxclr!(0xd73988); // 0xdc1821);
pub const UI_INACTIVE_CLR: (f32, f32, f32) = hxclr!(0x6f8782);
pub const UI_INACTIVE2_CLR: (f32, f32, f32) = hxclr!(0xa6dbd0);

pub fn get_ui_colors() -> Vec<(&'static str, (f32, f32, f32))> {
    vec![
        ("UI_BG_CLR", UI_BG_CLR),
        ("UI_BG2_CLR", UI_BG2_CLR),
        ("UI_BG3_CLR", UI_BG3_CLR),
        ("UI_TXT_CLR", UI_TXT_CLR),
        ("UI_BORDER_CLR", UI_BORDER_CLR),
        ("UI_LBL_BG_CLR", UI_LBL_BG_CLR),
        ("UI_LBL_BG_ALT_CLR", UI_LBL_BG_ALT_CLR),
        ("UI_ACCENT_CLR", UI_ACCENT_CLR),
        ("UI_ACCENT_DARK_CLR", UI_ACCENT_DARK_CLR),
        ("UI_ACCENT_BG1_CLR", UI_ACCENT_BG1_CLR),
        ("UI_ACCENT_BG2_CLR", UI_ACCENT_BG2_CLR),
        ("UI_PRIM_CLR", UI_PRIM_CLR),
        ("UI_PRIM2_CLR", UI_PRIM2_CLR),
        ("UI_HLIGHT_CLR", UI_HLIGHT_CLR),
        ("UI_HLIGHT2_CLR", UI_HLIGHT2_CLR),
        ("UI_SELECT_CLR", UI_SELECT_CLR),
        ("UI_INACTIVE_CLR", UI_INACTIVE_CLR),
        ("UI_INACTIVE2_CLR", UI_INACTIVE2_CLR),
    ]
}

pub fn get_standard_colors() -> Vec<(f32, f32, f32)> {
    vec![
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
    ]
}

#[derive(Debug, Clone, Copy)]
pub enum Align {
    Center,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub enum VAlign {
    Middle,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy)]
pub enum BorderStyle {
    Rect,
    Hex { offset: f32 },
    Bevel { corner_offsets: (f32, f32, f32, f32) },
}

#[derive(Debug, Clone)]
pub enum StyleExt {
    None,
    Graph {
        graph_line: f32,
        vline1: f32,
        vline2: f32,
        vline1_color: (f32, f32, f32),
        vline2_color: (f32, f32, f32),
        hline: f32,
        hline_color: (f32, f32, f32),
    },
    PatternEditor {
        row_height: f32,
        col_width: f32,
        col_div_pad: f32,
    },
    BlockCode {
        with_markers: bool,
        grid_marker_color: (f32, f32, f32),
        block_bg_hover_color: (f32, f32, f32),
        block_bg_color: (f32, f32, f32),
        port_select_color: (f32, f32, f32),
    },
}

#[derive(Debug, Clone)]
pub struct Style {
    pub border_style: BorderStyle,
    pub bg_color: (f32, f32, f32),
    pub border_color: (f32, f32, f32),
    pub border2_color: (f32, f32, f32),
    pub color: (f32, f32, f32),
    pub color2: (f32, f32, f32),
    pub border: f32,
    pub border2: f32,
    pub line: f32,
    pub pad_left: f32,
    pub pad_right: f32,
    pub pad_top: f32,
    pub pad_bottom: f32,
    pub pad_item: f32,
    pub shadow_offs: (f32, f32),
    pub shadow_color: (f32, f32, f32),
    pub hover_shadow_color: (f32, f32, f32),
    pub hover_border_color: (f32, f32, f32),
    pub hover_color: (f32, f32, f32),
    pub active_shadow_color: (f32, f32, f32),
    pub active_border_color: (f32, f32, f32),
    pub active_color: (f32, f32, f32),
    pub inactive_color: (f32, f32, f32),
    pub selected_color: (f32, f32, f32),
    pub text_align: Align,
    pub text_valign: VAlign,
    pub font_size: f32,
    pub colors: Vec<(f32, f32, f32)>,
    pub ext: StyleExt,
}

impl Style {
    pub fn new() -> Self {
        let colors = get_standard_colors();

        Self {
            bg_color: UI_BG_CLR,
            border_color: UI_BORDER_CLR,
            border2_color: UI_ACCENT_CLR,
            color: UI_PRIM_CLR,
            color2: UI_PRIM2_CLR,
            border: UI_BOX_BORD,
            border2: UI_BOX_BORD,
            line: 2.0 * UI_BOX_BORD,
            border_style: BorderStyle::Rect,
            pad_left: 0.0,
            pad_right: 0.0,
            pad_top: 0.0,
            pad_bottom: 0.0,
            pad_item: 0.0,
            shadow_offs: (0.0, 0.0),
            shadow_color: UI_PRIM_CLR,
            hover_shadow_color: UI_SELECT_CLR,
            hover_border_color: UI_HLIGHT_CLR,
            hover_color: UI_HLIGHT_CLR,
            active_shadow_color: UI_HLIGHT_CLR,
            active_border_color: UI_SELECT_CLR,
            active_color: UI_HLIGHT2_CLR,
            inactive_color: UI_INACTIVE_CLR,
            selected_color: UI_SELECT_CLR,
            text_align: Align::Center,
            text_valign: VAlign::Middle,
            font_size: 14.0,
            ext: StyleExt::None,
            colors,
        }
    }

    pub fn apply_padding(&self, dpi_f: f32, pos: Rect) -> Rect {
        Rect {
            x: pos.x + dpi_f * self.pad_left,
            y: pos.y + dpi_f * self.pad_top,
            w: pos.w - dpi_f * (self.pad_left + self.pad_right),
            h: pos.h - dpi_f * (self.pad_top + self.pad_bottom),
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

    pub fn choose_shadow_color(&self, is_active: bool, is_hovered: bool) -> (f32, f32, f32) {
        if is_active {
            self.active_shadow_color
        } else if is_hovered {
            self.hover_shadow_color
        } else {
            self.shadow_color
        }
    }

    pub fn choose_border_color(&self, is_active: bool, is_hovered: bool) -> (f32, f32, f32) {
        if is_active {
            self.active_border_color
        } else if is_hovered {
            self.hover_border_color
        } else {
            self.border_color
        }
    }

    pub fn choose_color(&self, is_active: bool, is_hovered: bool) -> (f32, f32, f32) {
        if is_active {
            self.active_color
        } else if is_hovered {
            self.hover_color
        } else {
            self.color
        }
    }
}

pub struct DPIStyle<'a> {
    style: &'a Style,
    dpi_factor: f32,
}

macro_rules! dpi_accessor {
    ($field: ident) => {
        pub fn $field(&self) -> f32 {
            self.style.$field * self.dpi_factor
        }
    };
}

macro_rules! dpi_ext_accessor {
    ($enum: ident :: $opt: ident, $field: ident, $default: expr) => {
        pub fn $field(&self) -> f32 {
            if let $enum::$opt { $field, .. } = &self.style.ext {
                $field * self.dpi_factor
            } else {
                $default * self.dpi_factor
            }
        }
    };
}

macro_rules! color_accessor {
    ($field: ident) => {
        pub fn $field(&self) -> (f32, f32, f32) {
            self.style.$field
        }
    };
}

macro_rules! color_ext_accessor {
    ($enum: ident :: $opt: ident, $field: ident, $default: expr) => {
        pub fn $field(&self) -> (f32, f32, f32) {
            if let $enum::$opt { $field, .. } = &self.style.ext {
                *$field
            } else {
                $default
            }
        }
    };
}

macro_rules! bool_ext_accessor {
    ($enum: ident :: $opt: ident, $field: ident, $default: expr) => {
        pub fn $field(&self) -> bool {
            if let $enum::$opt { $field, .. } = &self.style.ext {
                *$field
            } else {
                $default
            }
        }
    };
}

impl<'a> DPIStyle<'a> {
    pub fn new_from(dpi_factor: f32, style: &'a Style) -> Self {
        Self { style, dpi_factor }
    }

    pub fn border_style(&self) -> BorderStyle {
        match self.style.border_style {
            BorderStyle::Rect => BorderStyle::Rect,
            BorderStyle::Hex { offset } => BorderStyle::Hex { offset: offset * self.dpi_factor },
            BorderStyle::Bevel { corner_offsets } => BorderStyle::Bevel {
                corner_offsets: (
                    corner_offsets.0 * self.dpi_factor,
                    corner_offsets.1 * self.dpi_factor,
                    corner_offsets.2 * self.dpi_factor,
                    corner_offsets.3 * self.dpi_factor,
                ),
            },
        }
    }

    pub fn text_align(&self) -> Align {
        self.style.text_align
    }

    pub fn text_valign(&self) -> VAlign {
        self.style.text_valign
    }

    pub fn color_by_idx(&self, idx: usize) -> (f32, f32, f32) {
        self.style.color_by_idx(idx)
    }

    pub fn choose_shadow_color(&self, is_active: bool, is_hovered: bool) -> (f32, f32, f32) {
        self.style.choose_shadow_color(is_active, is_hovered)
    }

    pub fn choose_border_color(&self, is_active: bool, is_hovered: bool) -> (f32, f32, f32) {
        self.style.choose_border_color(is_active, is_hovered)
    }

    pub fn choose_color(&self, is_active: bool, is_hovered: bool) -> (f32, f32, f32) {
        self.style.choose_color(is_active, is_hovered)
    }

    pub fn shadow_offs(&self) -> (f32, f32) {
        (self.style.shadow_offs.0 * self.dpi_factor, self.style.shadow_offs.1 * self.dpi_factor)
    }

    pub fn apply_padding(&self, pos: Rect) -> Rect {
        self.style.apply_padding(self.dpi_factor, pos)
    }

    dpi_accessor! {border}
    dpi_accessor! {border2}
    dpi_accessor! {line}
    dpi_accessor! {pad_left}
    dpi_accessor! {pad_right}
    dpi_accessor! {pad_top}
    dpi_accessor! {pad_bottom}
    dpi_accessor! {pad_item}
    dpi_accessor! {font_size}

    dpi_ext_accessor! {StyleExt::Graph, graph_line, 0.9}
    dpi_ext_accessor! {StyleExt::Graph, vline1, 1.0}
    dpi_ext_accessor! {StyleExt::Graph, vline2, 1.0}
    dpi_ext_accessor! {StyleExt::Graph, hline, 0.0}

    dpi_ext_accessor! {StyleExt::PatternEditor, row_height, 14.0}
    dpi_ext_accessor! {StyleExt::PatternEditor, col_width, 38.0}
    dpi_ext_accessor! {StyleExt::PatternEditor, col_div_pad, 3.0}

    color_accessor! {bg_color}
    color_accessor! {color}
    color_accessor! {color2}
    color_accessor! {border_color}
    color_accessor! {border2_color}
    color_accessor! {shadow_color}
    color_accessor! {hover_shadow_color}
    color_accessor! {hover_border_color}
    color_accessor! {hover_color}
    color_accessor! {active_shadow_color}
    color_accessor! {active_border_color}
    color_accessor! {active_color}
    color_accessor! {inactive_color}
    color_accessor! {selected_color}

    color_ext_accessor! {StyleExt::Graph, hline_color, UI_ACCENT_CLR}
    color_ext_accessor! {StyleExt::Graph, vline1_color, UI_PRIM2_CLR}
    color_ext_accessor! {StyleExt::Graph, vline2_color, UI_PRIM_CLR}

    color_ext_accessor! {StyleExt::BlockCode, grid_marker_color, UI_ACCENT_DARK_CLR}
    color_ext_accessor! {StyleExt::BlockCode, block_bg_hover_color, UI_ACCENT_CLR}
    color_ext_accessor! {StyleExt::BlockCode, block_bg_color, UI_ACCENT_BG2_CLR}
    color_ext_accessor! {StyleExt::BlockCode, port_select_color, UI_SELECT_CLR}

    bool_ext_accessor! {StyleExt::BlockCode, with_markers, false}
}
