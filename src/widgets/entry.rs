// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::{Widget, InputEvent, Event, MButton, EvPayload, Style};
use keyboard_types::{KeyboardEvent, Key};

use crate::painter::*;
use crate::rect::*;

pub struct Entry {
    data:        String,
    pre_string:  String,
    post_string: String,
    cursor:      usize,
    mouse_pos:   (f32, f32),
}

impl Entry {
    pub fn new() -> Self {
        Self {
            data:        String::from("Test 123"),
            pre_string:  String::from("Test"),
            post_string: String::from(" 123"),
            cursor:      4,
            mouse_pos:   (0.0, 0.0),
        }
    }

    pub fn check_change(&mut self) -> bool {
        false
    }
    
    fn update_cursor(&mut self) {
        self.pre_string  = self.data.chars().take(self.cursor).collect();
        self.post_string = self.data.chars().skip(self.cursor).collect();
    }

    pub fn draw(&mut self, w: &Widget, style: &Style, pos: Rect, p: &mut Painter) {
        let is_hovered = w.is_hovered();
        let is_active  = w.is_active();

        let fh          = p.font_height(style.font_size, true);
        let cur_start_x = p.text_width(style.font_size, true, &self.pre_string[..]);

        let color =
            if is_active        { style.active_color }
            else if is_hovered  { style.hover_color }
            else                { style.color };

        p.label_mono(
            style.font_size, -1, style.color,
            pos.x, pos.y, pos.w, fh,
            &self.data);

        p.stroke(
            1.0, color, &[
                ((pos.x + cur_start_x).round() + 0.5, pos.y),
                ((pos.x + cur_start_x).round() + 0.5, pos.y + fh),
            ], false);
    }

    pub fn handle(
        &mut self, w: &Widget, event: &InputEvent, out_events: &mut Vec<(usize, Event)>)
    {
        let is_hovered = w.is_hovered();
        let is_active  = w.is_active();

//      out_events.push((w.id(), Event {
//          name: "click".to_string(),
//          data: EvPayload::WichTextCommand {
//              line, frag, cmd,
//          },
//      }));
        println!("EV: {:?}", event);


        match event {
            InputEvent::KeyPressed(key) => {
                let len = self.data.chars().count();

                if !is_active { return; }

                match &key.key {
                    Key::Character(s) => {
                        self.data =
                            self.pre_string.clone() + &s + &self.post_string;
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
                let (x, y) = self.mouse_pos;

                if is_hovered {
                    if *btn == MButton::Left {
                        w.activate();
                        w.emit_redraw_required();
                    }
                }
            },
            InputEvent::MouseButtonReleased(btn) => {
                if w.is_active() {
                    let (x, y) = self.mouse_pos;

                    if *btn == MButton::Left {
//                        w.deactivate();
//                        w.emit_redraw_required();
                    }
                }
            },
            InputEvent::MousePosition(x, y) => {
                self.mouse_pos = (*x, *y);
            },
            _ => {},
        }
    }
}
