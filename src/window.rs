// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use femtovg::{renderer::OpenGl, Canvas, Color, FontId, ImageId};

use crate::painter::{Painter, PersistPainterData};

use raw_window_handle::RawWindowHandle;

use baseview::{
    Event, EventStatus, MouseButton, MouseEvent, ScrollDelta, Size, Window, WindowEvent,
    WindowHandle, WindowHandler, WindowOpenOptions, WindowScalePolicy,
};

use crate::{InputEvent, MButton, WindowUI};

pub struct FrameTimeMeasurement {
    buf: [u128; 240],
    idx: usize,
    cur: Option<std::time::Instant>,
    lbl: String,
}

impl FrameTimeMeasurement {
    pub fn new(lbl: &str) -> Self {
        Self { buf: [0; 240], idx: 0, cur: None, lbl: lbl.to_string() }
    }

    pub fn start_measure(&mut self) {
        self.cur = Some(std::time::Instant::now());
    }

    pub fn end_measure(&mut self) {
        if let Some(cur) = self.cur.take() {
            let dur_microseconds = cur.elapsed().as_micros();
            if (self.idx + 1) >= self.buf.len() {
                let mut min = 99999999;
                let mut max = 0;
                let mut avg = 0;

                for b in self.buf.iter() {
                    if *b < min {
                        min = *b;
                    }
                    if *b > max {
                        max = *b;
                    }
                    avg += *b;
                }

                avg /= self.buf.len() as u128;

                println!(
                    "Frame time [{:10}]: min={:5.3}, max={:5.3}, avg={:5.3}",
                    self.lbl,
                    min as f32 / 1000.0,
                    max as f32 / 1000.0,
                    avg as f32 / 1000.0
                );

                self.idx = 0;
            } else {
                self.idx += 1;
            }
            self.buf[self.idx] = dur_microseconds;
        }
    }
}

pub struct GUIWindowHandler {
    canvas: Canvas<OpenGl>,
    font: FontId,
    font_mono: FontId,
    img_buf: ImageId,
    //    ftm:        FrameTimeMeasurement,
    //    ftm_redraw: FrameTimeMeasurement,
    ui: Box<dyn WindowUI>,
    // size:       (f32, f32),
    // focused:    bool,
    counter: usize,

    bg_color: (f32, f32, f32),

    painter_data: PersistPainterData,
}

impl WindowHandler for GUIWindowHandler {
    fn on_event(&mut self, _: &mut Window, event: Event) -> EventStatus {
        match event {
            Event::Mouse(MouseEvent::CursorMoved { position: p }) => {
                self.ui.handle_input_event(InputEvent::MousePosition(p.x as f32, p.y as f32));
            }
            Event::Mouse(MouseEvent::ButtonPressed(btn)) => {
                let ev_btn = match btn {
                    MouseButton::Left => MButton::Left,
                    MouseButton::Right => MButton::Right,
                    MouseButton::Middle => MButton::Middle,
                    _ => MButton::Left,
                };
                self.ui.handle_input_event(InputEvent::MouseButtonPressed(ev_btn));
            }
            Event::Mouse(MouseEvent::ButtonReleased(btn)) => {
                let ev_btn = match btn {
                    MouseButton::Left => MButton::Left,
                    MouseButton::Right => MButton::Right,
                    MouseButton::Middle => MButton::Middle,
                    _ => MButton::Left,
                };
                self.ui.handle_input_event(InputEvent::MouseButtonReleased(ev_btn));
            }
            Event::Mouse(MouseEvent::WheelScrolled(scroll)) => match scroll {
                ScrollDelta::Lines { y, .. } => {
                    self.ui.handle_input_event(InputEvent::MouseWheel(y));
                }
                ScrollDelta::Pixels { y, .. } => {
                    self.ui.handle_input_event(InputEvent::MouseWheel(y / 50.0));
                }
            },
            Event::Keyboard(ev) => {
                use keyboard_types::KeyState;
                match ev.state {
                    KeyState::Up => {
                        self.ui.handle_input_event(InputEvent::KeyReleased(ev));
                    }
                    KeyState::Down => {
                        self.ui.handle_input_event(InputEvent::KeyPressed(ev));
                    }
                }
            }
            Event::Window(WindowEvent::WillClose) => {
                self.ui.handle_input_event(InputEvent::WindowClose);
            }
            Event::Window(WindowEvent::Focused) => {
                // self.focused = true;
            }
            Event::Window(WindowEvent::Unfocused) => {
                // self.focused = false;
            }
            Event::Window(WindowEvent::Resized(info)) => {
                let size = info.logical_size();

                self.canvas.set_size(size.width as u32, size.height as u32, 1.0);
                let (w, h) = (self.canvas.width(), self.canvas.height());
                self.canvas.delete_image(self.img_buf);
                self.img_buf = self
                    .canvas
                    .create_image_empty(
                        w as usize,
                        h as usize,
                        femtovg::PixelFormat::Rgb8,
                        femtovg::ImageFlags::FLIP_Y,
                    )
                    .expect("making image buffer");

                self.ui.set_window_size(w, h);
            }
            _ => {
                println!("UNHANDLED EVENT: {:?}", event);
            }
        }

        EventStatus::Captured
    }

