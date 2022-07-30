// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use hexotk::*;
use hexodsp::blocklang::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

const WINDOW_W: i32 = 1020;
const WINDOW_H: i32 = 720;

#[derive(Debug)]
pub struct ASTNode {
    pub id:    usize,
    pub typ:   String,
    pub lbl:   String,
    pub nodes: Vec<(String, String, ASTNodeRef)>,
}

#[derive(Debug, Clone)]
pub struct ASTNodeRef(Rc<RefCell<ASTNode>>);

impl ASTNodeRef {
    pub fn walk_dump(&self, input: &str, output: &str, indent: usize) -> String {
        let indent_str = "   ".repeat(indent + 1);

        let out_port =
            if output.len() > 0 { format!("(out: {})", output) }
            else { "".to_string() };
        let in_port =
            if input.len() > 0 { format!("(in: {})", input) }
            else { "".to_string() };

        let mut s = format!(
            "{}{}#{}[{}] {}{}\n",
            indent_str, self.0.borrow().id, self.0.borrow().typ,
            self.0.borrow().lbl, out_port, in_port);

        for (inp, out, n) in &self.0.borrow().nodes {
            s += &n.walk_dump(&inp, &out, indent + 1);
        }

        s
    }
}

impl BlockASTNode for ASTNodeRef {
    fn from(id: usize, typ: &str, lbl: &str) -> ASTNodeRef {
        ASTNodeRef(Rc::new(RefCell::new(ASTNode {
            id,
            typ:    typ.to_string(),
            lbl:    lbl.to_string(),
            nodes:  vec![],
        })))
    }

    fn add_node(&self, in_port: String, out_port: String, node: ASTNodeRef) {
        self.0.borrow_mut().nodes.push((in_port, out_port, node));
    }
}

