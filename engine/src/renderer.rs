use crate::Vertex;
use crate::hud;
use crate::mesh::debug;
use cgmath::Point3;

use glium::backend::glutin::SimpleWindowBuilder;
use glium::glutin::surface::WindowSurface;
use glium::uniforms::{AsUniformValue, Uniforms};
use glium::winit::event_loop::EventLoop;
use glium::winit::window::{CursorGrabMode, Window};
use glium::{Surface, glutin, winit};
use std::fs;
use std::path::Path;

pub const DEFAULT_WIDTH: u32 = 1024;
pub const DEFAULT_HEIGHT: u32 = 768;

pub struct Context {
    window: Window,
    display: glium::Display<WindowSurface>,
    chunk_program: glium::Program,
    debug_program: glium::Program,
    hud: hud::Renderer,
    window_dimensions: (u32, u32),
    render_params: glium::DrawParameters<'static>,
    frame: Option<glium::Frame>,
}

impl Context {
    pub fn new(event_loop: &EventLoop<()>, title: &str, vert: &str, frag: &str) -> Self {
        let window_dimensions = (DEFAULT_WIDTH, DEFAULT_HEIGHT);

        let config = glutin::config::ConfigTemplateBuilder::new()
            .with_depth_size(24)
            .with_multisampling(4);

        let (window, display) = SimpleWindowBuilder::new()
            .with_title(title)
            .with_inner_size(DEFAULT_WIDTH, DEFAULT_HEIGHT)
            .with_vsync(true)
            .with_config_template_builder(config)
            .build(event_loop);

        window.set_outer_position(winit::dpi::LogicalPosition::new(0, 0));

        let shader_path = Path::new(env!("CARGO_WORKSPACE_DIR"))
            .join("assets")
            .join("shaders");
        println!("Shader directory: {:?}", &shader_path);

        // CHUNK SHADER
        let vertex_shader_src = fs::read_to_string(&shader_path.join(vert))
            .expect("Something went wrong reading the file");
        let fragment_shader_src = fs::read_to_string(&shader_path.join(frag))
            .expect("Something went wrong reading the file");
        let chunk_program = glium::Program::new(
            &display,
            glium::program::ProgramCreationInput::SourceCode {
                vertex_shader: &vertex_shader_src,
                fragment_shader: &fragment_shader_src,
                geometry_shader: None,
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                transform_feedback_varyings: None,
                outputs_srgb: false,
                uses_point_size: false,
            },
        )
        .unwrap();

        // DEBUG SHADER
        let vertex_shader_src = fs::read_to_string(&shader_path.join("debug").join("vertex.glsl"))
            .expect("Something went wrong reading the file");
        let fragment_shader_src =
            fs::read_to_string(&shader_path.join("debug").join("fragment.glsl"))
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
        let ui_manager = hud::Renderer::new(&display, &ui_path, image::ImageFormat::Png);

        let frame = None;

        window
            .set_cursor_grab(CursorGrabMode::Locked)
            .expect("Couldn't grab the cursor!");
        window.set_cursor_visible(false);

        Self {
            window,
            display,
            hud: ui_manager,
            window_dimensions,
            render_params,
            chunk_program,
            debug_program,
            frame,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn display(&self) -> &glium::Display<WindowSurface> {
        &self.display
    }

    pub fn frame(&mut self) -> &mut glium::Frame {
        self.frame.as_mut().expect("Couldn't get frame")
    }

    pub fn aspect_ratio(&self) -> f64 {
        let (width, height) = self.window_dimensions;
        width as f64 / height as f64
    }

    pub fn clear_color(&mut self, color: [f32; 4]) {
        self.frame
            .as_mut()
            .unwrap()
            .clear_color_and_depth((color[0], color[1], color[2], color[3]), 1.0);
    }

    pub fn draw_with_params<T: AsUniformValue, R: Uniforms>(
        &mut self,
        vb: &glium::VertexBuffer<Vertex>,
        ib: &glium::IndexBuffer<u32>,
        u: &glium::uniforms::UniformsStorage<T, R>,
        r: glium::DrawParameters,
    ) {
        self.frame
            .as_mut()
            .unwrap()
            .draw(vb, ib, &self.chunk_program, u, &r)
            .unwrap();
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

    pub fn draw_ui(&mut self) {
        let mesh = self.hud.mesh();
        let texture = self.hud.sampler();
        let projection: [[f32; 4]; 4] = cgmath::ortho(0., 10., 10., 0., 0., 1.).into();
        let size = cgmath::Matrix4::from_nonuniform_scale(0.1, 0.1, 0.);
        let position = cgmath::Matrix4::from_translation(cgmath::Vector3::new(9.7, 9.7, 0.));
        let model: [[f32; 4]; 4] = (position + size).into();

        let uniforms = glium::uniform! {
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
            .draw(
                mesh.get_vb(),
                mesh.get_ib(),
                self.hud.shader(),
                &uniforms,
                &render_params,
            )
            .unwrap();
    }

    pub fn draw_line<T: AsUniformValue, R: Uniforms>(
        &mut self,
        from: Point3<f32>,
        to: Point3<f32>,
        color: [f32; 4],
        uniforms: &glium::uniforms::UniformsStorage<T, R>,
    ) {
        let mut mesh = debug::MeshData::new();
        let vertices = vec![
            debug::Vertex::new([from.x, from.y, from.z], color),
            debug::Vertex::new([to.x, to.y, to.z], color),
        ];
        mesh.add(vertices, vec![0, 1]);
        let mesh = mesh.build(&self.display, glium::index::PrimitiveType::LinesList);

        let render_params = glium::DrawParameters {
            line_width: Some(4.),
            ..Default::default()
        };

        self.frame
            .as_mut()
            .unwrap()
            .draw(
                mesh.vertices(),
                mesh.indices(),
                &self.debug_program,
                uniforms,
                &render_params,
            )
            .unwrap();
    }

    pub fn draw_hitbox<T: AsUniformValue, R: Uniforms>(
        &mut self,
        min: Point3<f32>,
        max: Point3<f32>,
        color: [f32; 4],
        uniforms: &glium::uniforms::UniformsStorage<T, R>,
    ) {
        let mut mesh = debug::MeshData::new();
        let min = min.map(|p| p - (1. / 1000.));
        let max = max.map(|p| p + (1. / 1000.));
        mesh.add(
            vec![
                // back
                debug::Vertex::new([min.x, min.y, min.z], color), // 0, back-left-bottom
                debug::Vertex::new([min.x, min.y, max.z], color), // 1, back-right-bottom
                debug::Vertex::new([min.x, max.y, max.z], color), // 2, back-right-top
                debug::Vertex::new([min.x, max.y, min.z], color), // 3, back-left-top
                // front
                debug::Vertex::new([max.x, min.y, min.z], color), // 4, front-left-bottom
                debug::Vertex::new([max.x, min.y, max.z], color), // 5, front-right-bottom
                debug::Vertex::new([max.x, max.y, max.z], color), // 6, front-right-top
                debug::Vertex::new([max.x, max.y, min.z], color), // 7, front-left-top
            ],
            vec![
                // back
                0, 1, 0, 3, 1, 2, 2, 3, //front
                4, 5, 4, 7, 5, 6, 6, 7, // left-bottom
                0, 4, // left-top
                3, 7, // right-bottom
                1, 5, // right-top
                2, 6,
            ],
        );
        let mesh = mesh.build(&self.display, glium::index::PrimitiveType::LinesList);

        let render_params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            line_width: Some(2.),
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullCounterClockwise,
            ..Default::default()
        };

        self.frame
            .as_mut()
            .unwrap()
            .draw(
                mesh.vertices(),
                mesh.indices(),
                &self.debug_program,
                uniforms,
                &render_params,
            )
            .unwrap();
    }

    pub fn new_frame(&mut self) {
        let target = self.display.draw();
        self.frame = Some(target);
    }

    pub fn finish_frame(&mut self) {
        self.frame
            .take()
            .unwrap()
            .finish()
            .expect("Couldn't finish frame!");
    }
}
