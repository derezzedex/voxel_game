pub mod hud;
pub mod mesh;
pub mod renderer;
pub mod utils;

pub use glium::winit;

use cgmath::{InnerSpace, Vector3};

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub block: [u32; 2],
    pub tint: [f32; 4],
}

impl Vertex {
    pub const fn new(position: [f32; 3], uv: [f32; 2], block: [u32; 2]) -> Vertex {
        Vertex {
            position,
            uv,
            block,
            tint: [1., 1., 1., 1.],
        }
    }
}

glium::implement_vertex!(Vertex, position, uv, block, tint);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Direction {
    East = 0,
    West = 1,
    Top = 2,
    Bottom = 3,
    North = 4,
    South = 5,
}

impl Direction {
    pub fn normal(&self) -> Vector3<isize> {
        match self {
            Direction::East => Vector3::new(1, 0, 0),
            Direction::West => Vector3::new(-1, 0, 0),
            Direction::Top => Vector3::new(0, 1, 0),
            Direction::Bottom => Vector3::new(0, -1, 0),
            Direction::North => Vector3::new(0, 0, 1),
            Direction::South => Vector3::new(0, 0, -1),
        }
    }
}

impl From<Vector3<f32>> for Direction {
    fn from(v: Vector3<f32>) -> Self {
        match v.normalize() {
            Vector3 { x: 1., .. } => Direction::East,
            Vector3 { x: -1., .. } => Direction::West,
            Vector3 { y: 1., .. } => Direction::Top,
            Vector3 { y: -1., .. } => Direction::Bottom,
            Vector3 { z: 1., .. } => Direction::North,
            Vector3 { z: -1., .. } | _ => Direction::South,
        }
    }
}