fn make_block_lang() -> Rc<RefCell<BlockLanguage>> {
    let mut lang = BlockLanguage::new();

    lang.define(BlockType {
        category: "source".to_string(),
        name: "phse".to_string(),
        rows: 1,
        inputs: vec![Some("f".to_string())],
        outputs: vec![Some("".to_string())],
        area_count: 0,
        user_input: BlockUserInput::None,
        description: "A phasor, returns a saw tooth wave to scan through things or use as modulator.".to_string(),
        color: 2,
    });

    lang.define(BlockType {
        category:       "literals".to_string(),
        name:           "zero".to_string(),
        rows:           1,
        inputs:         vec![],
        outputs:        vec![Some("".to_string())],
        area_count:     0,
        user_input:     BlockUserInput::None,
        description:    "The 0.0 value".to_string(),
        color:          1,
    });

    lang.define(BlockType {
        category:       "routing".to_string(),
        name:           "->".to_string(),
        rows:           1,
        inputs:         vec![Some("".to_string())],
        outputs:        vec![Some("".to_string())],
        area_count:     0,
        user_input:     BlockUserInput::None,
        description:    "Forwards the value one block".to_string(),
        color:          6,
    });

    lang.define(BlockType {
        category:       "routing".to_string(),
        name:           "->2".to_string(),
        rows:           2,
        inputs:         vec![Some("".to_string())],
        outputs:        vec![Some("".to_string()), Some("".to_string())],
        area_count:     0,
        user_input:     BlockUserInput::None,
        description:    "Forwards the value one block and sends it to multiple destinations".to_string(),
        color:          6,
    });

    lang.define(BlockType {
        category:       "routing".to_string(),
        name:           "->3".to_string(),
        rows:           3,
        inputs:         vec![Some("".to_string())],
        outputs:        vec![Some("".to_string()), Some("".to_string()), Some("".to_string())],
        area_count:     0,
        user_input:     BlockUserInput::None,
        description:    "Forwards the value one block and sends it to multiple destinations".to_string(),
        color:          6,
    });

    lang.define(BlockType {
        category:       "variables".to_string(),
        name:           "set".to_string(),
        rows:           1,
        inputs:         vec![Some("".to_string())],
        outputs:        vec![],
        area_count:     0,
        user_input:     BlockUserInput::Identifier,
        description:    "Stores into a variable".to_string(),
        color:          2,
    });

    lang.define(BlockType {
        category:       "variables".to_string(),
        name:           "get".to_string(),
        rows:           1,
        inputs:         vec![],
        outputs:        vec![Some("".to_string())],
        area_count:     0,
        user_input:     BlockUserInput::Identifier,
        description:    "Loads a variable".to_string(),
        color:          12,
    });

    lang.define(BlockType {
        category:       "variables".to_string(),
        name:           "if".to_string(),
        rows:           1,
        inputs:         vec![Some("".to_string())],
        outputs:        vec![Some("".to_string())],
        area_count:     2,
        user_input:     BlockUserInput::None,
        description:    "Divides the controlflow based on a true (>= 0.5) \
                         or false (< 0.5) input value.".to_string(),
        color:          0,
    });

    lang.define(BlockType {
        category:       "nodes".to_string(),
        name:           "1pole".to_string(),
        rows:           2,
        inputs:         vec![Some("in".to_string()), Some("f".to_string())],
        outputs:        vec![Some("lp".to_string()), Some("hp".to_string())],
        area_count:     0,
        user_input:     BlockUserInput::None,
        description:    "Runs a simple one pole filter on the input".to_string(),
        color:          8,
    });

    lang.define(BlockType {
        category:       "nodes".to_string(),
        name:           "svf".to_string(),
        rows:           3,
        inputs:         vec![Some("in".to_string()), Some("f".to_string()), Some("r".to_string())],
        outputs:        vec![Some("lp".to_string()), Some("bp".to_string()), Some("hp".to_string())],
        area_count:     0,
        user_input:     BlockUserInput::None,
        description:    "Runs a state variable filter on the input".to_string(),
        color:          8,
    });

    lang.define(BlockType {
        category:       "functions".to_string(),
        name:           "sin".to_string(),
        rows:           1,
        inputs:         vec![Some("".to_string())],
        outputs:        vec![Some("".to_string())],
        area_count:     0,
        user_input:     BlockUserInput::None,
        description:    "Calculates the sine of the input".to_string(),
        color:          16,
    });

    lang.define(BlockType {
        category:       "nodes".to_string(),
        name:           "delay".to_string(),
        rows:           2,
        inputs:         vec![Some("in".to_string()), Some("t".to_string())],
        outputs:        vec![Some("".to_string())],
        area_count:     0,
        user_input:     BlockUserInput::None,
        description:    "Runs a linearly interpolated delay on the input".to_string(),
        color:          8,
    });

    for fun_name in &["+", "-", "*", "/"] {
        lang.define(BlockType {
            category:       "arithmetics".to_string(),
            name:           fun_name.to_string(),
            rows:           2,
            inputs:
                if fun_name == &"-" || fun_name == &"/" {
                    vec![Some("a".to_string()), Some("b".to_string())]
                } else {
                    vec![Some("".to_string()), Some("".to_string())]
                },
            outputs:        vec![Some("".to_string())],
            area_count:     0,
            user_input:     BlockUserInput::None,
            description:    "A binary arithmetics operation".to_string(),
            color:          4,
        });
    }

    lang.define_identifier("alpha");
    lang.define_identifier("beta");
    lang.define_identifier("delta");
    lang.define_identifier("gamma");
    lang.define_identifier("&sig1");
    lang.define_identifier("&sig2");

    Rc::new(RefCell::new(lang))
}

