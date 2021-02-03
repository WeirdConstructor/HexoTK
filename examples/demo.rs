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
}

impl WidgetUI for DemoUI {
    fn define_active_zone_rect(&mut self, az: ActiveZone, x: f64, y: f64, w: f64, h: f64) {
    }

    fn add_widget_type(&mut self, w_type_id: usize, wtype: Box<dyn WidgetType>) {
        if w_type_id >= self.types.len() {
            self.types.resize_with((w_type_id + 1) * 2, || Box::new(DummyWidget::new()));
        }

        self.types[w_type_id] = wtype;
    }

    fn draw_widget(&mut self, w_type_id: usize, data: &mut WidgetData, rect: Rect) {
    }

    fn grab_focus(&mut self) {
    }

    fn release_focus(&mut self) {
    }

    fn emit_event(&self, event: UIEvent) {
    }
}

impl WindowUI for DemoUI {
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
    }

    fn draw(&mut self, painter: &mut dyn Painter) {
        painter.label(20.0, 0, (1.0, 1.0, 0.0), 10.0, 40.0, 100.0, 20.0, "TEST");
    }

    fn set_window_size(&mut self, w: f64, h: f64) {
    }
}

fn main() {
    open_window("HexoTK Demo", 400, 400, None, Box::new(|| {
        Box::new(DemoUI {
            types: vec![],
        })
    }));
}

