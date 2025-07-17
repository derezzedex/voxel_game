use glium::glutin::surface::WindowSurface;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl Vertex {
    pub const fn new(position: [f32; 3], color: [f32; 4]) -> Self {
        Self { position, color }
    }
}

glium::implement_vertex!(Vertex, position, color);

#[derive(Clone)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl MeshData {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn add(&mut self, vertices: Vec<Vertex>, indices: Vec<u32>) {
        self.vertices.extend_from_slice(&vertices);
        self.indices.extend_from_slice(&indices);
    }

    pub fn build(
        &self,
        display: &glium::Display<WindowSurface>,
        primitive: glium::index::PrimitiveType,
    ) -> Mesh {
        let vb =
            glium::vertex::VertexBuffer::new(display, &self.vertices).expect("Couldn't create VB");
        let ib =
            glium::IndexBuffer::new(display, primitive, &self.indices).expect("Couldn't create IB");

        Mesh { vb, ib }
    }
}

pub struct Mesh {
    vb: glium::vertex::VertexBuffer<Vertex>,
    ib: glium::index::IndexBuffer<u32>,
}

impl Mesh {
    pub fn vertices(&self) -> &glium::vertex::VertexBuffer<Vertex> {
        &self.vb
    }

    pub fn indices(&self) -> &glium::index::IndexBuffer<u32> {
        &self.ib
    }
}
