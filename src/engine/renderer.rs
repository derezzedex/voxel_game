use crate::engine::Vertex;
use glium::uniforms::{AsUniformValue, Uniforms};
use glium::{glutin, Surface};
use std::fs;
use std::path::Path;
use glium_text_rusttype as glium_text;


pub const DEFAULT_WIDTH: u32 = 800;
pub const DEFAULT_HEIGHT: u32 = 600;

pub type Text<'a> = glium_text::TextDisplay<&'a glium_text::FontTexture>;
pub struct GUIManager{
    system: glium_text::TextSystem,
    font: glium_text::FontTexture,
}

impl GUIManager{
    pub fn new(display: &glium::Display) -> Self{
        let system = glium_text::TextSystem::new(display);

        let cargo_dir = env!("CARGO_MANIFEST_DIR");
        let font_data = fs::read(&Path::new(cargo_dir).join("res").join("fonts").join("pixel-operator").join("PixelOperator8.ttf")).expect("Couldn't load font file!");
        let font = glium_text::FontTexture::new(display, &font_data[..], 100, glium_text::FontTexture::ascii_character_list()).expect("Couldn't create font!");

        Self{
            system,
            font
        }
    }

    pub fn get_system(&self) -> &glium_text::TextSystem{
        &self.system
    }

    pub fn get_font(&self) -> &glium_text::FontTexture{
        &self.font
    }

    pub fn text(&self, content: &str) -> Text{
        glium_text::TextDisplay::new(&self.system, &self.font, content)
    }
}

pub struct Context {
    pub events_loop: glium::glutin::EventsLoop,
    pub display: glium::Display,
    pub shader_program: glium::Program,
    gui: GUIManager,
    window_dimensions: (u32, u32),
    mouse_grab: bool,
    render_params: glium::DrawParameters<'static>,
    pub frame: Option<glium::Frame>,
}

impl Context {
    pub fn new(title: &str, vert: &str, frag: &str) -> Self {
        let window_dimensions = (DEFAULT_WIDTH, DEFAULT_HEIGHT);

        let events_loop = glutin::EventsLoop::new();
        let wb = glutin::WindowBuilder::new()
            .with_title(title)
            .with_dimensions(window_dimensions.into());
        let cb = glutin::ContextBuilder::new()
            .with_depth_buffer(24)
            .with_multisampling(4)
            .with_vsync(true);
        let mut display =
            glium::Display::new(wb, cb, &events_loop).expect("Couldn't create the display!");

        let gui = GUIManager::new(&display);

        display
            .gl_window()
            .window()
            .set_position(glium::glutin::dpi::LogicalPosition::new(0., 0.));

        let cargo_dir = env!("CARGO_MANIFEST_DIR");

        // NORMAL SHADER
        let vertex_shader_src =
            fs::read_to_string(&Path::new(cargo_dir).join("shaders").join(vert))
                .expect("Something went wrong reading the file");
        let fragment_shader_src =
            fs::read_to_string(&Path::new(cargo_dir).join("shaders").join(frag))
                .expect("Something went wrong reading the file");
        let shader_program =
            glium::Program::from_source(&display, &vertex_shader_src, &fragment_shader_src, None)
                .unwrap();

        let render_params = glium::DrawParameters {
            // polygon_mode: glium::draw_parameters::PolygonMode::Line,
            // line_width: Some(10f32),
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            // backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };

        let frame = None;
        let mouse_grab = true;
        display.gl_window().window().grab_cursor(mouse_grab);
        display.gl_window().window().hide_cursor(mouse_grab);

        Self {
            events_loop,
            display,
            gui,
            window_dimensions,
            mouse_grab,
            render_params,
            shader_program,
            frame,
        }
    }

    pub fn grab_mouse(&mut self) {
        self.mouse_grab = !self.mouse_grab;
        self.display
            .gl_window()
            .window()
            .grab_cursor(self.mouse_grab);
        self.display
            .gl_window()
            .window()
            .hide_cursor(self.mouse_grab);
    }

    pub fn reset_mouse_position(&mut self) {
        if self.mouse_grab {
            let (width, height) = self.window_dimensions();
            self.display
                .gl_window()
                .window()
                .set_cursor_position((width as f64 / 2., height as f64 / 2.).into());
        }
    }

    pub fn get_display(&self) -> &glium::Display {
        &self.display
    }

    pub fn poll_events(&mut self) -> Vec<glutin::Event> {
        let mut events = Vec::new();
        self.events_loop.poll_events(|e| events.push(e));
        events
    }

    pub fn get_frame(&mut self) -> &mut glium::Frame {
        self.frame.as_mut().expect("Couldn't get frame")
    }

    pub fn window_dimensions(&self) -> (u32, u32) {
        // self.frame.as_ref().expect("Couldn't get frame").get_dimensions()
        self.window_dimensions
    }

    pub fn get_aspect_ratio(&self) -> f64 {
        let (width, height) = self.window_dimensions();
        width as f64 / height as f64
    }

    pub fn clear_color(&mut self, color: [f32; 4]) {
        self.frame
            .as_mut()
            .unwrap()
            .clear_color_and_depth((color[0], color[1], color[2], color[3]), 1.0);
    }

    pub fn draw<T: AsUniformValue, R: Uniforms>(
        &mut self,
        vb: &glium::VertexBuffer<Vertex>,
        ib: &glium::IndexBuffer<u32>,
        u: &glium::uniforms::UniformsStorage<T, R>,
    ) {
        self.frame
            .as_mut()
            .unwrap()
            .draw(vb, ib, &self.shader_program, u, &self.render_params)
            .unwrap();
    }

    pub fn get_gui(&self) -> &GUIManager{
        &self.gui
    }

    pub fn get_frame_and_gui(&mut self) -> (&mut glium::Frame, &GUIManager){
        (self.frame.as_mut().expect("Couldn't get frame"), &self.gui)
    }
    pub fn get_program(&self) -> &glium::Program {
        &self.shader_program
    }

    pub fn new_frame(&mut self) {
        let target = self.get_display().draw();
        self.frame = Some(target);
    }

    pub fn finish_frame(&mut self) {
        self.frame.take().unwrap().finish();
    }
}
