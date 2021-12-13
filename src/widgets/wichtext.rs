// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::{Widget, InputEvent, Event, MButton, EvPayload, Style};
use crate::painter::*;
use crate::rect::*;

use std::rc::Rc;
use std::cell::RefCell;

use std::collections::HashMap;
use std::str::Chars;
use std::iter::Peekable;

pub trait WichTextData {
    /// The text of WichText. If you change this, you need to increase
    /// the [WichTextData::text_generation] to let WichText reparse it.
    fn text(&self) -> String;
    /// The generation counter tells WichText when to reparse the output of
    /// [WichTextData::text].
    fn text_generation(&self) -> usize;

    /// Change the knobs normalized value to `v`.
    fn knob_set(&self, key: &str, v: f32);
    /// Retrieve the knobs normalized value in the range 0.0 to 1.0.
    fn knob_value(&self, key: &str) -> f32;
    /// Clamp the knob value to a range inside 0.0 to 1.0
    fn knob_clamp(&self, key: &str, v: f32) -> f32;
    /// Map the normalized value to the UI range
    fn knob_map_ui_range(&self, key: &str, v: f32) -> f32;
    /// The step of a knob that is applied when dragging the knob.
    fn knob_step(&self, key: &str) -> f32;
    /// Format the value to text, here you can calculate the denormalized
    /// value for the given normalized value `v` and write the label
    /// text to `buf`.
    fn knob_fmt(&self, key: &str, v: f32, buf: &mut [u8]) -> usize;

    /// Retrieve the graph data source for the given `key`.
    fn data_source(&self, key: &str) -> Option<Rc<dyn WichTextDataSource>>;

    /// Check if anything of this data changed and it should be redrawn.
    fn check_change(&self) -> bool;
}

pub trait WichTextDataSource {
    fn samples(&self) -> usize;
    fn sample(&self, i: usize) -> f32;
    fn generation(&self) -> usize { 1 }
}

impl WichTextDataSource for Vec<f32> {
    fn samples(&self) -> usize { self.len() }
    fn sample(&self, i: usize) -> f32 { self.get(i).copied().unwrap_or(0.0) }
}

impl WichTextDataSource for Rc<RefCell<Vec<f32>>> {
    fn samples(&self) -> usize { self.borrow().len() }
    fn sample(&self, i: usize) -> f32 { self.borrow().get(i).copied().unwrap_or(0.0) }
}

#[derive(Debug)]
struct WichTextSimpleDataStoreImpl {
    text:           String,
    text_gen:       usize,

    knobs:          HashMap<String, f32>,
    data_sources:   HashMap<String, Rc<Vec<f32>>>,

    changed:        bool,
}

#[derive(Debug, Clone)]
pub struct WichTextSimpleDataStore(Rc<RefCell<WichTextSimpleDataStoreImpl>>);

impl WichTextSimpleDataStore {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(WichTextSimpleDataStoreImpl {
            text:           String::from(""),
            text_gen:       0,
            knobs:          HashMap::new(),
            data_sources:   HashMap::new(),
            changed:        false,
        })))
    }

    pub fn set_text(&self, text: String) {
        self.0.borrow_mut().text = text;
        self.0.borrow_mut().text_gen += 1;
        self.0.borrow_mut().changed = true;
    }
}

impl WichTextData for WichTextSimpleDataStore {
    fn text(&self) -> String { self.0.borrow().text.clone() }
    fn text_generation(&self) -> usize { self.0.borrow().text_gen }

    fn knob_set(&self, key: &str, v: f32) {
        let mut bor = self.0.borrow_mut();
        if let Some(val) = bor.knobs.get_mut(key) {
            *val = v;
        } else {
            bor.knobs.insert(key.to_string(), v);
        }

        bor.changed = true;
    }

    fn knob_value(&self, key: &str) -> f32 {
        self.0.borrow().knobs.get(key).copied().unwrap_or(0.0)
    }

