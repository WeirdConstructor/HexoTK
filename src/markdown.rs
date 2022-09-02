// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag};

pub struct MarkdownWichtextGenerator {
    header_color_font_size: Vec<(u8, u8)>,
    emphasis_color: u8,
    code_color: u8,
    strong_color: u8,
    normal_color: u8,
    block_width: u16,
    text_lines: Vec<String>,
}

fn escape(s: &str) -> String {
    s.replace("[", "[[").replace("]", "]]")
}

const CODE_COLOR_IDX: u8 = 15;
const LINK_COLOR_IDX: u8 = 8;
const STRONG_COLOR_IDX: u8 = 4;
const EMPHASIS_COLOR_IDX: u8 = 2;
const STRIKE_COLOR_IDX: u8 = 11;
const STRIKE_SIZE: u8 = 12;
const LIST_MARK_COLOR_IDX: u8 = 17;
//            emphasis_color: 2,
//            strong_color: 4,
//            code_color: 15,
//            normal_color: 9,

struct Style {
    add_fmt: Option<String>,
    color: Option<u8>,
    size: Option<u8>,
    raw: bool,
}

impl Style {
    pub fn new() -> Self {
        Self { add_fmt: None, color: None, size: None, raw: false }
    }

    pub fn with_list_mark(&self) -> Self {
        Self {
            add_fmt: self.add_fmt.clone(),
            color: Some(LIST_MARK_COLOR_IDX),
            size: self.size,
            raw: self.raw,
        }
    }

    pub fn with_strong(&self) -> Self {
        Self {
            add_fmt: self.add_fmt.clone(),
            color: Some(STRONG_COLOR_IDX),
            size: self.size,
            raw: self.raw,
        }
    }

    pub fn with_emphasis(&self) -> Self {
        Self {
            add_fmt: self.add_fmt.clone(),
            color: Some(EMPHASIS_COLOR_IDX),
            size: self.size,
            raw: self.raw,
        }
    }

    pub fn with_strike(&self) -> Self {
        Self {
            add_fmt: self.add_fmt.clone(),
            color: Some(STRIKE_COLOR_IDX),
            size: Some(STRIKE_SIZE),
            raw: self.raw,
        }
    }

    pub fn with_code(&self) -> Self {
        Self {
            add_fmt: self.add_fmt.clone(),
            color: Some(CODE_COLOR_IDX),
            size: self.size,
            raw: self.raw,
        }
    }

    pub fn with_link(&self, lref: &str) -> Self {
        let mut add_fmt =
            if let Some(add_fmt) = &self.add_fmt { add_fmt.to_string() } else { String::new() };
        add_fmt += "a";
        Self {
            add_fmt: Some(add_fmt),
            color: Some(LINK_COLOR_IDX),
            size: self.size,
            raw: lref == "$",
        }
    }

    pub fn with_heading_level(&self, hl: HeadingLevel, heading_styles: &[(u8, u8)]) -> Self {
        let index = match hl {
            HeadingLevel::H1 => 0,
            HeadingLevel::H2 => 1,
            HeadingLevel::H3 => 2,
            HeadingLevel::H4 => 3,
            HeadingLevel::H5 => 4,
            HeadingLevel::H6 => 5,
        };

        let size_idx = index.min(heading_styles.len() - 1);
        let (color, size) = heading_styles[size_idx];

        Self { add_fmt: self.add_fmt.clone(), color: Some(color), size: Some(size), raw: self.raw }
    }

    pub fn fmt_word(&self, word: &str) -> String {
        if self.raw {
            return word.to_string();
        }
        if word.is_empty() {
            return String::new();
        }

        let mut fmt = String::new();

        if let Some(size) = self.size {
            fmt += &format!("f{}", size);
        }
        if let Some(color) = self.color {
            fmt += &format!("c{}", color);
        }
        if let Some(add_fmt) = &self.add_fmt {
            fmt += add_fmt;
        }

        let word = escape(word);
        if fmt.len() > 0 {
            format!("[{}:{}]", fmt, word)
        } else {
            word
        }
    }
}

