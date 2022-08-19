// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use femtovg::{renderer::OpenGl, Canvas, Color, FontId, ImageId};
use glow;
use glow::HasContext;

use crate::painter::{Painter, PersistPainterData};

use raw_window_handle::RawWindowHandle;

use baseview::{
    Event, EventStatus, MouseButton, MouseEvent, ScrollDelta, Size, Window, WindowEvent,
    WindowHandle, WindowHandler, WindowOpenOptions, WindowScalePolicy,
};

use crate::{InputEvent, MButton, WindowUI};

#[repr(C)]
pub struct Vertex {
    pub pos: [f32; 2],
}

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

pub struct GlStuff {
    program: glow::NativeProgram,
    vao: glow::NativeVertexArray,
    vbo: glow::NativeBuffer,
}

pub struct GUIWindowHandler {
    canvas: Canvas<OpenGl>,
    context: glow::Context,
    gl_stuff: Option<GlStuff>,
    font: FontId,
    font_mono: FontId,
    img_buf: ImageId,
    //    ftm:        FrameTimeMeasurement,
    //    ftm_redraw: FrameTimeMeasurement,
    ui: Box<dyn WindowUI>,
    size: (f32, f32),
    dpi_factor: f32,
    scale_policy: WindowScalePolicy,
    // focused:    bool,
    counter: usize,

    bg_color: (f32, f32, f32),

    painter_data: PersistPainterData,
}

