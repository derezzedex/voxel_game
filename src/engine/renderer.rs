use crate::engine::Vertex;
use std::path::Path;
use std::fs;
use glium::uniforms::{AsUniformValue, Uniforms};
use glium::{glutin, Surface};

use crate::engine::WinitDisplay;
use crate::engine::ui::Ui;

pub const DEFAULT_WIDTH: u32 = 800;
pub const DEFAULT_HEIGHT: u32 = 600;

pub struct Context<'a>{
    pub events_loop: glium::glutin::EventsLoop,
    pub ui: Ui,
    pub display: WinitDisplay,
    pub shader_program: glium::Program,
    window_dimensions: (u32, u32),
    mouse_grab: bool,
    render_params: glium::DrawParameters<'a>,
    pub frame: Option<glium::Frame>
}

impl<'a> Context<'a>{
    pub fn new(title: &str, vert: &str, frag: &str) -> Self{
        let window_dimensions = (DEFAULT_WIDTH, DEFAULT_HEIGHT);

        let events_loop = glutin::EventsLoop::new();
        let wb = glutin::WindowBuilder::new()
                .with_title(title)
                .with_dimensions(window_dimensions.into());
        let cb = glutin::ContextBuilder::new()
                .with_depth_buffer(24)
                .with_multisampling(4)
                .with_vsync(true);
        let mut display = glium::Display::new(wb, cb, &events_loop).expect("Couldn't create the display!");

        display.gl_window().window().set_position(glium::glutin::dpi::LogicalPosition::new(0., 0.));

        let cargo_dir = env!("CARGO_MANIFEST_DIR");

        let vertex_shader_src = fs::read_to_string(&Path::new(cargo_dir).join(vert))
            .expect("Something went wrong reading the file");
        let fragment_shader_src = fs::read_to_string(&Path::new(cargo_dir).join(frag))
            .expect("Something went wrong reading the file");

        let shader_program = glium::Program::from_source(&display, &vertex_shader_src, &fragment_shader_src, None).unwrap();

        let render_params = glium::DrawParameters {
            // polygon_mode: glium::draw_parameters::PolygonMode::Line,
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            .. Default::default()
        };


        let frame = None;
        let mouse_grab = false;

        let display = WinitDisplay(display);

        let ui = Ui::new(&display, DEFAULT_WIDTH as f64, DEFAULT_HEIGHT as f64);

        Self{
            events_loop,
            display,
            window_dimensions,
            mouse_grab,
            render_params,
            ui,
            shader_program,
            frame
        }
    }

    pub fn grab_mouse(&mut self){
        self.mouse_grab = !self.mouse_grab;
        self.display.0.gl_window().window().hide_cursor(self.mouse_grab);
    }

    pub fn reset_mouse_position(&mut self){
        if self.mouse_grab{
            let (width, height) = self.window_dimensions();
            self.display.0.gl_window().window().set_cursor_position((width as f64/2., height as f64/2.).into());
        }
    }

    pub fn get_display(&self) -> &glium::Display{
        &self.display.0
    }

    pub fn poll_events(&mut self) -> Vec<glutin::Event>{
        let mut events = Vec::new();
        self.events_loop.poll_events(|e| events.push(e));
        events
    }

    pub fn get_frame(&mut self) -> &mut glium::Frame{
        self.frame.as_mut().expect("Couldn't get frame")
    }

    pub fn window_dimensions(&self) -> (u32, u32){
        // self.frame.as_ref().expect("Couldn't get frame").get_dimensions()
        self.window_dimensions
    }

    pub fn get_aspect_ratio(&self) -> f64{
        let (width, height) = self.window_dimensions();
        width as f64/ height as f64
    }

    pub fn clear_color(&mut self, color: [f32; 4]){
        self.frame.as_mut().unwrap().clear_color_and_depth((color[0], color[1], color[2], color[3]), 1.0);
    }

    pub fn draw<T: AsUniformValue, R: Uniforms>(&mut self, vb: &glium::VertexBuffer<Vertex>, ib: &glium::IndexBuffer<u32>, u: &glium::uniforms::UniformsStorage<T, R>){
        self.frame.as_mut().unwrap().draw(vb, ib, &self.shader_program, u, &self.render_params).unwrap();
    }

    pub fn draw_ui(&mut self){
        self.ui.draw(&self.display.0, self.frame.as_mut().unwrap());
    }

    pub fn new_frame(&mut self){
        let target = self.get_display().draw();
        self.frame = Some(target);
    }

    pub fn finish_frame(&mut self){
        self.frame.take().unwrap().finish();
    }
}
