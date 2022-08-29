// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use markdown::*;

pub struct MarkdownWichtextGenerator {
    header_color_font_size: Vec<(u8, u8)>,
    emphasis_color: u8,
    code_color: u8,
    strong_color: u8,
    normal_color: u8,
    block_width: u16,
    text_lines: Vec<String>,
}

fn word_wrap(indent: u16, width: u16, s: String, out_lines: &mut Vec<String>) {
    let mut indent_s = String::new();
    for _ in 0..indent {
        indent_s += " ";
    }

    let words : Vec<&str> = s.split(" ").collect();

    let mut cur_line = indent_s.clone();
    for word in words.iter() {
        if cur_line.len() > width.into() {
            out_lines.push(cur_line);
            cur_line = indent_s.clone();
        }

        if !cur_line.is_empty() {
            cur_line += " ";
        }
        cur_line += word;
    }

    if cur_line.len() > 0 {
        out_lines.push(cur_line);
    }
}

fn escape(s: &str) -> String {
    s.replace("[", "[[").replace("]", "]]")
}

impl MarkdownWichtextGenerator {
    pub fn new(bw: u16) -> Self {
        Self {
            header_color_font_size: vec![(15, 22), (11, 21), (12, 20), (17, 19)],
            emphasis_color: 2,
            strong_color: 4,
            code_color: 15,
            normal_color: 9,
            block_width: bw,
            text_lines: vec![],
        }
    }

    pub fn header_span2text(&self, s: &Span, size: Option<u8>, color: Option<u8>) -> String {
        match s {
            Span::Break => "".to_string(),
            Span::Link(s, _, _) => {
                let mut fmt = String::new();
                if let Some(size) = size {
                    fmt += &format!("f{}", size);
                }
                if let Some(color) = color {
                    fmt += &format!("c{}", color);
                }
                if fmt.len() > 0 {
                    format!("[a{}:{}]", fmt, escape(s))
                } else {
                    format!("[a:{}]", escape(s))
                }
            }
            Span::Text(s) => {
                let mut fmt = String::new();
                if let Some(size) = size {
                    fmt += &format!("f{}", size);
                }
                if let Some(color) = color {
                    fmt += &format!("c{}", color);
                }
                if fmt.len() > 0 {
                    format!("[{}:{}]", fmt, escape(s))
                } else {
                    escape(s)
                }
            }
            Span::Code(s) => {
                let mut fmt = String::new();
                if let Some(size) = size {
                    fmt += &format!("f{}", size);
                }
                format!("[{}c{}:{}]", fmt, self.code_color, escape(s))
            }
            Span::Strong(spans) => {
                let mut res = String::new();
                for s in spans.iter() {
                    res += &self.header_span2text(s, size, Some(self.strong_color))
                }
                res
            }
            Span::Emphasis(spans) => {
                let mut res = String::new();
                for s in spans.iter() {
                    res += &self.header_span2text(s, size, Some(self.emphasis_color))
                }
                res
            }
            Span::Image(_, _, _) => {
                String::new()
            }
        }
    }

    pub fn append_block(&mut self, b: Block, indent: u16) {
        let width = self.block_width - indent;
        let mut indent_s = String::new();
        for _ in 0..indent {
            indent_s += " ";
        }

        match b {
            Block::Header(spans, size) => {
                let size_idx = size.min(self.header_color_font_size.len() - 1);
                let (color, size) = self.header_color_font_size[size_idx];
                let mut header = String::new();
                for s in spans.iter() {
                    header += &self.header_span2text(s, Some(color), Some(size));
                }
                self.text_lines.push(indent_s + &header);
            },
            Block::Raw(s) => {
                self.text_lines.push(s);
            },
            Block::Hr => {
                let mut dashes = String::from("[t9:{}]");
                for _ in 0..width {
                    dashes += "-";
                }
                self.text_lines.push(indent_s + &dashes);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_mkd2wt_1() {
        let mwg = MarkdownWichtextGenerator::new(10);
    }
}
