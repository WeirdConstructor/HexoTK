use crate::widgets::hexgrid::{HexEdge, HexCell, HexDir, HexGridModel};
use crate::widgets::*;
use crate::MButton;
use std::rc::Rc;
use crate::constants::*;
use crate::{UIPos, ParamID};

#[derive(Debug)]
struct DummyNode {
    name:     String,
}

impl NodeType for DummyNode {
    fn name(&self) -> &str { &self.name }
    fn output_label(&self, param: usize) -> Option<&str> {
        match param {
            0 => Some("Sig L"),
            1 => Some("Sig R"),
            _ => None,
        }
    }
    fn input_label(&self, param: usize) -> Option<&str> {
        match param {
            0 => Some("Gain"),
            1 => Some("Sig"),
            2 => Some("Cut"),
            3 => Some("Freq"),
            4 => Some("Res"),
            _ => None,
        }
    }
}

pub trait NodeType : std::fmt::Debug {
    fn name(&self) -> &str;
    fn output_label(&self, param: usize) -> Option<&str>;
    fn input_label(&self, param: usize) -> Option<&str>;
}

#[derive(Debug, Clone)]
pub struct MatrixCell {
    visible: bool,
    node: Option<(Rc<dyn NodeType>, usize)>,
    out1: Option<usize>,
    out2: Option<usize>,
    out3: Option<usize>,
    in1:  Option<usize>,
    in2:  Option<usize>,
    in3:  Option<usize>,
}

impl MatrixCell {
    fn new(node: Rc<dyn NodeType>, instance: usize) -> Self {
        Self {
            visible: true,
            node: Some((node, instance)),
            out1: None,
            out2: None,
            out3: None,
            in1:  None,
            in2:  None,
            in3:  None,
        }
    }

    fn empty() -> Self {
        Self {
            visible: true,
            node: None,
            out1: None,
            out2: None,
            out3: None,
            in1:  None,
            in2:  None,
            in3:  None,
        }
    }

    pub fn out(mut self, o1: Option<usize>, o2: Option<usize>, o3: Option<usize>) -> Self {
        self.out1 = o1;
        self.out2 = o2;
        self.out3 = o3;
        self
    }

    pub fn input(mut self, i1: Option<usize>, i2: Option<usize>, i3: Option<usize>) -> Self {
        self.in1 = i1;
        self.in2 = i2;
        self.in3 = i3;
        self
    }
}

#[derive(Debug)]
pub struct UIMatrixModel {
    w: usize,
    h: usize,
    cells: Vec<MatrixCell>,
}

impl UIMatrixModel {
    pub fn new(w: usize, h: usize) -> Self {
        let mut cells = Vec::new();
        cells.resize(w * h, MatrixCell::empty());

        let dmy = Rc::new(DummyNode { name: String::from("Dmy") });
        let x   = Rc::new(DummyNode { name: String::from("*") });

        cells[18] = MatrixCell::new(dmy.clone(), 0).out(Some(1), Some(1), None).input(Some(2), Some(0), None);
        cells[19] = MatrixCell::new(dmy.clone(), 1).out(Some(0), Some(1), None).input(Some(1), Some(3), None);
        cells[11] = MatrixCell::new(x.clone(),   2).out(Some(1), Some(1), Some(2));

        Self {
            w,
            h,
            cells,
        }
    }

    pub fn set(&mut self, x: usize, y: usize, cell: MatrixCell) {
        if x >= self.w || y >= self.h { return; }

        self.cells[y * self.w + x] = cell;
    }

    pub fn clear(&mut self, x: usize, y: usize) {
        if x >= self.w || y >= self.h { return; }

        self.cells[y * self.w + x] = MatrixCell::empty();
    }
}

impl HexGridModel for UIMatrixModel {
    fn width(&self) -> usize { self.w }
    fn height(&self) -> usize { self.h }

    fn cell_click(&self, _x: usize, _y: usize, _btn: MButton) {
    }

    fn cell_empty(&self, x: usize, y: usize) -> bool {
        if x >= self.w || y >= self.h { return true; }

        !self.cells[y * self.w + x].node.is_some()
    }

