use crate::engine::Vertex;

#[derive(Clone)]
pub struct MeshData{
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>
}

impl MeshData{
    pub fn new() -> Self{
        Self{
            vertices: Vec::new(),
            indices: Vec::new()
        }
    }

    pub fn add(&mut self, vertices: Vec<Vertex>, mut indices: Vec<u32>){
        let index_count = self.vertices.len() as u32;

        for index in &mut indices {
            *index += index_count
        }

        self.vertices.extend_from_slice(&vertices);
        self.indices.extend_from_slice(&indices);
    }

    pub fn build(&self, display: &glium::Display) -> Mesh{
        Mesh{
            vb: glium::vertex::VertexBuffer::immutable(display, &self.vertices[..]).expect("Couldn't create VB"),
            ib: glium::IndexBuffer::immutable(display, glium::index::PrimitiveType::TrianglesList, &self.indices[..]).expect("Couldn't create IB")
        }
    }
}

pub struct Mesh{
    vb: glium::vertex::VertexBuffer<Vertex>,
    ib: glium::index::IndexBuffer<u32>
}

impl Mesh{
    pub fn get_vb(&self) -> &glium::vertex::VertexBuffer<Vertex>{
        &self.vb
    }

    pub fn get_ib(&self) -> &glium::index::IndexBuffer<u32>{
        &self.ib
    }
 }
