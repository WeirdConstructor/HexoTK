use hexotk::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

const WINDOW_W: i32 = 1020;
const WINDOW_H: i32 = 720;

fn main() {
    open_window(
        "HexoTK List Test",
        WINDOW_W,
        WINDOW_H,
        None,
        Box::new(|| {
            let mut style = Style::new();
            style.font_size = 20.0;

            let style_ref = Rc::new(style.clone());

            let s = style_ref.with_style_clone(|style| {
                style.bg_color = hexotk::style::UI_ACCENT_BG1_CLR;
                style.border_style = BorderStyle::Rect;
                style.pad_top = 0.0;
                style.pad_left = 0.0;
                style.pad_right = 0.0;
                style.pad_bottom = 0.0;
                style.border = 4.0;
                style.shadow_offs = (0.0, 0.0);
                style.border_color = hexotk::style::UI_SELECT_CLR;
            });
            let root = Widget::new(s.clone());
            root.set_ctrl(Control::Rect);
            root.enable_cache();
            root.change_layout(|l| l.layout_type = Some(LayoutType::Column));

            let list_data = Rc::new(RefCell::new(ListData::new()));
            list_data.borrow_mut().push("Test123".to_string());
            list_data.borrow_mut().push("fiefi oewf eowijfewo ifjweo jwefoi jweofiew".to_string());
            list_data.borrow_mut().push("Oooofeofewofe wf ewf owef ewo".to_string());

            let list = Widget::new(s.with_style_clone(|s| {
                s.bg_color = hexotk::style::UI_HLIGHT2_CLR;
                s.border_color = (0.0, 1.0, 0.0);
            }));
            list.set_ctrl(Control::List { list: Box::new(List::new(list_data.clone())) });
            list.enable_cache();
            list.change_layout(|l| {
                l.left = Some(Units::Pixels(30.0));
                l.top = Some(Units::Pixels(50.0));
                l.width = Some(Units::Pixels(100.0));
            });

            root.add(list);

            let mut ui = Box::new(UI::new(Rc::new(RefCell::new(1))));

            ui.add_layer_root(root);

            ui
        }),
    );
}
