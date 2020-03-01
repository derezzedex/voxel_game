use cgmath::Point3;
use crate::engine::Vertex;
use crate::game::terrain::block::Direction;

pub const UNIT: f32 = 1.;
pub const HALF: f32 = UNIT / 2.;

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

    pub fn add_face(&mut self, position: Point3<f32>, direction: Direction, block: [u32; 2]){
        let vertices = match direction{
            Direction::North => vec![
                Vertex::new([position.x + (-HALF),  position.y + (-HALF),   position.z+HALF], [0., 0.], block),
                Vertex::new([position.x + HALF,     position.y + (-HALF),   position.z+HALF], [1., 0.], block),
                Vertex::new([position.x + (-HALF),  position.y + HALF,      position.z+HALF],  [0., 1.], block),
                Vertex::new([position.x + HALF,     position.y + HALF,      position.z+HALF],  [1., 1.], block)
            ],
            Direction::South => vec![
                Vertex::new([position.x + HALF,     position.y + (-HALF),   position.z+(-HALF)], [0., 0.], block),
                Vertex::new([position.x + (-HALF),  position.y + (-HALF),   position.z+(-HALF)], [1., 0.], block),
                Vertex::new([position.x + HALF,     position.y + HALF,      position.z+(-HALF)],  [0., 1.], block),
                Vertex::new([position.x + (-HALF),  position.y + HALF,      position.z+(-HALF)],  [1., 1.], block)
            ],
            Direction::West => vec![
                Vertex::new([position.x + (-HALF),  position.y + (-HALF),   position.z+(-HALF)], [0., 0.], block),
                Vertex::new([position.x + (-HALF),  position.y + (-HALF),   position.z+HALF], [1., 0.], block),
                Vertex::new([position.x + (-HALF),  position.y + HALF,      position.z+(-HALF)], [0., 1.], block),
                Vertex::new([position.x + (-HALF),  position.y + HALF,      position.z+HALF], [1., 1.], block)
            ],
            Direction::East => vec![
                Vertex::new([position.x + HALF,     position.y + (-HALF),   position.z+HALF], [0., 0.], block),
                Vertex::new([position.x + HALF,     position.y + (-HALF),   position.z+(-HALF)], [1., 0.], block),
                Vertex::new([position.x + HALF,     position.y + HALF,      position.z+HALF], [0., 1.], block),
                Vertex::new([position.x + HALF,     position.y + HALF,      position.z+(-HALF)], [1., 1.], block)
            ],
            Direction::Top => vec![
                Vertex::new([position.x + (-HALF),  position.y + HALF,      position.z+HALF], [0., 0.], block),
                Vertex::new([position.x + HALF,     position.y + HALF,      position.z+HALF], [1., 0.], block),
                Vertex::new([position.x + (-HALF),  position.y + HALF,      position.z+(-HALF)], [0., 1.], block),
                Vertex::new([position.x + HALF,     position.y + HALF,      position.z+(-HALF)], [1., 1.], block)
            ],
            Direction::Bottom => vec![
                Vertex::new([position.x + (-HALF),  position.y + (-HALF),   position.z+(-HALF)], [0., 0.], block),
                Vertex::new([position.x + HALF,     position.y + (-HALF),   position.z+(-HALF)], [1., 0.], block),
                Vertex::new([position.x + (-HALF),  position.y + (-HALF),   position.z+HALF], [0., 1.], block),
                Vertex::new([position.x + HALF,     position.y + (-HALF),   position.z+HALF], [1., 1.], block)
            ],
        };
        let indices = vec![2, 3, 1, 1, 0, 2];
        self.add(vertices, indices);
    }

    pub fn build(&self, display: &glium::Display) -> Mesh{
        let vb = glium::vertex::VertexBuffer::new(display, &self.vertices[..]).expect("Couldn't create VB");
        let ib = glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &self.indices[..]).expect("Couldn't create IB");
        Mesh{
            vb,
            ib
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
