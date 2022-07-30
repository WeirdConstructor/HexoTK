use hexotk::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

const WINDOW_W: i32 = 1020;
const WINDOW_H: i32 = 720;

fn make_block_lang() -> Rc<RefCell<BlockLanguage>> {
    let mut lang = BlockLanguage::new();

    lang.define(BlockType {
        category: "source".to_string(),
        name: "phasor".to_string(),
        rows: 1,
        inputs: vec![Some("freq".to_string())],
        outputs: vec![Some("s".to_string())],
        area_count: 0,
        user_input: BlockUserInput::None,
        description: "A phasor, returns a saw tooth wave to scan through things or use as modulator.".to_string(),
        color: 2,
    });

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

            block_fun.borrow_mut().instanciate_at(0, 3, 3, "phasor", None).unwrap();

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
            });

            let root = Widget::new(style_ref.clone());
            root.set_ctrl(Control::Rect);
            root.enable_cache();
            root.change_layout(|l| l.layout_type = Some(LayoutType::Column));

            let blockcode = Widget::new(s.clone());
            blockcode.set_ctrl(Control::BlockCode { code: Box::new(BlockCode::new(block_fun.clone())) });
            block_fun.borrow_mut().instanciate_at(0, 5, 5, "phasor", None).unwrap();

            root.add(blockcode);

            let mut ui = Box::new(UI::new(Rc::new(RefCell::new(1))));

            ui.add_layer_root(root);

            ui
        }),
    );
}
