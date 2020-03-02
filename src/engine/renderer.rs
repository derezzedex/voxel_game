use cgmath::Point3;
use crate::engine::{DebugVertex, Vertex};
use crate::engine::ui::UIManager;
use crate::engine::mesh::{DebugMeshData};

use glium::uniforms::{AsUniformValue, Uniforms};
use glium::{glutin, Surface};
use std::fs;
use std::path::Path;

pub const DEFAULT_WIDTH: u32 = 1024;
pub const DEFAULT_HEIGHT: u32 = 768;

pub struct Context {
    pub events_loop: glium::glutin::EventsLoop,
    pub display: glium::Display,
    chunk_program: glium::Program,
    debug_program: glium::Program,
    ui_manager: UIManager,
    window_dimensions: (u32, u32),
    mouse_grab: bool,
    render_params: glium::DrawParameters<'static>,
    pub frame: Option<glium::Frame>,
}

#[allow(dead_code)]
impl Context {
    pub fn new(title: &str, vert: &str, frag: &str) -> Self {
        let window_dimensions = (DEFAULT_WIDTH, DEFAULT_HEIGHT);

        let events_loop = glutin::EventsLoop::new();
        let wb = glutin::WindowBuilder::new()
            .with_title(title)
            .with_dimensions(window_dimensions.into());
        let cb = glutin::ContextBuilder::new()
            .with_srgb(true)
            .with_depth_buffer(24)
            .with_multisampling(4);
            // .with_vsync(true);
        let display =
            glium::Display::new(wb, cb, &events_loop).expect("Couldn't create the display!");

        display
            .gl_window()
            .window()
            .set_position(glium::glutin::dpi::LogicalPosition::new(0., 0.));

        let cargo_dir = env!("CARGO_MANIFEST_DIR");

        // CHUNK SHADER
        let vertex_shader_src =
            fs::read_to_string(&Path::new(cargo_dir).join("res").join("shaders").join(vert))
                .expect("Something went wrong reading the file");
        let fragment_shader_src =
            fs::read_to_string(&Path::new(cargo_dir).join("res").join("shaders").join(frag))
                .expect("Something went wrong reading the file");
        let chunk_program =
            glium::Program::from_source(&display, &vertex_shader_src, &fragment_shader_src, None)
                .unwrap();

        // DEBUG SHADER
        let vertex_shader_src =
            fs::read_to_string(&Path::new(cargo_dir).join("res").join("shaders").join("debug").join("vertex.glsl"))
                .expect("Something went wrong reading the file");
        let fragment_shader_src =
            fs::read_to_string(&Path::new(cargo_dir).join("res").join("shaders").join("debug").join("fragment.glsl"))
                .expect("Something went wrong reading the file");
        let debug_program =
            glium::Program::from_source(&display, &vertex_shader_src, &fragment_shader_src, None)
                .unwrap();

        let render_params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            blend: glium::Blend::alpha_blending(),
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullCounterClockwise,
            ..Default::default()
        };

        let ui_path = Path::new("img").join("ui").join("crosshair.png");
        let ui_manager = UIManager::new(&display, &ui_path, image::ImageFormat::Png);

        let frame = None;
        let mouse_grab = true;
        display.gl_window().window().grab_cursor(mouse_grab).expect("Couldn't grab the cursor!");
        display.gl_window().window().hide_cursor(mouse_grab);

        Self {
            events_loop,
            display,
            ui_manager,
            window_dimensions,
            mouse_grab,
            render_params,
            chunk_program,
            debug_program,
            frame,
        }
    }

    pub fn grab_mouse(&mut self) {
        self.mouse_grab = !self.mouse_grab;
        self.display
            .gl_window()
            .window()
            .grab_cursor(self.mouse_grab)
            .expect("Couldn't grab the cursor!");
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
                .set_cursor_position((width as f64 / 2., height as f64 / 2.).into())
                .expect("Couldn't set the cursor position!");
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
            .draw(vb, ib, &self.chunk_program, u, &self.render_params)
            .unwrap();
    }

    pub fn draw_ui(&mut self){
        let mesh = self.ui_manager.get_mesh();
        let texture = self.ui_manager.get_sampled();
        let projection: [[f32; 4]; 4] = cgmath::ortho(0., 10., 10., 0., 0., 1.).into();
        let size = cgmath::Matrix4::from_nonuniform_scale(0.1, 0.1, 0.);
        let position = cgmath::Matrix4::from_translation(cgmath::Vector3::new(9.7, 9.7, 0.));
        let model: [[f32; 4]; 4] = (position + size).into();
        // for i in &model{
        //     println!("{:?}", i);
        // }

        let uniforms = uniform!{
            t: texture,
            p: projection,
            m: model
        };

        let render_params = glium::DrawParameters {
            blend: glium::Blend::alpha_blending(),
            ..Default::default()
        };

        self.frame
            .as_mut()
            .unwrap()
            .draw(mesh.get_vb(), mesh.get_ib(), self.ui_manager.get_shader(), &uniforms, &render_params)
            .unwrap();
    }

    pub fn draw_line<T: AsUniformValue, R: Uniforms>(&mut self, from: Point3<f32>, to: Point3<f32>, color: [f32; 4], uniforms: &glium::uniforms::UniformsStorage<T, R>){
        let mut mesh = DebugMeshData::new();
        let vertices = vec![
            DebugVertex::new([from.x, from.y, from.z], color),
            DebugVertex::new([to.x, to.y, to.z], color),
        ];
        mesh.add(vertices);
        let mesh = mesh.build(self.get_display(), None);

        let render_params = glium::DrawParameters {
            // depth: glium::Depth {
            //     test: glium::DepthTest::IfLess,
            //     write: true,
            //     ..Default::default()
            // },
            line_width: Some(4.),
            ..Default::default()
        };

        self.frame
            .as_mut()
            .unwrap()
            .draw(mesh.get_vb(), mesh.get_ib(), &self.debug_program, uniforms, &render_params)
            .unwrap();
    }

    pub fn new_frame(&mut self) {
        let target = self.get_display().draw();
        self.frame = Some(target);
    }

    pub fn finish_frame(&mut self) {
        self.frame.take().unwrap().finish().expect("Couldn't finish frame!");
    }
}