fn indent_str(indent: u16) -> String {
    let mut indent_s = String::new();
    for _ in 0..indent {
        indent_s += " ";
    }
    indent_s
}

struct BlockLayout {
    indent: u16,
    indent_stack: Vec<u16>,
    first_without_indent: bool,
    width: usize,
    cur_line: String,
    cur_line_w: usize,
}

impl BlockLayout {
    pub fn new(width: usize) -> Self {
        Self {
            indent: 0,
            indent_stack: vec![],
            first_without_indent: false,
            width,
            cur_line: String::new(),
            cur_line_w: 0,
        }
    }

    pub fn push_indent(&mut self, inc: u16) {
        self.indent_stack.push(self.indent);
        self.indent += inc;
    }

    pub fn pop_indent(&mut self) {
        self.indent = self.indent_stack.pop().unwrap_or(0);
    }

    pub fn set_first_without_indent(&mut self) {
        self.first_without_indent = true;
    }

    pub fn force_space(&mut self) {
        self.cur_line += " ";
        self.cur_line_w += 1;
    }

    pub fn add_words_from_string(&mut self, s: &str, style: &Style, out_lines: &mut Vec<String>) {
        let indent_s = if self.first_without_indent {
            self.first_without_indent = false;
            indent_str(self.indent_stack.last().copied().unwrap_or(0))
        } else {
            indent_str(self.indent)
        };

        if self.cur_line.is_empty() {
            self.cur_line = indent_s.clone();
            self.cur_line_w = indent_s.len();
        }

        let words: Vec<&str> = s.split(" ").collect();

        let mut started_block = true;

        for word in words.iter() {
            let word = word.trim();

            if self.cur_line_w > self.width.into() {
                out_lines.push(self.cur_line.clone());
                self.cur_line = indent_s.clone();
                self.cur_line_w = indent_s.len();
            }

            if !started_block && self.cur_line.find(|c| !char::is_whitespace(c)).is_some() {
                self.cur_line += " ";
                self.cur_line_w += 1;
            }

            self.cur_line += &style.fmt_word(word);
            self.cur_line_w += word.len();

            started_block = false;
        }
    }

    pub fn flush(&mut self, out_lines: &mut Vec<String>) {
        if self.cur_line_w > 0 {
            out_lines.push(self.cur_line.clone());
            self.cur_line = String::new();
            self.cur_line_w = 0;
            self.first_without_indent = false;
        }
    }
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

    fn ensure_empty_line(&mut self) {
        let prev_empty = if let Some(l) = self.text_lines.last() { l.is_empty() } else { true };
        if !prev_empty {
            self.text_lines.push(String::new());
        }
    }