fn main() {
    let concurrent_data = Arc::new(Mutex::new(CloneMutable::new(("Count:".to_string(), 0))));

    std::thread::spawn({
        let data = concurrent_data.clone();
        move || loop {
            if let Ok(mut data) = data.lock() {
                (*data).1 += 1;
            }
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    });

    open_window(
        "HexoTK BlockCode Test",
        WINDOW_W,
        WINDOW_H,
        None,
        Box::new(|| {
            let mut style = Style::new();
            style.font_size = 20.0;

            let style_ref = Rc::new(style.clone());

            let lang = make_block_lang();

            let block_fun = Rc::new(RefCell::new(BlockFun::new(lang)));

            block_fun.borrow_mut().instanciate_at(0, 3, 3, "phse", None).unwrap();

            let s = style_ref.with_style_clone(|style| {
                style.bg_color = hexotk::style::UI_ACCENT_BG1_CLR;
                style.border_style = BorderStyle::Rect;
                style.pad_top = 0.0;
                style.pad_left = 0.0;
                style.pad_right = 0.0;
                style.pad_bottom = 0.0;
                style.border = 2.0;
                style.shadow_offs = (0.0, 0.0);
                style.border_color = hexotk::style::UI_SELECT_CLR;
                style.font_size = 10.0;
                style.ext = StyleExt::BlockCode {
                    with_markers: true,
                    grid_marker_color: hexotk::style::UI_ACCENT_DARK_CLR,
                    block_bg_hover_color: hexotk::style::UI_ACCENT_CLR,
                    block_bg_color: hexotk::style::UI_ACCENT_BG2_CLR,
                    port_select_color: hexotk::style::UI_SELECT_CLR,
                };
            });

            let root = Widget::new(style_ref.clone());
            root.set_ctrl(Control::Rect);
            root.enable_cache();
            root.change_layout(|l| l.layout_type = Some(LayoutType::Column));

            let blockcode = Widget::new(s.clone());
            blockcode.set_ctrl(Control::BlockCode { code: Box::new(BlockCode::new(block_fun.clone())) });
            block_fun.borrow_mut().instanciate_at(0, 5, 5, "phse", None).unwrap();

            let code = block_fun.clone();
            blockcode.reg("click", move |_ctx, _wid, ev| {
                if let EvPayload::BlockPos { button, at, .. } = ev.data {
                    if let BlockPos::Block { row, col, .. } = at {
                        let (id, x, y) = at.pos();

                        if button == MButton::Right {
                            println!("PORT CLICK {:?}", at);
                            code.borrow_mut()
                                .shift_port(id, x, y, row, col == 1);
                        } else {
                            if col == 1 {
                                let _ = code.borrow_mut()
                                    .split_block_chain_after(
                                        id, x, y, Some("->"));
                            } else {
                                let _ = code.borrow_mut()
                                    .split_block_chain_after(
                                        id, x - 1, y, None);
                            }
                        }

                        let tree = code.borrow().generate_tree::<ASTNodeRef>("zero").unwrap();
                        println!("{}", tree.walk_dump("", "", 0));

                        code.borrow_mut()
                            .recalculate_area_sizes();
                    } else {
                        println!("CLICK POPUP {:?}", at);
    //                    state.insert_event(
    //                        Event::new(PopupEvent::OpenAtCursor)
    //                        .target(pop)
    //                        .origin(Entity::root()));
                    }
//                    (*on_change)(state, entity, code.clone());
                }
                println!("CLICK: {:?}", ev);
            });

            let code = block_fun.clone();
            blockcode.reg("drag", move |_ctx, _wid, ev| {
                if let EvPayload::BlockPos { at, to: Some(to), button } = ev.data {
                    println!("CLICK: {:?}", ev);
                    let (id, x, y)    = at.pos();
                    let (id2, x2, y2) = to.pos();

                    println!("P1={:?} P2={:?}", at, to);

                    if let BlockPos::Cell { .. } = at {
                        if let BlockPos::Block { .. } = to {
                            let _ = code.borrow_mut()
                                .clone_block_from_to(
                                    id2, x2, y2, id, x, y);
                            code.borrow_mut()
                                .recalculate_area_sizes();

                            // (*ouagen_change)(state, entity, code.clone());
                        }
                    } else {
                        if button == MButton::Right {
                            let _ = code.borrow_mut()
                                .move_block_from_to(
                                    id, x, y, id2, x2, y2);
                        } else {
                            if at.pos() == to.pos() {
                                let _ = code.borrow_mut()
                                    .remove_at(id, x, y);
                            } else {
                                let _ = code.borrow_mut()
                                    .move_block_chain_from_to(
                                        id, x, y, id2, x2, y2);
                            }
                        }

                        code.borrow_mut()
                            .recalculate_area_sizes();

                        // (*on_change)(state, entity, code.clone());
                    }
                }
            });

            root.add(blockcode);

            let mut ui = Box::new(UI::new(Rc::new(RefCell::new(1))));

            ui.add_layer_root(root);

            ui
        }),
    );
}