    fn cell_visible(&self, x: usize, y: usize) -> bool {
        if x >= self.w || y >= self.h { return false; }

        self.cells[y * self.w + x].visible
    }

    fn cell_label<'a>(&self, x: usize, y: usize, buf: &'a mut [u8]) -> Option<(&'a str, HexCell)> {
        if x >= self.w || y >= self.h { return None; }
        let cell = &self.cells[y * self.w + x];

        if let Some(node) = &cell.node {
            use std::io::Write;
            let mut cur = std::io::Cursor::new(buf);

            let hc =
                match (node.0.name(), node.1 + 1) {
                    ("Dmy", 1) => HexCell::HLight,
                    ("Dmy", 2) => HexCell::Select,
                    _          => HexCell::Normal,
                };

            match write!(cur, "{} {}", node.0.name(), node.1 + 1) {
                Ok(_)  => {
                    let len = cur.position() as usize;
                    Some((
                        std::str::from_utf8(&(cur.into_inner())[0..len])
                        .unwrap(),
                        hc
                    ))
                },
                Err(_) => None,
            }
        } else {
            None
        }
    }

    fn cell_edge<'a>(&self, x: usize, y: usize, edge: HexDir, out: &'a mut [u8]) -> Option<(&'a str, HexEdge)> {
        if x >= self.w || y >= self.h { return None; }
        let cell = &self.cells[y * self.w + x];

        if let Some(node) = &cell.node {
            let param_idx =
                match edge {
                    HexDir::TR => cell.out1,
                    HexDir::BR => cell.out2,
                    HexDir::B  => cell.out3,
                    HexDir::BL => cell.in3,
                    HexDir::TL => cell.in2,
                    HexDir::T  => cell.in1,
                };

            let et =
                match edge {
                    HexDir::TR => HexEdge::Arrow,
                    HexDir::BR => HexEdge::NoArrow,
                    HexDir::B  => HexEdge::ArrowValue { value: 0.2 },
                    _          => HexEdge::NoArrow,
                };

            if let Some(param_idx) = param_idx {
                let param_name =
                    if edge.is_right_half() {
                        node.0.output_label(param_idx).unwrap_or("?")
                    } else {
                        node.0.input_label(param_idx).unwrap_or("?")
                    };

                let byt_len = param_name.as_bytes().len();
                out[0..byt_len].copy_from_slice(param_name.as_bytes());
                Some((
                    std::str::from_utf8(&out[0..byt_len]).unwrap(),
                    et
                ))
            } else {
                None
            }
        } else {
            None
        }
    }
}


#[derive(Debug, Clone)]
pub struct NodeMatrix {
}

impl NodeMatrix {
    pub fn new() -> Self {
        Self { }
    }
}

use std::cell::RefCell;
use crate::widgets::{HexGrid, HexGridData};
use crate::{Rect, WidgetUI, Painter, WidgetData, WidgetType, UIEvent};

pub struct NodeMatrixModel {
    matrix: Rc<UIMatrixModel>,
    menu:   Rc<UINodeMenuModel>,
}

impl NodeMatrixModel {
    pub fn new() -> Self {
        Self {
            matrix: Rc::new(UIMatrixModel::new(8, 7)),
            menu:   Rc::new(UINodeMenuModel::new()),
        }
    }
}

pub struct UINodeMenuModel {
}

impl HexGridModel for UINodeMenuModel {
    fn width(&self) -> usize { 3 }
    fn height(&self) -> usize { 3 }

    fn cell_click(&self, x: usize, y: usize, btn: MButton) {
        println!("MENU CLICK CELL: {},{}: {:?}", x, y, btn);
    }

    fn cell_hover(&self, x: usize, y: usize) {
        println!("MENU HOVER CELL: {},{}", x, y);
    }

    fn cell_empty(&self, x: usize, y: usize) -> bool {
        if x >= 3 || y >= 3 { return true; }
        false
    }