    pub fn parse(&mut self, txt: &str) {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(txt, options);

        let mut layout = BlockLayout::new(self.block_width.into());

        let mut style_stack = vec![Style::new()];

        let mut list_stack = vec![];
        let mut current_list_index = None;

        for ev in parser {
            println!("EVENT: {:?}", ev);

            match ev {
                Event::Rule => {
                    let indent_s = indent_str(layout.indent);
                    let mut dashes = String::from("[c11:");
                    for _ in 0..self.block_width {
                        dashes += "-";
                    }
                    dashes += "]";
                    self.ensure_empty_line();
                    self.text_lines.push(indent_s + &dashes);
                }
                Event::Start(tag) => match tag {
                    Tag::List(start) => {
                        list_stack.push(current_list_index);
                        current_list_index = start;
                        layout.flush(&mut self.text_lines);
                    }
                    Tag::Item => {
                        layout.flush(&mut self.text_lines);
                        let item = if let Some(index) = &mut current_list_index {
                            *index += 1;
                            format!("{}", *index - 1)
                        } else {
                            "*".to_string()
                        };
                        layout.add_words_from_string(
                            &item,
                            &style_stack.last().unwrap().with_list_mark(),
                            &mut self.text_lines,
                        );
                        layout.force_space();
                        layout.push_indent(2);
                    }
                    Tag::Image(_, lref, _) => self.text_lines.push(format!("[I{}:]", lref)),
                    Tag::Link(_, lref, _) => {
                        style_stack.push(style_stack.last().unwrap().with_link(&lref));
                    }
                    Tag::CodeBlock(_) => {
                        style_stack.push(style_stack.last().unwrap().with_code());
                        layout.flush(&mut self.text_lines);
                        layout.push_indent(4);
                        self.ensure_empty_line();
                    }
                    Tag::Heading(hl, _, _) => {
                        layout.flush(&mut self.text_lines);
                        self.ensure_empty_line();

                        style_stack.push(
                            style_stack
                                .last()
                                .unwrap()
                                .with_heading_level(hl, &self.header_color_font_size),
                        );
                    }
                    Tag::Strong => {
                        style_stack.push(style_stack.last().unwrap().with_strong());
                    }
                    Tag::Emphasis => {
                        style_stack.push(style_stack.last().unwrap().with_emphasis());
                    }
                    Tag::Strikethrough => {
                        style_stack.push(style_stack.last().unwrap().with_strike());
                    }
                    _ => {}
                },
                Event::End(tag) => match tag {
                    Tag::CodeBlock(_) => {
                        style_stack.pop();
                        layout.flush(&mut self.text_lines);
                        layout.pop_indent();
                        self.ensure_empty_line();
                    }
                    Tag::List(_) => {
                        current_list_index = list_stack.pop().flatten();
                    }
                    Tag::Image(_, _, _) => {}
                    Tag::Link(_, _, _) => {
                        style_stack.pop();
                    }
                    Tag::Item => {
                        layout.flush(&mut self.text_lines);
                        layout.pop_indent();
                    }
                    Tag::Heading(_, _, _) => {
                        style_stack.pop();
                        layout.flush(&mut self.text_lines);
                        self.ensure_empty_line();
                    }
                    Tag::Paragraph => {
                        layout.flush(&mut self.text_lines);
                        self.ensure_empty_line();
                    }
                    Tag::Strong => {
                        style_stack.pop();
                    }
                    Tag::Emphasis => {
                        style_stack.pop();
                    }
                    Tag::Strikethrough => {
                        style_stack.pop();
                    }
                    _ => {}
                },
                Event::Code(s) => {
                    style_stack.push(style_stack.last().unwrap().with_code());
                    layout.add_words_from_string(
                        &s,
                        style_stack.last().unwrap(),
                        &mut self.text_lines,
                    );
                    style_stack.pop();
                }
                Event::Text(s) => {
                    layout.add_words_from_string(
                        &s,
                        style_stack.last().unwrap(),
                        &mut self.text_lines,
                    );
                }
                _ => {}
            }
        }
    }