    fn on_frame(&mut self, win: &mut Window) {
        self.counter += 1;
        if self.counter % 500 == 0 {
            //            println!("REDRAW.....");
            self.counter = 0;
        }

        // some hosts only stop calling idle(), so we stop the UI redraw here:
        if !self.ui.is_active() {
            return;
        }

        self.ui.pre_frame();

        self.canvas.save();
        self.canvas.clear_rect(
            0,
            0,
            self.canvas.width() as u32,
            self.canvas.height() as u32,
            Color::rgbf(self.bg_color.0, self.bg_color.1, self.bg_color.2),
        );
        self.canvas.set_render_target(femtovg::RenderTarget::Screen);
        self.painter_data.init_render_targets(femtovg::RenderTarget::Screen);

        {
            let painter = &mut Painter {
                canvas: &mut self.canvas,
                data: &mut self.painter_data,
                font: self.font,
                font_mono: self.font_mono,
                lbl_collect: None,
            };
            self.ui.draw(painter);
        }

        self.painter_data.cleanup(&mut self.canvas);

        self.canvas.flush();
        self.canvas.restore();

        win.gl_context().unwrap().swap_buffers();

        self.ui.post_frame();
    }
}

struct StupidWindowHandleHolder {
    handle: RawWindowHandle,
}

unsafe impl Send for StupidWindowHandleHolder {}

unsafe impl raw_window_handle::HasRawWindowHandle for StupidWindowHandleHolder {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.handle
    }
}

pub struct HexoTKWindowHandle {
    hdl: Option<WindowHandle>,
}

impl HexoTKWindowHandle {
    pub fn close(&mut self) {
        if let Some(mut hdl) = self.hdl.take() {
            hdl.close();
        }
    }
    pub fn is_open(&self) -> bool {
        self.hdl.as_ref().map(|h| h.is_open()).unwrap_or(false)
    }
}

impl Drop for HexoTKWindowHandle {
    fn drop(&mut self) {
        self.close()
    }
}

pub fn open_window(
    title: &str,
    window_width: i32,
    window_height: i32,
    parent: Option<RawWindowHandle>,
    factory: Box<dyn FnOnce() -> Box<dyn WindowUI> + Send>,
) -> Option<HexoTKWindowHandle> {
    //d// println!("*** OPEN WINDOW ***");
    let options = WindowOpenOptions {
        title: title.to_string(),
        size: Size::new(window_width as f64, window_height as f64),
        //            scale:     WindowScalePolicy::ScaleFactor(1.25),
        scale: WindowScalePolicy::ScaleFactor(1.0),
        gl_config: Some(baseview::gl::GlConfig::default()),
    };

    let window_create_fun = move |win: &mut Window| {
        let context = win.gl_context().unwrap();
        unsafe {
            context.make_current();
        }
        gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

        #[allow(deprecated)]
        let renderer = unsafe {
            OpenGl::new_from_function(|symbol| context.get_proc_address(symbol) as *const _)
        }
        .expect("Cannot create renderer");

        let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
        canvas.set_size(window_width as u32, window_height as u32, 1.0);
        let font = canvas.add_font_mem(std::include_bytes!("font.ttf")).expect("can load font");
        let font_mono =
            canvas.add_font_mem(std::include_bytes!("font_mono.ttf")).expect("can load font");
        let (w, h) = (canvas.width(), canvas.height());
        let img_buf = canvas
            .create_image_empty(
                w as usize,
                h as usize,
                femtovg::PixelFormat::Rgb8,
                femtovg::ImageFlags::FLIP_Y,
            )
            .expect("making image buffer");

        let mut ui = factory();

        ui.set_window_size(window_width as f32, window_height as f32);

        GUIWindowHandler {
            ui,
            // size: (window_width as f32, window_height as f32),
            canvas,
            font,
            font_mono,
            img_buf,
            //            ftm:        FrameTimeMeasurement::new("img"),
            //            ftm_redraw: FrameTimeMeasurement::new("redraw"),
            // focused:    false,
            counter: 0,
            painter_data: PersistPainterData::new(),
            bg_color: (0.3, 0.1, 0.3),
        }
    };

    if let Some(parent) = parent {
        let swhh = StupidWindowHandleHolder { handle: parent };
        Some(HexoTKWindowHandle {
            hdl: Some(Window::open_parented(&swhh, options, window_create_fun)),
        })
    } else {
        Window::open_blocking(options, window_create_fun);
        None
    }
}
