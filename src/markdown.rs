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
//            emphasis_color: 2,
//            strong_color: 4,
//            code_color: 15,
//            normal_color: 9,

struct Style {
    add_fmt: Option<String>,
    color: Option<u8>,
    size: Option<u8>,
}

impl Style {
    pub fn new() -> Self {
        Self { add_fmt: None, color: None, size: None }
    }

    pub fn with_code(&self) -> Self {
        Self { add_fmt: self.add_fmt.clone(), color: Some(CODE_COLOR_IDX), size: self.size }
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

        Self { add_fmt: self.add_fmt.clone(), color: Some(color), size: Some(size) }
    }

    pub fn fmt_word(&self, word: &str) -> String {
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

        if fmt.len() > 0 {
            format!("[{}:{}]", fmt, escape(word))
        } else {
            escape(word)
        }
    }
}

struct BlockLayout {
    width: usize,
    cur_line: String,
    cur_line_w: usize,
}

impl BlockLayout {
    pub fn new(width: usize) -> Self {
        Self { width, cur_line: String::new(), cur_line_w: 0 }
    }

    pub fn add_words_from_string(
        &mut self,
        s: &str,
        indent: u16,
        style: &Style,
        out_lines: &mut Vec<String>,
    ) {
        let mut indent_s = String::new();
        for _ in 0..indent {
            indent_s += " ";
        }

        if self.cur_line.is_empty() {
            self.cur_line = indent_s.clone();
            self.cur_line_w = indent_s.len();
        }

        let words: Vec<&str> = s.split(" ").collect();

        for word in words.iter() {
            let word = word.trim();
            if word.is_empty() {
                continue;
            }

            if self.cur_line_w > self.width.into() {
                out_lines.push(self.cur_line.clone());
                self.cur_line = indent_s.clone();
                self.cur_line_w = indent_s.len();
            }

            if !self.cur_line.is_empty() {
                self.cur_line += " ";
                self.cur_line_w += 1;
            }

            self.cur_line += &style.fmt_word(word);
            self.cur_line_w += word.len();
        }
    }

    pub fn flush(&mut self, out_lines: &mut Vec<String>) {
        if self.cur_line_w > 0 {
            out_lines.push(self.cur_line.clone());
            self.cur_line = String::new();
            self.cur_line_w = 0;
        }
    }
}