    fn knob_clamp(&self, key: &str, v: f32) -> f32 { v.clamp(0.0, 1.0) }
    fn knob_map_ui_range(&self, key: &str, v: f32) -> f32 { v.clamp(0.0, 1.0) }
    fn knob_step(&self, key: &str) -> f32 { 0.05 }
    fn knob_fmt(&self, key: &str, v: f32, buf: &mut [u8]) -> usize {
        use std::io::Write;
        let mut bw = std::io::BufWriter::new(buf);

//        let prec =
//            if let Some((_, _, prec)) = self.borrow().get(key) {
//                *prec
//            } else { 2 };
        let prec = 2;

        match write!(bw, "{:5.prec$}", v, prec = prec as usize) {
            Ok(_) => bw.buffer().len(),
            _     => 0,
        }
    }

    fn data_source(&self, key: &str) -> Option<Rc<dyn WichTextDataSource>> {
        if let Some(data) = self.0.borrow().data_sources.get(key).cloned() {
            Some(data)
        } else {
            None
        }
    }

    fn check_change(&self) -> bool {
        let mut ch_borrow = self.0.borrow_mut();
        let is_changed = ch_borrow.changed;
        ch_borrow.changed = false;
        is_changed
    }
}

#[derive(Debug, Clone)]
enum FragType {
    Text,
    Graph { key: String },
    Value { key: String },
}

