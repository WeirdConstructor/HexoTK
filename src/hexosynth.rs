use crate::widgets::hexgrid::HexGridModel;
use std::rc::Rc;

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
pub struct MatrixModel {
    w: usize,
    h: usize,
    cells: Vec<MatrixCell>,
}

impl MatrixModel {
    pub fn new(w: usize, h: usize) -> Self {
        let mut cells = Vec::new();
        cells.resize(w * h, MatrixCell::empty());

        let dmy = Rc::new(DummyNode { name: String::from("Dmy") });

        cells[10] = MatrixCell::new(dmy.clone(), 0).out(Some(1), Some(1), None).input(Some(2), Some(0), None);
        cells[11] = MatrixCell::new(dmy.clone(), 1).out(Some(0), Some(1), None).input(Some(1), Some(3), None);;

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

impl HexGridModel for MatrixModel {
    fn width(&self) -> usize { self.w }
    fn height(&self) -> usize { self.h }

    fn cell_empty(&self, x: usize, y: usize) -> bool {
        if x >= self.w || y >= self.h { return true; }

        !self.cells[y * self.w + x].node.is_some()
    }

    fn cell_visible(&self, x: usize, y: usize) -> bool {
        if x >= self.w || y >= self.h { return false; }

        self.cells[y * self.w + x].visible
    }

    fn cell_label<'a>(&self, x: usize, y: usize, mut buf: &'a mut [u8]) -> Option<&'a str> {
        if x >= self.w || y >= self.h { return None; }
        let cell = &self.cells[y * self.w + x];

        if let Some(node) = &cell.node {
            use std::io::Write;
            let orig_len = buf.len();
            let mut cur = std::io::Cursor::new(buf);
            match write!(cur, "{} {}", node.0.name(), node.1 + 1) {
                Ok(_)  => {
                    let len = cur.position() as usize;
                    Some(
                        std::str::from_utf8(&(cur.into_inner())[0..len])
                        .unwrap())
                },
                Err(_) => None,
            }
        } else {
            None
        }
    }

    fn cell_edge<'a>(&self, x: usize, y: usize, edge: u8, out: &'a mut [u8]) -> Option<&'a str> {
        if x >= self.w || y >= self.h { return None; }
        let cell = &self.cells[y * self.w + x];

        if let Some(node) = &cell.node {
            let param_idx =
                match edge {
                    0 => cell.in1,
                    1 => cell.out1,
                    2 => cell.out2,
                    3 => cell.out3,
                    4 => cell.in3,
                    5 => cell.in2,
                    _ => None,
                };

            if let Some(param_idx) = param_idx {
                let param_name =
                    if edge == 1 || edge == 2 || edge == 3 {
                        node.0.output_label(param_idx).unwrap_or("?")
                    } else {
                        node.0.input_label(param_idx).unwrap_or("?")
                    };

                let byt_len = param_name.as_bytes().len();
                out[0..byt_len].copy_from_slice(param_name.as_bytes());
                Some(std::str::from_utf8(&out[0..byt_len]).unwrap())
            } else {
                None
            }
        } else {
            None
        }
    }
}