//fn word_wrap(indent: u16, width: u16, s: String, out_lines: &mut Vec<String>) {
//    let mut indent_s = String::new();
//    for _ in 0..indent {
//        indent_s += " ";
//    }
//
//    let words : Vec<&str> = s.split(" ").collect();
//
//    let mut cur_line = indent_s.clone();
//    for word in words.iter() {
//        if cur_line.len() > width.into() {
//            out_lines.push(cur_line);
//            cur_line = indent_s.clone();
//        }
//
//        if !cur_line.is_empty() {
//            cur_line += " ";
//        }
//        cur_line += word;
//    }
//
//    if cur_line.len() > 0 {
//        out_lines.push(cur_line);
//    }
//}

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

    pub fn parse(&mut self, txt: &str) {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(txt, options);

        let mut layout = BlockLayout::new(self.block_width.into());

        let mut style_stack = vec![Style::new()];

        let mut indent: u16 = 0;

        for ev in parser {
            println!("EVENT: {:?}", ev);

            match ev {
                Event::Rule => {
                    let mut indent_s = String::new();
                    for _ in 0..indent {
                        indent_s += " ";
                    }
                    let mut dashes = String::from("[t9]");
                    for _ in 0..self.block_width {
                        dashes += "-";
                    }
                    self.text_lines.push(indent_s + &dashes);
                }
                Event::Start(tag) => match tag {
                    Tag::CodeBlock(_) => {
                        indent += 4;
                        style_stack.push(style_stack.last().unwrap().with_code());
                        layout.flush(&mut self.text_lines);
                        self.text_lines.push(String::new());
                    }
                    Tag::Heading(hl, _, _) => {
                        layout.flush(&mut self.text_lines);
                        self.text_lines.push(String::new());

                        style_stack.push(
                            style_stack
                                .last()
                                .unwrap()
                                .with_heading_level(hl, &self.header_color_font_size),
                        );
                    }
                    _ => {}
                },
                Event::End(tag) => match tag {
                    Tag::CodeBlock(_) => {
                        indent -= 4;
                        style_stack.pop();
                        layout.flush(&mut self.text_lines);
                        self.text_lines.push(String::new());
                    }
                    Tag::Heading(_, _, _) => {
                        style_stack.pop();
                        layout.flush(&mut self.text_lines);
                        self.text_lines.push(String::new());
                    }
                    Tag::Paragraph => {
                        layout.flush(&mut self.text_lines);
                        self.text_lines.push(String::new());
                    }
                    _ => {}
                },
                Event::Code(s) => {
                    style_stack.push(style_stack.last().unwrap().with_code());
                    layout.add_words_from_string(
                        &s,
                        indent,
                        style_stack.last().unwrap(),
                        &mut self.text_lines,
                    );
                    style_stack.pop();
                }
                Event::Text(s) => {
                    layout.add_words_from_string(
                        &s,
                        indent,
                        style_stack.last().unwrap(),
                        &mut self.text_lines,
                    );
                }
                _ => {}
            }
        }
    }

    //    pub fn header_span2text(&self, s: &Span, size: Option<u8>, color: Option<u8>) -> String {
    //        match s {
    //            Span::Break => "".to_string(),
    //            Span::Link(s, _, _) => {
    //                let mut fmt = String::new();
    //                if let Some(size) = size {
    //                    fmt += &format!("f{}", size);
    //                }
    //                if let Some(color) = color {
    //                    fmt += &format!("c{}", color);
    //                }
    //                if fmt.len() > 0 {
    //                    format!("[a{}:{}]", fmt, escape(s))
    //                } else {
    //                    format!("[a:{}]", escape(s))
    //                }
    //            }
    //            Span::Text(s) => {
    //                let mut fmt = String::new();
    //                if let Some(size) = size {
    //                    fmt += &format!("f{}", size);
    //                }
    //                if let Some(color) = color {
    //                    fmt += &format!("c{}", color);
    //                }
    //                if fmt.len() > 0 {
    //                    format!("[{}:{}]", fmt, escape(s))
    //                } else {
    //                    escape(s)
    //                }
    //            }
    //            Span::Code(s) => {
    //                let mut fmt = String::new();
    //                if let Some(size) = size {
    //                    fmt += &format!("f{}", size);
    //                }
    //                format!("[{}c{}:{}]", fmt, self.code_color, escape(s))
    //            }
    //            Span::Strong(spans) => {
    //                let mut res = String::new();
    //                for s in spans.iter() {
    //                    res += &self.header_span2text(s, size, Some(self.strong_color))
    //                }
    //                res
    //            }
    //            Span::Emphasis(spans) => {
    //                let mut res = String::new();
    //                for s in spans.iter() {
    //                    res += &self.header_span2text(s, size, Some(self.emphasis_color))
    //                }
    //                res
    //            }
    //            Span::Image(_, _, _) => {
    //                String::new()
    //            }
    //        }
    //    }
    //
    //    pub fn append_block(&mut self, b: &Block, indent: u16) {
    //        let width = self.block_width - indent;
    //        let mut indent_s = String::new();
    //        for _ in 0..indent {
    //            indent_s += " ";
    //        }
    //        println!("BLCOK: {:?}", b);
    //
    //        match b {
    //            Block::Header(spans, size) => {
    //                let size_idx = (*size).min(self.header_color_font_size.len() - 1);
    //                let (color, size) = self.header_color_font_size[size_idx];
    //                let mut header = String::new();
    //                for s in spans.iter() {
    //                    header += &self.header_span2text(s, Some(size), Some(color));
    //                }
    //                self.text_lines.push(indent_s + &header);
    //            },
    //            Block::Raw(s) => {
    //                self.text_lines.push(indent_s + s);
    //            },
    //            Block::Hr => {
    //                println!("HR {}\n", width);
    //                let mut dashes = String::from("[t9]");
    //                for _ in 0..width {
    //                    dashes += "-";
    //                }
    //                self.text_lines.push(indent_s + &dashes);
    //            }
    //            _ => {}
    //        }
    //    }

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

### And here: [Foobar](Barfoo)

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
        "#;
        //        for block in tree.iter() {
        //            mwg.append_block(block, 0);
        //        }
        mwg.parse(text);
        println!("RES:\n{}", mwg.to_string());

        assert_eq!(mwg.to_string(), "FFF");
    }
}
