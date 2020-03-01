use std::io::Cursor;
use std::fs;
use std::path::Path;

#[derive(Copy, Clone, Debug)]
pub struct UIVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2]
}

impl UIVertex{
    pub const fn new(position: [f32; 2], uv: [f32; 2]) -> Self{
        Self{
            position,
            uv
        }
    }
}

implement_vertex!(UIVertex, position, uv);

#[derive(Clone)]
pub struct UIMeshData{
    pub vertices: Vec<UIVertex>,
    pub indices: Vec<u32>
}

impl UIMeshData{
    pub fn new() -> Self{
        Self{
            vertices: Vec::new(),
            indices: Vec::new()
        }
    }

    pub fn add(&mut self, vertices: Vec<UIVertex>, mut indices: Vec<u32>){
        let index_count = self.vertices.len() as u32;

        for index in &mut indices {
            *index += index_count
        }

        self.vertices.extend_from_slice(&vertices);
        self.indices.extend_from_slice(&indices);
    }

    pub fn build(&self, display: &glium::Display) -> UIMesh{
        let vb = glium::vertex::VertexBuffer::new(display, &self.vertices[..]).expect("Couldn't create VB");
        let ib = glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &self.indices[..]).expect("Couldn't create IB");
        UIMesh{
            vb,
            ib
        }
    }
}

pub struct UIMesh{
    vb: glium::vertex::VertexBuffer<UIVertex>,
    ib: glium::index::IndexBuffer<u32>
}

impl UIMesh{
    pub fn get_vb(&self) -> &glium::vertex::VertexBuffer<UIVertex>{
        &self.vb
    }

    pub fn get_ib(&self) -> &glium::index::IndexBuffer<u32>{
        &self.ib
    }
 }

pub type Texture2D = glium::texture::srgb_texture2d::SrgbTexture2d;
pub struct UIManager{
    texture: Texture2D,
    shader_program: glium::Program,
    mesh: UIMesh,
}

impl UIManager{
    pub fn new(display: &glium::Display, path: &Path, image_type: image::ImageFormat) -> Self{
        let cargo = env!("CARGO_MANIFEST_DIR");
        let path = Path::new(cargo).join("res").join(path);

        let data = std::fs::read(path).expect("Couldn't read image!");
        let bytes = Cursor::new(&data[..]);
        let image = image::load(bytes, image_type).expect("Couldn't load image!").to_rgba();
        let dimensions = image.dimensions();
        let raw_texture = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), dimensions);

        let texture = Texture2D::new(display, raw_texture).expect("Couldn't create Texture2D");

        let vertex_shader_src =
            fs::read_to_string(&Path::new(cargo).join("res").join("shaders").join("ui").join("vertex.glsl"))
                .expect("Something went wrong reading the file");
        let fragment_shader_src =
            fs::read_to_string(&Path::new(cargo).join("res").join("shaders").join("ui").join("fragment.glsl"))
                .expect("Something went wrong reading the file");
        let shader_program =
            glium::Program::from_source(display, &vertex_shader_src, &fragment_shader_src, None)
                .expect("Couldn't build UI Shader Program");

        let mut ui_mesh = UIMeshData::new();
        let vertices = vec![
            UIVertex::new([0., 0.], [0., 0.]),
            UIVertex::new([0.5, 0.], [1., 0.]),
            UIVertex::new([0., 0.5], [0., 1.]),
            UIVertex::new([0.5, 0.5], [1., 1.]),
        ];
        let indices = vec![2, 3, 1, 1, 0, 2];
        ui_mesh.add(vertices, indices);
        let mesh = ui_mesh.build(display);

        Self{
            texture,
            shader_program,
            mesh
        }
    }

    pub fn get_sampled(&self) -> glium::uniforms::Sampler<Texture2D>{
        self.texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
    }

    pub fn get_mesh(&self) -> &UIMesh{
        &self.mesh
    }

    pub fn get_shader(&self) -> &glium::Program{
        &self.shader_program
    }
}