    fn cell_visible(&self, x: usize, y: usize) -> bool {
        if x >= 3 || y >= 3 { return false; }
        if x == 0 && y == 0 || x == 2 && y == 0 { return false; }
        true
    }

    fn cell_label<'a>(&self, x: usize, y: usize, mut _buf: &'a mut [u8]) -> Option<(&'a str, HexCell)> {
        if x >= 3 || y >= 3 { return None; }
        Some(("test", HexCell::Plain))
    }

    fn cell_edge<'a>(&self, _x: usize, _y: usize, _edge: HexDir, _out: &'a mut [u8]) -> Option<(&'a str, HexEdge)> {
        None
    }
}


impl UINodeMenuModel {
    pub fn new() -> Self {
        Self {
        }
    }
}

#[allow(dead_code)]
pub struct NodeMatrixData {
    hex_grid:     Box<WidgetData>,
    hex_menu:     Box<WidgetData>,
    model:        Rc<RefCell<NodeMatrixModel>>,
    matrix_model: Rc<UIMatrixModel>,
    menu_model:   Rc<UINodeMenuModel>,
    display_menu: Option<(f64, f64)>,
}

impl NodeMatrixData {
    pub fn new(pos: UIPos, node_id: u32) -> WidgetData {
        let wt_nmatrix  = Rc::new(NodeMatrix::new());
        let wt_cont     = Rc::new(Container::new());

        let model        = Rc::new(RefCell::new(NodeMatrixModel::new()));
        let matrix_model = model.borrow().matrix.clone();
        let menu_model   = model.borrow().menu.clone();

        let wt_hexgrid =
            Rc::new(HexGrid::new(14.0, 10.0));
        let wt_hexgrid_menu =
            Rc::new(HexGrid::new_y_offs(14.0, 10.0).bg_color(UI_GRID_BG2_CLR));


        let mut hex_menu = ContainerData::new();
        hex_menu.contrast_border().title("Hex Menu")
           .new_row()
           .add_direct(WidgetData::new(
                    wt_hexgrid_menu.clone(),
                    ParamID::new(node_id, 2),
                    UIPos::center(12, 12),
                    HexGridData::new(menu_model.clone())));

        WidgetData::new(
            wt_nmatrix,
            ParamID::new(node_id, 1),
            pos,
            Box::new(Self {
                hex_grid: WidgetData::new_tl_box(
                    wt_hexgrid.clone(),
                    ParamID::new(node_id, 1),
                    HexGridData::new(matrix_model.clone())),
                hex_menu: WidgetData::new_tl_box(
                    wt_cont,
                    ParamID::new(node_id, 100),
                    hex_menu),
                model,
                matrix_model,
                menu_model,
                display_menu: None,
            }))
    }
}

impl WidgetType for NodeMatrix {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        data.with(|data: &mut NodeMatrixData| {
            (*data.hex_grid).draw(ui, p, pos);

            if let Some(mouse_pos) = data.display_menu {
                let menu_w = 270.0;
                let menu_h = 280.0 + UI_ELEM_TXT_H + 2.0 * UI_BORDER_WIDTH;

                let menu_rect =
                    Rect::from(
                        mouse_pos.0 - menu_w * 0.5,
                        mouse_pos.1 - menu_h * 0.5,
                        menu_w,
                        menu_h)
                    .move_into(&pos);

                (*data.hex_menu).draw(ui, p, menu_rect);
            }
        });
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        match ev {
            UIEvent::Click { id, .. } => {
                println!("EV: {:?} id={}, data.id={}", ev, *id, data.id());
                if id.node_id() == data.id().node_id() {
                    data.with(|data: &mut NodeMatrixData| {
                        if let Some(_) = data.display_menu {
                            data.hex_menu.event(ui, ev);
                            data.display_menu = None;
                        } else {
                            match ev {
                                UIEvent::Click { x, y, .. } => {
                                    data.display_menu = Some((*x, *y));
                                },
                                _ => {}
                            }
                        }
                    });
                    ui.queue_redraw();
                }
            },
            _ => {
                println!("EV: {:?}", ev);
            },
        }
    }
}