    pub fn to_string(&self) -> String {
        self.text_lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_mkd2wt_1() {
        let mut mwg = MarkdownWichtextGenerator::new(20);
        let text = r#"


fpoiewj fewoifj ewoifeowj 
f weiofj eiwoofj wejfwe
feiowjfeiowfeiowfwiofew

===================

feiofj wiofjwowe
f weoifewj ioewj fweo feiwjfewoi

-------

Test
====

## Test 123 *foobar* __LOL__ ´323423423´

AAA `cccc` DDDD

-------

BBBBB

### And here: [Foobar]()

Test 123 fiuowe fieuwf hewuif hewiuf weiuf hweifu wehfi uwehf iweufh ewiuf hweiuf weiuf weiuf
Test 123 fiuowe fieuwf hewuif hewiuf weiuf hweifu wehfi uwehf iweufh ewiuf hweiuf weiuf weiuf
Test 123 _fiuowe fieuwf hewuif hewiuf_ weiuf hweifu wehfi uwehf iweufh ewiuf hweiuf weiuf weiuf
Test 123 fiuowe fieuwf hewuif hewiuf weiuf hweifu wehfi uwehf iweufh ewiuf hweiuf weiuf weiuf
Test 123 fiuowe fieuwf hewuif hewiuf weiuf hweifu wehfi uwehf iweufh ewiuf hweiuf weiuf weiuf

1. List Itmee 1
2. List Item 2
2. List Item
foieuwj fewo fejwiof ewjfioe wiofj weoif iofwe
foieuwj fewo fejwiof ewjfioe wiofj weoif iofwe
foieuwj fewo fejwiof ewjfioe wiofj weoif iofwe
foieuwj fewo fejwiof ewjfioe wiofj weoif iofwe
    ```text
    Test 123 892u 923u 2389r 2389rj 98ew
    Test 123 892u 923u 2389r 2389rj 98ew
    Test 123 892u 923u 2389r 2389rj 98ew
    ```
3. Foobar lololol
  * Item A
  * Item B
  * Item C
  * Item D

Intdent start:

    fiif ewoif ejwoifw joiwej foiwef jeowiefwoi fjiowe
    fiif ewoif ejwoifw joiwej foiwef jeowiefwoi fjiowe
    fiif ewoif ejwoifw joiwej foiwef jeowiefwoi fjiowe
    fiif ewoif ejwoifw joiwej foiwef jeowiefwoi fjiowe
    fiif ewoif ejwoifw joiwej foiwef jeowiefwoi fjiowe

eindent end

---------------------------------------
```
COde blcapfcelfw
feiow fewf
```
- Other A
- Other B
- Other C
- Other D
- Other E

> ## Bla bla
> feoiwfjew ofew
> feoiwfjew ofew
> feoiwfjew ofew
> feoiwfjew ofew

[[c15f30:Foobar] Lol]($)

Image here: ![](main/bla.png)

**Strong text here lo lof efew jofiewj oiewfjoi we**

*Emphasis text hefew ewfhweiu fhweiu hewiuf ewiufhew*

~~Strike~~
        "#;
        //        for block in tree.iter() {
        //            mwg.append_block(block, 0);
        //        }
        mwg.parse(text);
        println!("RES:\n{}", mwg.to_string());
    }

    #[test]
    fn check_mkd2wt_para() {
        let mut mwg = MarkdownWichtextGenerator::new(50);
        mwg.parse("A\n\nC\n");
        assert_eq!(mwg.to_string(), "A\n\nC\n");
    }

    #[test]
    fn check_mkd2wt_lists() {
        let mut mwg = MarkdownWichtextGenerator::new(50);
        mwg.parse("- A\n  - B\n- C\n");
        println!("RES:\n{}", mwg.to_string());
        assert_eq!(mwg.to_string(), "[c17:*] A\n  [c17:*] B\n[c17:*] C");

        let mut mwg = MarkdownWichtextGenerator::new(50);
        mwg.parse("- A\n  - B\n\n- C\n");
        println!("RES:\n{}", mwg.to_string());
        assert_eq!(mwg.to_string(), "[c17:*] A\n\n  [c17:*] B\n[c17:*] C\n");
    }

    #[test]
    fn check_mkd2wt_emph() {
        let mut mwg = MarkdownWichtextGenerator::new(50);
        mwg.parse("A*B*C");
        println!("RES:\n{}", mwg.to_string());
        assert_eq!(mwg.to_string(), "A[c2:B]C\n");

        let mut mwg = MarkdownWichtextGenerator::new(50);
        mwg.parse("A<B@fo.de>C");
        println!("RES:\n{}", mwg.to_string());
        assert_eq!(mwg.to_string(), "A[c8a:B@fo.de]C\n");

        let mut mwg = MarkdownWichtextGenerator::new(50);
        mwg.parse("A <B@fo.de> C");
        println!("RES:\n{}", mwg.to_string());
        assert_eq!(mwg.to_string(), "A [c8a:B@fo.de] C\n");
    }
}