impl WindowHandler for GUIWindowHandler {
    fn on_event(&mut self, _: &mut Window, event: Event) -> EventStatus {
        match event {
            Event::Mouse(MouseEvent::CursorMoved { position: p }) => {
                self.ui.handle_input_event(InputEvent::MousePosition(
                    p.x as f32 * self.dpi_factor,
                    p.y as f32 * self.dpi_factor,
                ));
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
                let size = info.physical_size();
                let scale = match self.scale_policy {
                    WindowScalePolicy::SystemScaleFactor => info.scale() as f32,
                    WindowScalePolicy::ScaleFactor(scale) => scale as f32,
                };
                self.dpi_factor = scale;

                println!(
                    "DPI FACTOR = {}, Phys={:?}, Logic={:?}",
                    self.dpi_factor,
                    info.physical_size(),
                    info.logical_size(),
                );

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

                self.size = (w, h);
                self.ui.set_window_size(w, h, self.dpi_factor);
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
        self.canvas.set_render_target(femtovg::RenderTarget::Image(self.img_buf));
        self.painter_data.init_render_targets(femtovg::RenderTarget::Image(self.img_buf));

        self.canvas.clear_rect(
            0,
            0,
            self.canvas.width() as u32,
            self.canvas.height() as u32,
            Color::rgbf(self.bg_color.0, self.bg_color.1, self.bg_color.2),
        );

        {
            let painter = &mut Painter {
                canvas: &mut self.canvas,
                data: &mut self.painter_data,
                font: self.font,
                font_mono: self.font_mono,
                lbl_collect: None,
                dpi_factor: self.dpi_factor,
            };
            self.ui.draw(painter);
        }

        self.painter_data.cleanup(&mut self.canvas);

        let img_paint = femtovg::Paint::image(
            self.img_buf,
            0.0,
            0.0,
            self.canvas.width(),
            self.canvas.height(),
            0.0,
            1.0,
        );
        let mut path = femtovg::Path::new();
        path.rect(0.0, 0.0, self.canvas.width(), self.canvas.height());

        self.canvas.set_render_target(femtovg::RenderTarget::Screen);
        self.canvas.fill_path(&mut path, img_paint);

        self.canvas.flush();
        self.canvas.restore();

        {
            if self.gl_stuff.is_none() {
                let vsrc = r#"#version 330
in vec2 iPosition;

void main() {
    gl_Position = vec4(iPosition, 0.0, 1.0);
}
                "#;

                let fsrc =
                    r#"#version 330
layout(location = 0) out vec4 diffuseColor;
void main() {
    diffuseColor = vec4(vec3(1.0, 0.0, 0.5), 0.0);
}
                "#;

                let prog = unsafe {
                    let vshader = self.context.create_shader(glow::VERTEX_SHADER).expect("Create Shader Ok");
                    self.context.shader_source(vshader, vsrc);
                    self.context.compile_shader(vshader);
                    // FIXME: Compile errors!
                    if !self.context.get_shader_compile_status(vshader) {
                        println!("ERROR VERT: {}", self.context.get_shader_info_log(vshader));
                    }

                    let fshader = self.context.create_shader(glow::FRAGMENT_SHADER).expect("Create Shader Ok");
                    self.context.shader_source(fshader, fsrc);
                    self.context.compile_shader(fshader);
                    if !self.context.get_shader_compile_status(fshader) {
                        println!("ERROR FRAG: {}", self.context.get_shader_info_log(fshader));
                    }
                    // FIXME: Compile errors!

                    let prog = self.context.create_program().expect("Can create program");
                    self.context.attach_shader(prog, vshader);
                    self.context.attach_shader(prog, fshader);
                    self.context.link_program(prog);
                    if !self.context.get_program_link_status(prog) {
                        println!("ERROR PROG: {}", self.context.get_program_info_log(prog));
                    }
                    // FIXME: Link errors!

                    self.context.delete_shader(fshader);
                    self.context.delete_shader(vshader);

                    prog
                };

//                // ...
//                let verts: [Vertex; 6] = [
//                    Vertex { pos: [-1.0, -1.0] },
//                    Vertex { pos: [1.0, -1.0] },
//                    Vertex { pos: [1.0, 1.0] },
//                    Vertex { pos: [1.0, 1.0] },
//                    Vertex { pos: [-1.0, 1.0] },
//                    Vertex { pos: [-1.0, -1.0] },
//                ];

                let verts = vec![
                    -1.0, -1.0,
                    1.0, -1.0,
                    1.0, 1.0,
                    1.0, 1.0,
                    -1.0, 1.0,
                    -1.0, -1.0,
                ];

                unsafe {
                    let va = self.context.create_vertex_array().expect("Can create vertex array");
                    let vb = self.context.create_buffer().expect("Can create buffer");

                    self.context.bind_vertex_array(Some(va));
                    self.context.bind_buffer(glow::ARRAY_BUFFER, Some(vb));
                    let data = std::slice::from_raw_parts(
                        verts.as_ptr() as *const u8,
                        verts.len() * std::mem::size_of::<f32>()
                    );
                    self.context.buffer_data_u8_slice(
                        glow::ARRAY_BUFFER,
                        &data,
                        glow::STATIC_DRAW,
                    );

                    self.context.enable_vertex_attrib_array(0);
                    self.context.vertex_attrib_pointer_f32(
                        0,
                        2,
                        glow::FLOAT,
                        false,
                        (std::mem::size_of::<f32>() * 2) as i32,
                        0
                    );

                    self.context.bind_vertex_array(None);

                    self.gl_stuff = Some(GlStuff { program: prog, vao: va, vbo: vb });
                }
            }

            unsafe {
                self.context.enable(glow::BLEND);
                self.context.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
                self.context.enable(glow::DEPTH_TEST);
                self.context.depth_func(gl::LESS);
                self.context.clear_color(0.2, 1.0, 0.2, 1.0);

                self.context.viewport(10, 10, 500, 320);

                self.context.enable(glow::SCISSOR_TEST);
                self.context.scissor(10, 10, 500, 320);
            }

            unsafe {
                self.context.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

                if let Some(gls) = &self.gl_stuff {
                    self.context.use_program(Some(gls.program));
                    self.context.bind_vertex_array(Some(gls.vao));
                    self.context.draw_arrays(gl::TRIANGLES, 0, 6);
                    self.context.bind_vertex_array(None);
                }

                self.context.disable(glow::BLEND);
                self.context.disable(glow::DEPTH_TEST);
                self.context.disable(glow::SCISSOR_TEST);
            }
        }

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
    open_window_ext(
        title,
        window_width,
        window_height,
        parent,
        factory,
        WindowScalePolicy::SystemScaleFactor,
    )
}

pub fn open_window_ext(
    title: &str,
    window_width: i32,
    window_height: i32,
    parent: Option<RawWindowHandle>,
    factory: Box<dyn FnOnce() -> Box<dyn WindowUI> + Send>,
    scale_policy: WindowScalePolicy,
) -> Option<HexoTKWindowHandle> {
    let dpi_factor = match scale_policy {
        WindowScalePolicy::SystemScaleFactor => 1.0_f32, // How to get the real one?
        WindowScalePolicy::ScaleFactor(scale) => scale as f32,
    };

    //d// println!("*** OPEN WINDOW ***");
    let options = WindowOpenOptions {
        title: title.to_string(),
        size: Size::new(window_width as f64, window_height as f64),
        scale: scale_policy,
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

        let context = unsafe {
            glow::Context::from_loader_function(|symbol| {
                context.get_proc_address(symbol) as *const _
            })
        };

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

        let mut painter_data = PersistPainterData::new();

        for (file, data) in ui.get_image_data().iter() {
            painter_data.preload_image(file, data.clone());
        }

        ui.set_window_size(window_width as f32, window_height as f32, dpi_factor);

        GUIWindowHandler {
            ui,
            size: (window_width as f32, window_height as f32),
            dpi_factor,
            scale_policy,
            canvas,
            context,
            gl_stuff: None,
            font,
            font_mono,
            img_buf,
            //            ftm:        FrameTimeMeasurement::new("img"),
            //            ftm_redraw: FrameTimeMeasurement::new("redraw"),
            // focused:    false,
            counter: 0,
            painter_data,
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
