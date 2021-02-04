use hexotk::*;

struct OpDesc {
    // - Description for module
    // - Ports
    //    - Input / Output
    //    - Type: Control, Audio, Range (200...10000)
    //    - Label
    //    - Description (Help Text)
    // - UI Definition
    //    - Private Settings (UI Only)
    //    - Layout for Ports (multiple containers?)
}

// Backend:
// - All DSP Objects are represented as Node
//   - A node has a buffer for all inputs and outputs
//   - They have a "next()" method, producing the next sample
// - Define two continous arrays of input (parameters) and output values for the N nodes
//   Each node can at max have 28 inputs and 4 outputs.
//   - this ensures that all params are packed well in memory
// - A node only gets a slice of it's inputs and outputs it operates on
//   => I can reuse my abstracted stuff from Kickmess almost without change
//   => Denormalization happens here
//   => Signal outputs need to be normalized (x + 1.0) * 0.5
// - The byte code defines when a node produces a sample
//   - Directly after that op, ops follow for transferring 1, 2 or 3
//     values from the output buffers to the corresponding other nodes
//     input buffers.
//   - Math ops are maybe for later or never
//   - Byte code programs are just u8 vectors/array with a max size
//     which are copied to/from a ringbuf.
//   - Code:
//      0x01 <op_idx: u8> - Exec Node <op-idx>
//      0x02 <in_idx: u16> <out_idx: u16> <in_idx: u16> <out_idx: u16> <in_idx: u16> <out_idx: u16>
//                        - transfers 3 value to/from in/out
//      0x03 <in_idx: u16> <reg_idx: u16>
//                        - transfer value to one of the 255 registers for feedback?
//      0x04 <reg_idx: u16> <out_idx: u16>
//                        - transfer value from one of the 255 registers back
//    => Code is configuring up to K (max prog length) execution nodes
//      - They get the index stored
//      - They get 3 in/out indices stored
//      - If one exec node operates, it runs the Node
//        and then maps to/from the indices
//      => XXX: How to handle Audio <-> Control Signal changes?
//         - One idea is to introduce a state less node, that is reexecuted
//           and 3 in/outs are then converted.
//      => This means, the byte code is only interpreted once and then transferred
//         into a executable data structure
//              => u16 to usize conversion only once
//              => `match` on op code is replaced by a counter and code that
//                 knows which memory cells to fetch.
// - UI & Host Parameter changes are transmitted via RingBuffer (op_idx, in_idx, new_value)
//    - there are multiple "ramp" operators, each can be loaded with a parameter
//      change, which is then changed over the next N frames.
//      XXX => How to prevent multiple ramps from being operated for the same param?
//          => Maybe we can prevent the branching if we just play all ramps
//             there should not be too many parameter changes per process block
//             (max is 1-2 UI params and up to the 10 Mod ports)

// Widgets:
//  - Matrix
//      - Access to data model?
//      - draws the hex tiles
//      - and the labels
//  - Envelope view
//      - Point formular in Data
//  - Toggle Button
//      - Labels and values in Data
//  - Knob (3 Sizes?)
//  - Container
//      - Bordered Container
//      - Bordered Container with Title
//      - Tabs!

// - How to connect with UI code?

struct DemoUI {
    types: Vec<Box<dyn WidgetType>>,
    main: Option<Box<(usize, WidgetData)>>,
    zones: Option<Vec<ActiveZone>>,
    last_mouse: (f64, f64),
}

struct WidgetUIHolder<'a> {
    types: &'a Vec<Box<dyn WidgetType>>,
    zones: Vec<ActiveZone>,
}

impl<'a> WidgetUI for WidgetUIHolder<'a> {
    fn define_active_zone(&mut self, az: ActiveZone) {
        self.zones.push(az);
    }

    fn draw_widget(&mut self, w_type_id: usize, data: &mut WidgetData, p: &mut dyn Painter, rect: Rect) {
    }

    fn grab_focus(&mut self) {
    }

    fn release_focus(&mut self) {
    }

    fn hl_style_for(&mut self, az_id: usize) -> HLStyle {
        HLStyle::None
    }
//    fn emit_event(&self, event: UIEvent) {
//    }
}

impl WindowUI for DemoUI {
    fn add_widget_type(&mut self, w_type_id: usize, wtype: Box<dyn WidgetType>) {
        if w_type_id >= self.types.len() {
            self.types.resize_with((w_type_id + 1) * 2, || Box::new(DummyWidget::new()));
        }

        self.types[w_type_id] = wtype;
    }

    fn pre_frame(&mut self) {
    }

    fn post_frame(&mut self) {
    }

    fn needs_redraw(&mut self) -> bool {
        true
    }

    fn is_active(&mut self) -> bool {
        true
    }

    fn handle_input_event(&mut self, event: InputEvent) {
        println!("INPUT: {:?}", event);
        match event {
            InputEvent::MousePosition(x, y) => {
                self.last_mouse = (x, y);
                // TODO:
                //   - determine hover zone here
                //   - remember to redraw if the hover zone changed
            },
            InputEvent::MouseButtonPressed(btn) => {
                for z in self.zones.iter() {
                    // TODO:
                    // check if inside zone
                    // if true:
                    //     remember ID
                    //     execute event() call tree with self.main.take() Data
                    //     remember to redraw
                    println!("CHECK {:?}", z);
                }
            },
            _ => {},
        }
    }

    fn draw(&mut self, painter: &mut dyn Painter) {
        painter.label(20.0, 0, (1.0, 1.0, 0.0), 10.0, 40.0, 100.0, 20.0, "TEST");
        let mut data  = self.main.take();
        let mut zones = self.zones.take();

        if let Some(mut data) = data {
            let mut zones = zones.unwrap();
            zones.clear();

            let w_type_id = data.0;
            let wt        = &self.types[w_type_id];
            let mut wui   = WidgetUIHolder {
                types: &self.types,
                zones,
            };

            wt.draw(
                &mut wui, &mut data.1, painter, Rect::from(100.0, 100.0, 50.0, 20.0));

            self.zones = Some(wui.zones);
            self.main  = Some(data);
        }
    }

    fn set_window_size(&mut self, w: f64, h: f64) {
    }
}

fn main() {
    open_window("HexoTK Demo", 400, 400, None, Box::new(|| {
        let mut ui = Box::new(DemoUI {
            types: vec![],
            zones: Some(vec![]),
            last_mouse: (0.0, 0.0),
            main:
                Some(Box::new((0, hexotk::WidgetData::new(
                    10,
                    Box::new(hexotk::widgets::ButtonData {
                        label:  String::from("UWU"),
                        counter: 0,
                    })))))
        });

        ui.add_widget_type(0, Box::new(hexotk::widgets::Button { }));

        ui
    }));
}

