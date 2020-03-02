#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub block: [u32; 2]
}

impl Vertex{
    pub const fn new(position: [f32; 3], uv: [f32; 2], block: [u32; 2]) -> Vertex{
        Vertex{
            position,
            uv,
            block
        }
    }
}

implement_vertex!(Vertex, position, uv, block);

#[derive(Copy, Clone, Debug)]
pub struct DebugVertex {
    pub position: [f32; 3],
    pub color: [f32; 4]
}

impl DebugVertex{
    pub const fn new(position: [f32; 3], color: [f32; 4]) -> Self{
        Self{
            position,
            color
        }
    }
}

implement_vertex!(DebugVertex, position, color);

pub mod renderer;
pub mod mesh;
pub mod utils;
pub mod ui;
