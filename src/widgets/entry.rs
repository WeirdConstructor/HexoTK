// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::{Widget, InputEvent, Event, MButton, EvPayload, Style, Mutable};
use keyboard_types::Key;

use crate::painter::*;
use crate::rect::*;

use std::rc::Rc;
use std::cell::RefCell;

pub trait EditableText : Mutable {
    fn update(&self, changed: String);
    fn get(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct TextField(Rc<RefCell<(String, u64)>>);

impl TextField {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new((String::from(""), 0))))
    }

    pub fn set(&self, new: String) {
        let mut tf = self.0.borrow_mut();
        tf.0 = new;
        tf.1 += 1;
    }
}

impl Mutable for TextField {
    fn get_generation(&mut self) -> u64 {
        let tf = self.0.borrow_mut();
        tf.1
    }
}

impl EditableText for TextField {
    fn update(&self, changed: String) {
        let mut tf = self.0.borrow_mut();
        tf.0 = changed;
    }

    fn get(&self) -> String {
        let tf = self.0.borrow_mut();
        tf.0.clone()
    }
}

pub struct Entry {
    update_text: Box<dyn EditableText>,
    data:        String,
    data_gen:    u64,
    pre_string:  String,
    post_string: String,
    cursor:      usize,
    mouse_pos:   (f32, f32),
}

impl Entry {
    pub fn new(update_text: Box<dyn EditableText>) -> Self {
        Self {
            update_text,
            data:        String::from(""),
            data_gen:    0,
            pre_string:  String::from(""),
            post_string: String::from(""),
            cursor:      0,
            mouse_pos:   (0.0, 0.0),
        }
    }

    pub fn get_generation(&mut self) -> u64 {
        let cur_data_gen = self.update_text.get_generation();

        if cur_data_gen != self.data_gen {
            self.data = self.update_text.get();
            self.cursor = self.data.len();
            self.update_cursor();
            self.data_gen = cur_data_gen;
        }

        cur_data_gen
    }

    fn update_cursor(&mut self) {
        self.pre_string  = self.data.chars().take(self.cursor).collect();
        self.post_string = self.data.chars().skip(self.cursor).collect();
    }

    pub fn draw(&mut self, w: &Widget, style: &Style, pos: Rect, real_pos: Rect, p: &mut Painter) {
        let real_offs = (real_pos.x - pos.x, real_pos.y - pos.y);

        p.clip_region(pos.x, pos.y, pos.w, pos.h);
        let is_hovered = w.is_hovered();
        let is_active  = w.is_active();

        let fh          = p.font_height(style.font_size, true);
        let cur_start_x = p.text_width(style.font_size, true, &self.pre_string[..]);

        let color =
            if is_active        { style.active_color }
            else if is_hovered  { style.hover_color }
            else                { style.color };

        let y = ((pos.h - fh) * 0.5).round();

        let mut xo    = 0.0;
        while (cur_start_x + xo) > 0.75 * pos.w {
            xo -= pos.w * 0.25;
        }
        xo = xo.round();

        let mut dbg = LblDebugTag::from_id(w.id());

        p.label_mono(
            style.font_size, -1, style.color,
            pos.x + xo, pos.y + y, pos.w, fh,
            &self.data,
            dbg.offs_src(real_offs, "text"));

        p.stroke(
            1.0, color, &[
                ((pos.x + cur_start_x + xo).round() + 0.5, pos.y + y),
                ((pos.x + cur_start_x + xo).round() + 0.5, pos.y + y + fh),
            ], false);
        p.reset_clip_region();
    }

    pub fn handle(
        &mut self, w: &Widget, event: &InputEvent, out_events: &mut Vec<(usize, Event)>)
    {
        let is_hovered = w.is_hovered();
        let is_active  = w.is_active();

        //d// println!("ENTRY EV: {:?}", event);

        let mut changed = false;

        match event {
            InputEvent::KeyPressed(key) => {
                let len = self.data.chars().count();

                if !is_active { return; }

                match &key.key {
                    Key::Character(s) => {
                        self.data =
                            self.pre_string.clone() + &s + &self.post_string;
                        changed = true;
                        self.cursor += 1;
                        self.update_cursor();
                        w.emit_redraw_required();
                    },
                    Key::Home => {
                        self.cursor = 0;
                        self.update_cursor();
                        w.emit_redraw_required();
                    },
                    Key::End => {
                        self.cursor = len;
                        self.update_cursor();
                        w.emit_redraw_required();
                    },
                    Key::Backspace => {
                        if self.cursor > 0 {
                            let spre : String =
                                self.pre_string
                                    .chars().take(self.cursor - 1).collect();
                            changed = true;
                            self.data = spre + &self.post_string;
                            self.cursor -= 1;
                            self.update_cursor();
                            w.emit_redraw_required();
                        }
                    },
                    Key::Delete => {
                        if self.cursor < len {
                            let spost : String =
                                self.post_string.chars().skip(1).collect();
                            changed = true;
                            self.data = self.pre_string.clone() + &spost;
                            self.update_cursor();
                            w.emit_redraw_required();
                        }
                    },
                    Key::ArrowRight => {
                        if (self.cursor + 1) <= len {
                            self.cursor += 1;
                            self.update_cursor();
                            w.emit_redraw_required();
                        }
                    },
                    Key::ArrowLeft => {
                        if self.cursor > 0 {
                            self.cursor -= 1;
                            self.update_cursor();
                            w.emit_redraw_required();
                        }
                    },
                    _ => { },
                }
            },
            InputEvent::MouseButtonPressed(btn) => {
                if is_hovered {
                    if *btn == MButton::Left {
                        w.activate();
                        w.emit_redraw_required();
                    }
                }
            },
            InputEvent::MouseButtonReleased(_btn) => {
//                if w.is_active() {
//                    let (x, y) = self.mouse_pos;
//
//                    if *btn == MButton::Left {
////                        w.deactivate();
////                        w.emit_redraw_required();
//                    }
//                }
            },
            InputEvent::MousePosition(x, y) => {
                self.mouse_pos = (*x, *y);
            },
            _ => {},
        }

        if changed {
            self.update_text.update(self.data.clone());
            out_events.push((w.id(), Event {
                name: "changed".to_string(),
                data: EvPayload::Text(self.data.clone()) }));
        }
    }
}