impl FragType {
    fn is_value(&self) -> bool {
        if let FragType::Value { .. } = self {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct WTFragment {
    typ:            FragType,
    font_size:      f32,
    color:          usize,
    color2:         usize,
    is_active:      bool,
    text:           String,
    cmd:            Option<String>,
    chars:          Vec<char>,
    ext_size_px:    (f32, f32),
    space_px:       f32,
    width_px:       f32,
    height_px:      f32,
    x:              f32,
}

impl WTFragment {
    fn new(font_size: f32) -> Self {
        Self {
            font_size,
            typ:            FragType::Text,
            color:          9,
            color2:         17,
            is_active:      false,
            text:           String::from(""),
            cmd:            None,
            chars:          vec![],
            ext_size_px:    (0.0, 0.0),
            width_px:       0.0,
            space_px:       0.0,
            height_px:      0.0,
            x:              0.0,
        }
    }

    fn push_char(&mut self, c: char) {
        self.chars.push(c);
    }

    fn finish(&mut self, p: &mut Painter) {
        if self.is_active {
            self.cmd = Some(self.chars.iter().collect());

            self.chars.insert(0, '[');
            self.chars.push(']');
            self.text = self.chars.iter().collect();
        }

        self.text = self.chars.iter().collect();

        let fs = self.font_size;

        self.space_px = p.text_width(fs, true, " ");

        match self.typ {
            FragType::Value { .. } => {
                if self.is_active && self.height_px < 1.0 {
                    self.height_px = 40.0;
                }

                self.width_px =
                    self.width_px.max(
                        p.text_width(fs, true, &self.text) + 6.0);
                self.height_px += 2.0 * p.font_height(fs, true);
                self.ext_size_px.1 = p.font_height(fs, true);
            },
            FragType::Graph { .. } => {
                self.ext_size_px.0 = p.text_width(fs, true, &self.text) + 1.0;
                self.ext_size_px.1 = p.font_height(fs, true);
                self.height_px += self.ext_size_px.1;
                self.width_px   = self.ext_size_px.0.max(self.width_px);
            },
            FragType::Text => {
                let w = p.text_width(fs, true, &self.text);
                self.width_px  = if w > 0.01 { w + 1.0 } else { 0.0 };
                self.height_px = p.font_height(fs, true);
            }
        }
    }

    fn draw<F: for<'a> Fn(&str, &'a mut [u8]) -> (&'a str, f32)>(
        &self,
        p: &mut Painter,
        data:         &Rc<dyn WichTextData>,
        data_sources: &mut HashMap<String, DataSource>,
        fetch_value: &F,
        pos:        Rect,
        bg_clr:     (f32, f32, f32),
        color:      (f32, f32, f32),
        color2:     (f32, f32, f32),
        orig_color: (f32, f32, f32))
    {
        match &self.typ {
            FragType::Graph { key } => {
                let graph_h = pos.h - self.ext_size_px.1 - 2.0;
                let graph_w = pos.w - 2.0;
                p.rect_stroke(
                    1.0,
                    color,
                    pos.x + 0.5,
                    pos.y + 1.5,
                    graph_w,
                    graph_h);

                if let Some(src) = data_sources.get_mut(key) {
                    src.create_points(graph_w, graph_h);
                    src.draw(p, pos.x, pos.y, color2);
                } else {
                    if let Some(src) = data.data_source(key) {
                        let mut data_src = DataSource::new(src);
                        data_src.create_points(graph_w, graph_h);
                        data_src.draw(p, pos.x, pos.y, color2);
                        data_sources.insert(key.to_string(), data_src);
                    }
                }

                p.label_mono(
                    self.font_size,
                    0,
                    color,
                    pos.x,
                    pos.y + graph_h,
                    graph_w,
                    self.ext_size_px.1,
                    &self.text);

            },
            FragType::Value { key } => {
                let mut buf : [u8; 15] = [0; 15];
                let (val_s, knb_v) = (*fetch_value)(key, &mut buf[..]);
                let knob_h = pos.h - 2.0 * self.ext_size_px.1;

                p.rect_stroke(
                    1.0,
                    color,
                    pos.x + 1.5,
                    pos.y + 1.5,
                    pos.w - 3.0,
                    pos.h - self.ext_size_px.1 - 2.0);

                p.rect_fill(
                    bg_clr,
                    pos.x + 3.0,
                    pos.y + 3.0,
                    pos.w - 6.0,
                    pos.h - self.ext_size_px.1 - 5.0);

                let factor = knob_h / 40.0;

                if knob_h > 10.0 {
                    let r = knob_h - (10.0 * factor);
                    p.arc_stroke(1.0, orig_color, r * 0.5,
                        std::f32::consts::PI * 0.6,
                        std::f32::consts::PI * (0.6 + 1.8),
                        (pos.x + pos.w * 0.5).floor(),
                        (pos.y + 2.0 + knob_h * 0.5).floor());
                    p.arc_stroke(4.0 * factor, color2, r * 0.5,
                        std::f32::consts::PI * 0.6,
                        std::f32::consts::PI * (0.6 + 1.8 * knb_v),
                        (pos.x + pos.w * 0.5).floor(),
                        (pos.y + 2.0 + knob_h * 0.5).floor());
                }

                p.label_mono(
                    self.font_size,
                    1,
                    color2,
                    pos.x,
                    pos.y + knob_h,
                    pos.w - 3.0,
                    self.ext_size_px.1,
                    val_s);

                p.label_mono(
                    self.font_size,
                    0,
                    color,
                    pos.x,
                    pos.y + knob_h + self.ext_size_px.1,
                    pos.w, self.ext_size_px.1,
                    &self.text);

            },
            FragType::Text => {
                p.label_mono(
                    self.font_size,
                    -1,
                    color,
                    pos.x, pos.y, pos.w, pos.h,
                    &self.text);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum VAlign {
    Bottom,
    Top,
    Middle,
}

impl VAlign {
    fn from_char(c: char) -> Self {
        match c {
            't'     => VAlign::Top,
            'm'     => VAlign::Middle,
            'b' | _ => VAlign::Bottom,
        }
    }
}

#[derive(Debug, Clone)]
struct WTLine {
    frags:  Vec<WTFragment>,
    line_h: f32,
    line_y: f32,
    align:  VAlign,
    wrap:   bool,
}

impl WTLine {
    fn new() -> Self {
        Self {
            frags:  vec![],
            line_h: 0.0,
            line_y: 0.0,
            align:  VAlign::Bottom,
            wrap:   false,
        }
    }

    fn add(&mut self, frag: WTFragment) { self.frags.push(frag); }

    fn calc_cur_w(&self, wrap: bool, tail_frag: Option<&WTFragment>) -> f32 {
        let mut w            = 0.0;
        let mut next_space_w = 0.0;

        for frag in &self.frags {
            if w > 0.1 && wrap {
                w += next_space_w;
            }

            w += frag.width_px;
            if frag.width_px > 0.1 {
                next_space_w = frag.space_px;
            } else {
                next_space_w = 0.0;
            }
        }

        if let Some(frag) = tail_frag {
            if w > 0.1 && wrap {
                w += next_space_w;
            }

            w += frag.width_px;
        }

        w
    }

    fn finish(&mut self, align: VAlign, wrap: bool, y: f32) -> f32 {
        let mut line_h = 3.0_f32;
        let mut x      = 0.0;
        let mut next_space_w = 0.0;

        for frag in &mut self.frags {
            if x > 0.1 && wrap {
                x += next_space_w;
            }

            //d// println!("FRAG: {}px for: '{}'", frag.height_px, frag.text);
            line_h = line_h.max(frag.height_px);
            frag.x = x;

            x += frag.width_px;
            if frag.width_px > 0.1 {
                next_space_w = frag.space_px;
            } else {
                next_space_w = 0.0;
            }
        }

        self.wrap   = wrap;
        self.align  = align;
        self.line_h = line_h;
        self.line_y = y;

        line_h
    }
}

#[derive(Clone)]
struct DataSource {
    generation:         usize,
    cache_graph_size:   (f32, f32),
    point_cache:        Vec<(f32, f32)>,
    source:             Rc<dyn WichTextDataSource>,
}

impl DataSource {
    fn new(source: Rc<dyn WichTextDataSource>) -> Self {
        Self {
            generation:         0,
            cache_graph_size:   (0.0, 0.0),
            point_cache:        vec![],
            source,
        }
    }

    fn create_points(&mut self, graph_w: f32, graph_h: f32) {
        let graph_size = (graph_w.floor(), graph_h.floor());
        if    self.generation       == self.source.generation()
           && self.cache_graph_size == graph_size
        {
            return;
        }

        self.point_cache.clear();
        self.cache_graph_size = graph_size;

        println!("REDRAW POINTS {:?}", graph_size);

        if self.source.samples() > 0 {
            let xd =
                (graph_w - 1.0)
                / (self.source.samples() as f32 - 1.0);
            let mut x = 0.0;

            for i in 0..self.source.samples() {
                let s = 1.0 - self.source.sample(i).clamp(0.0, 1.0);
                self.point_cache.push((x, s * (graph_h - 2.0)));
                x += xd;
            }
        }

        self.generation = self.source.generation();
    }

    fn draw(&self, p: &mut Painter, x: f32, y: f32, color: (f32, f32, f32)) {
        p.path_stroke(
            1.0, color,
            &mut self.point_cache.iter().copied()
                .map(|p| (
                    (p.0 + x + 0.5).round(),
                    (p.1 + y + 1.5).round() + 0.5)),
            false);
    }
}

pub struct WichText {
    data:               Rc<dyn WichTextData>,
    text_generation:    usize,

    lines:              Vec<WTLine>,
    wrapped_lines:      Vec<WTLine>,
    full_h:             f32,
    last_width:         i64,

    zones:              Vec<(Rect, usize, usize)>,

    hover:              Option<(usize, usize)>,
    active:             Option<(usize, usize)>,
    drag:               Option<(f32, f32, f32, f32, f32, String)>,

    scroll:             (f32, f32),
    render:             (f32, f32),
    pan_pos:            Option<(f32, f32)>,

    data_sources:       HashMap<String, DataSource>,

    mouse_pos:          (f32, f32),
}

fn parse_key(ci: &mut Peekable<Chars<'_>>) -> String {
    let mut key = String::from("");
    while let Some(c) = ci.peek().copied() {
        if c != ':' {
            ci.next();
            key.push(c);
        } else {
            break;
        }
    }
    key
}

fn parse_number<T: std::str::FromStr>(ci: &mut Peekable<Chars<'_>>, default: T) -> T {
    let mut s = String::from("");

    while let Some(c) = ci.peek().copied() {
        if c.is_ascii_digit() {
            ci.next();
            s.push(c);
        } else {
            break;
        }
    }

    s.parse::<T>().unwrap_or(default)
}

impl WichText {
    pub fn new(data: Rc<dyn WichTextData>) -> Self {
        Self {
            data,
            text_generation: 0,

            lines:           vec![],
            wrapped_lines:   vec![],
            full_h:          0.0,
            last_width:      0,
            data_sources:    HashMap::new(),

            zones:           vec![],

            scroll:          (0.0, 0.0),
            render:          (0.0, 0.0),
            pan_pos:         None,

            hover:           None,
            active:          None,
            drag:            None,

            mouse_pos:       (0.0, 0.0),
        }

    }

    pub fn data(&self) -> &Rc<dyn WichTextData> { &self.data }

//    pub fn on_value<F>(mut self, on_value: F) -> Self
//    where
//        F: 'static + Fn(&mut Self, &mut State, Entity, usize, usize, &str, f32),
//    {
//        self.on_value = Some(Box::new(on_value));
//
//        self
//    }
//
//    pub fn on_hover<F>(mut self, on_hover: F) -> Self
//    where
//        F: 'static + Fn(&mut Self, &mut State, Entity, bool, usize),
//    {
//        self.on_hover = Some(Box::new(on_hover));
//
//        self
//    }

    fn parse(&mut self, style_font_size: f32, p: &mut Painter, text: &str) {
        self.lines.clear();

        let mut cur_y = 0.0;

        for line in text.lines() {
            let mut frag_line = WTLine::new();
            let mut ci = line.chars().peekable();

            let mut cur_font_size = style_font_size;
            let mut cur_fragment  = WTFragment::new(cur_font_size);
            let mut first_frag    = true;
            let mut in_frag_start = false;
            let mut in_frag       = false;

            let mut align : VAlign = VAlign::Bottom;
            let mut wordwrap = false;

            while let Some(c) = ci.next() {
                //d// println!("CHAR:'{}' {},{}", c, in_frag_start, in_frag);
                if in_frag_start {
                    first_frag = false;

                    match c {
                        'L' => {
                            align = VAlign::from_char(ci.next().unwrap_or('b'));
                        },
                        'R' => { wordwrap = true; },
                        'v' => {
                            let key = parse_key(&mut ci);
                            cur_fragment.typ = FragType::Value { key };
                        },
                        'g' => {
                            let key = parse_key(&mut ci);
                            cur_fragment.typ = FragType::Graph { key };
                        },
                        'w' => {
                            let mut num = String::from("");
                            while let Some(c) = ci.peek().copied() {
                                if c.is_ascii_digit() {
                                    ci.next();
                                    num.push(c);
                                } else {
                                    break;
                                }
                            }

                            let w = num.parse::<f32>().unwrap_or(0.0);
                            cur_fragment.width_px      = w;
                            cur_fragment.ext_size_px.0 = w;
                        },
                        'h' => {
                            let h = parse_number::<f32>(&mut ci, 0.0);
                            cur_fragment.height_px     = h;
                            cur_fragment.ext_size_px.1 = h;
                        },
                        'c' => {
                            cur_fragment.color =
                                parse_number::<usize>(&mut ci, 0);
                        },
                        'C' => {
                            cur_fragment.color2 =
                                parse_number::<usize>(&mut ci, 0);
                        },
                        'f' => {
                            cur_font_size =
                                parse_number::<f32>(&mut ci, 0.0);
                            cur_fragment.font_size = cur_font_size;
                        },
                        'a' => {
                            cur_fragment.is_active = true;
                        },
                        ':' => {
                            in_frag_start = false;
                            in_frag       = true;
                        },
                        ']' => {
                            cur_fragment.finish(p);

                            frag_line.add(
                                std::mem::replace(
                                    &mut cur_fragment,
                                    WTFragment::new(cur_font_size)));

                            in_frag_start = false;
                            in_frag       = false;
                        },
                        _ => {
                            // ignore until ':'
                        },
                    }
                } else if in_frag {
                    match c {
                        ']' => {
                            let c2 = ci.peek().copied().unwrap_or('\0');
                            if c2 == ']' {
                                ci.next();
                                cur_fragment.push_char(']');
                            } else {
                                cur_fragment.finish(p);

                                frag_line.add(
                                    std::mem::replace(
                                        &mut cur_fragment,
                                        WTFragment::new(cur_font_size)));

                                in_frag = false;
                            }
                        },
                        _ => {
                            cur_fragment.push_char(c);
                        }
                    }

                } else {
                    match c {
                        '[' => {
                            let c2 = ci.peek().copied().unwrap_or('\0');
                            if c2 == '[' {
                                ci.next();
                                cur_fragment.push_char('[');
                            } else {
                                if first_frag && cur_fragment.chars.len() == 0 {
                                    cur_fragment = WTFragment::new(cur_font_size);
                                } else {
                                    cur_fragment.finish(p);

                                    frag_line.add(
                                        std::mem::replace(
                                            &mut cur_fragment,
                                            WTFragment::new(cur_font_size)));
                                }

                                in_frag_start = true;
                            }
                        },
                        _ => {
                            if wordwrap {
                                if c.is_whitespace() {
                                    if cur_fragment.chars.len() > 0 {
                                        cur_fragment.finish(p);

                                        frag_line.add(
                                            std::mem::replace(
                                                &mut cur_fragment,
                                                WTFragment::new(cur_font_size)));
                                    }
                                } else {
                                    cur_fragment.push_char(c);
                                }
                            } else {
                                cur_fragment.push_char(c);
                            }
                        },
                    }
                }
            }

            if first_frag || cur_fragment.chars.len() > 0 {
                cur_fragment.finish(p);

                frag_line.add(
                    std::mem::replace(
                        &mut cur_fragment,
                        WTFragment::new(cur_font_size)));
            }

            let default_font_h = p.font_height(cur_font_size, true);
            let line_h = frag_line.finish(align, wordwrap, cur_y);
            self.lines.push(frag_line);

            cur_y += line_h;
        }
    }

    fn wrap_lines(&mut self, width: f32) {
        self.wrapped_lines.clear();

        let mut y = 0.0;

        for line in self.lines.iter() {
            if !line.wrap {
                let mut new_line = line.clone();
                y += new_line.finish(line.align, false, y);
                self.wrapped_lines.push(new_line);
                continue;
            }

            let mut cur_line = WTLine::new();

            for frag in &line.frags {
                let add_after =
                    if cur_line.calc_cur_w(true, Some(&frag)) <= width
                       || cur_line.frags.len() == 0
                    {
                        cur_line.add(frag.clone());
                        false
                    } else { true };

                if add_after || cur_line.calc_cur_w(true, None) > width {
                    y += cur_line.finish(line.align, true, y);

                    self.wrapped_lines.push(
                        std::mem::replace(&mut cur_line, WTLine::new()));
                }

                if add_after {
                    cur_line.add(frag.clone());
                }
            }

            if cur_line.frags.len() > 0 {
                y += cur_line.finish(line.align, true, y);
                self.wrapped_lines.push(cur_line);
            }
        }

        self.full_h = y;
    }

    fn find_frag_idx_at(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        for z in &self.zones {
            if z.0.is_inside(x, y) {
                return Some((z.1, z.2));
            }
        }

        None
    }

    fn clamp_scroll(&mut self, mut dx: f32, mut dy: f32) -> (f32, f32) {
        let max_scroll =
            if self.full_h > self.render.1 { self.full_h - self.render.1 }
            else { 0.0 };

        if let Some((px, py)) = self.pan_pos {
            dx += self.mouse_pos.0 - px;
            dy += self.mouse_pos.1 - py;
        }

        (self.scroll.0 + dx, (self.scroll.1 + dy).clamp(-max_scroll, 0.0))
    }

    fn drag_val(&self, mouse_y: f32) -> f32 {
        if let Some((_ox, oy, step, val, _tmp, _key)) = &self.drag {
            val + ((oy - mouse_y) / 20.0) * step
        } else {
            0.0
        }
    }

    fn get(&self, line: usize, frag: usize) -> Option<&WTFragment> {
        self.wrapped_lines.get(line)?.frags.get(frag)
    }

    fn get_mut(&mut self, line: usize, frag: usize) -> Option<&mut WTFragment> {
        self.wrapped_lines.get_mut(line)?.frags.get_mut(frag)
    }

    pub fn handle(
        &mut self, w: &Widget, event: &InputEvent, out_events: &mut Vec<(usize, Event)>)
    {
        let is_hovered = w.is_hovered();

        match event {
            InputEvent::MouseButtonPressed(btn) => {
                let (x, y) = self.mouse_pos;

                if is_hovered {
                    if *btn == MButton::Middle {
                        self.pan_pos = Some((x, y));
                        w.activate();

                    } else {
                        self.active = self.find_frag_idx_at(x, y);

                        if let Some((line, frag)) = self.active {
                            if let Some(FragType::Value { key }) =
                                self.get(line, frag).map(|f| &f.typ)
                            {
                                let s = self.data.knob_step(&key);
                                let v = self.data.knob_value(&key);

                                self.drag =
                                    Some((x, y, s, v, v, key.to_string()));
                                w.activate();
                            }
                        }
                    }

                    w.emit_redraw_required();
                }
            },
            InputEvent::MouseButtonReleased(btn) => {
                if w.is_active() {
                    let (x, y) = self.mouse_pos;

                    let cur = self.find_frag_idx_at(x, y);

                    if *btn == MButton::Middle {
                        self.scroll = self.clamp_scroll(0.0, 0.0);
                        self.pan_pos = None;

                    } else if self.active.is_some() && self.drag.is_some() {
                        let new_val = self.drag_val(y);
                        if let Some((_ox, _oy, _step, _val, _tmp, key)) =
                            self.drag.take()
                        {
                            let new_val =
                                self.data.knob_clamp(&key, new_val);
                            self.data.knob_set(&key, new_val);
                        }

                    } else if self.active.is_some() && self.active == cur {

                        if let Some((line, frag)) = self.active.take() {
                            if let Some(cmd) =
                                self.get_mut(line, frag)
                                    .map(|f| f.cmd.clone())
                                    .flatten()
                            {
                                out_events.push((w.id(), Event {
                                    name: "click".to_string(),
                                    data: EvPayload::WichTextCommand {
                                        line, frag, cmd,
                                    },
                                }));
                            }
                        }

                    }

                    w.deactivate();

                    self.active = None;
                    self.drag   = None;

                    w.emit_redraw_required();
                }
            },
            InputEvent::MouseWheel(scroll) => {
                if is_hovered {
                    if self.pan_pos.is_none() {
                        self.scroll = self.clamp_scroll(0.0, *scroll * 50.0);
                    }

                    w.emit_redraw_required();
                }
            },
            InputEvent::MousePosition(x, y) => {
                self.mouse_pos = (*x, *y);

                let old_hover = self.hover;
                self.hover = self.find_frag_idx_at(*x, *y);

                let d_val = self.drag_val(*y);

                if let Some((_ox, _oy, _step, _val, tmp, _key)) =
                    self.drag.as_mut()
                {
                    *tmp = d_val;

                    w.emit_redraw_required();
                }

                if self.pan_pos.is_some() {
                    w.emit_redraw_required();

                } else if old_hover != self.hover {
                    w.emit_redraw_required();
                }
            },
            _ => {},
        }
    }

    pub fn draw(&mut self, w: &Widget, style: &Style, pos: Rect, p: &mut Painter) {
        p.clip_region(pos.x, pos.y, pos.w, pos.h);
        p.rect_fill(style.bg_color, pos.x, pos.y, pos.w, pos.h);

        let pos = pos.floor();
        let pos = pos.crop_right(10.0);

        let scroll_box = Rect {
            x: pos.w + pos.x,
            y: pos.y,
            w: 10.0,
            h: pos.h,
        };

        if self.text_generation != self.data.text_generation() {
            self.parse(style.font_size, p, &self.data.text());
            self.last_width = 0;

            self.text_generation = self.data.text_generation();
        }

        let width = pos.w.floor() as i64;
        if self.last_width != width {
            self.wrap_lines(pos.w.floor());
            self.last_width = width;
        }

        self.zones.clear();

        self.render = (pos.w, pos.h);

        let (_scroll_x, scroll_y) = self.clamp_scroll(0.0, 0.0);

        let val_src = self.data.clone();
        let drag = self.drag.clone();

        let fetch_value
            : Box<dyn for<'a> Fn(&str, &'a mut [u8]) -> (&'a str, f32)> =
            Box::new(move |key: &str, buf: &mut [u8]| {
                let v = val_src.knob_value(&key);
                let v =
                    if let Some((_, _, _, _, tmp, drag_key)) = &drag {
                        if &key == &drag_key { *tmp }
                        else { v }
                    } else { v };

                let v     = val_src.knob_clamp(&key, v);
                let knb_v = val_src.knob_map_ui_range(&key, v);
                let len   = val_src.knob_fmt(&key, v, &mut buf[..]);
                let val_s = std::str::from_utf8(&buf[0..len]).unwrap();

                (val_s, knb_v)
            });

        for (line_idx, WTLine { frags, line_h, line_y, align, .. }) in
            self.wrapped_lines.iter().enumerate()
        {
            for (frag_idx, frag) in frags.iter().enumerate() {
                let valign_offs =
                    match align {
                        VAlign::Middle => ((line_h - frag.height_px) * 0.5).floor(),
                        VAlign::Top    => 0.0,
                        VAlign::Bottom => line_h - frag.height_px,
                    };

                let frag_pos = Rect {
                    x: pos.x + frag.x,
                    y: pos.y + line_y + valign_offs + scroll_y,
                    w: frag.width_px,
                    h: frag.height_px,
                };

                let frag_pos = frag_pos.floor();

                if (frag_pos.y + frag_pos.h) < 0.0 {
                    continue;
                } else if frag_pos.y > (pos.y + self.render.1) {
                    continue;
                }

                let mut color = style.color_by_idx(frag.color);
                let orig_color = color;

                let color2 = style.color_by_idx(frag.color2);

                if (self.active == self.hover || frag.typ.is_value())
                   && self.active == Some((line_idx, frag_idx)) {
                    color = style.active_border_color;

                } else if self.hover == Some((line_idx, frag_idx)) {
                    p.rect_fill(
                        color, frag_pos.x, frag_pos.y, frag_pos.w, frag_pos.h);
                    color = style.bg_color;
                }

                frag.draw(
                    p,
                    &self.data,
                    &mut self.data_sources,
                    &fetch_value,
                    frag_pos,
                    style.bg_color, color, color2, orig_color);

                if frag.is_active {
                    self.zones.push((frag_pos, line_idx, frag_idx));
                }
            }
        }

        if self.full_h > self.render.1 {
            let scroll_marker_h = (scroll_box.h / 20.0).floor();
            let max_scroll = self.full_h - self.render.1;
            let marker_y =
                (scroll_y / max_scroll)
                    // XXX: +1.0 for the extra pixel padding!
                * ((scroll_marker_h + 1.0) - scroll_box.h);

            p.rect_stroke(1.0, style.border_color,
                scroll_box.x + 0.5,
                scroll_box.y + 0.5,
                scroll_box.w - 1.0,
                scroll_box.h - 1.0);

            p.rect_fill(style.border_color,
                scroll_box.x + 2.0,
                marker_y + 2.0,
                scroll_box.w - 4.0,
                scroll_marker_h - 3.0);
        }

        p.reset_clip_region();
    }
}
