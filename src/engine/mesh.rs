use crate::engine::Vertex;
use crate::game::terrain::block::*;

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

    pub fn new_raw(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self{
        Self{
            vertices,
            indices
        }
    }

    pub fn build_no_indices(&self, display: &glium::Display) -> BasicMesh{
        BasicMesh{
            vb: glium::vertex::VertexBuffer::new(display, &self.vertices[..]).expect("Couldn't create VB"),
            ib: glium::index::NoIndices(glium::index::PrimitiveType::LinesList),
        }
    }

    pub fn build(&self, display: &glium::Display) -> Mesh{
        Mesh{
            vb: glium::vertex::VertexBuffer::new(display, &self.vertices[..]).expect("Couldn't create VB"),
            ib: glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &self.indices[..]).expect("Couldn't create IB")
        }
    }

    pub fn add_face(&mut self, face: FaceData){
        let mut vertices = match face.get_direction(){
            Direction::North => NORTH_FACE,
            Direction::South => SOUTH_FACE,
            Direction::West => WEST_FACE,
            Direction::East => EAST_FACE,
            Direction::Up => UP_FACE,
            Direction::Down => DOWN_FACE
        };

        let color = match face.get_direction(){
            Direction::North | Direction::South |
            Direction::West | Direction::East => [0.58 * 0.8, 0.45 * 0.8, 0.37 * 0.8],
            Direction::Up => [0.53, 0.73, 0.34],
            Direction::Down => [0.58 * 0.4, 0.45 * 0.4, 0.37 * 0.4],
        };

        let coords = face.get_coordinates().as_vec();
        for (i, vertex) in vertices.iter_mut().enumerate() {
            vertex.position[0] += face.get_position()[0] as f32;
            vertex.position[1] += face.get_position()[1] as f32;
            vertex.position[2] += face.get_position()[2] as f32;
            vertex.color = [coords[i].0, coords[i].1, vertex.color[2]];
        }

        let mut indices = INDICES;
        let index_count = self.vertices.len() as u32;

        for index in &mut indices {
            *index += index_count
        }

        self.vertices.extend_from_slice(&vertices);
        self.indices.extend_from_slice(&indices);
    }
}

pub struct BasicMesh{
    vb: glium::vertex::VertexBuffer<Vertex>,
    ib: glium::index::NoIndices,
}

impl BasicMesh{
    pub fn get_vb(&self) -> &glium::vertex::VertexBuffer<Vertex>{
        &self.vb
    }

    pub fn get_ib(&self) -> &glium::index::NoIndices{
        &self.ib
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
