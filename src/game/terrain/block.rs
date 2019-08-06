use crate::engine::Vertex;
use crate::utils::texture::TextureCoords;

pub struct FaceData{
    position: [u8; 3],
    block_type: BlockType,
    direction: Direction,
}

impl FaceData{
    pub fn new(position: [u8; 3], block_type: BlockType, direction: Direction) -> Self{
        Self{
            position,
            block_type,
            direction
        }
    }

    pub fn get_direction(&self) -> &Direction{
        &self.direction
    }

    pub fn get_position(&self) -> &[u8; 3]{
        &self.position
    }

    pub fn get_type(&self) -> &BlockType{
        &self.block_type
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Direction{
    North,
    South,
    East,
    West,
    Up,
    Down
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BlockType{
    Air = 0,
    Dirt,
    Cobblestone
}

pub const BLOCK_SIZE: f32 = 1.;

pub const HALF: f32 = BLOCK_SIZE/2.;
pub const INDICES: [u32; 6] = [0, 1, 2, 2, 1, 3];

/*
0: Bottom Left
1: Bottom Right
2: Top Left
3: Top Right
*/
pub const NORTH_FACE: [Vertex; 4] = [
    Vertex::new([-HALF, -HALF, HALF], [0., 0., 0.]),
    Vertex::new([ HALF, -HALF, HALF], [0., 0., 0.]),
    Vertex::new([-HALF, HALF, HALF],  [0., 0., 0.]),
    Vertex::new([ HALF, HALF, HALF],  [0., 0., 0.])
];

pub const SOUTH_FACE: [Vertex; 4] = [
    Vertex::new([ HALF, -HALF, -HALF], [0., 0., 0.]),
    Vertex::new([-HALF, -HALF, -HALF], [0., 0., 0.]),
    Vertex::new([ HALF, HALF, -HALF],  [0., 0., 0.]),
    Vertex::new([-HALF, HALF, -HALF],  [0., 0., 0.])
];

pub const WEST_FACE: [Vertex; 4] = [
    Vertex::new([-HALF, -HALF,-HALF], [0., 0., 0.]),
    Vertex::new([-HALF, -HALF, HALF], [0., 0., 0.]),
    Vertex::new([-HALF, HALF, -HALF], [0., 0., 0.]),
    Vertex::new([-HALF, HALF,  HALF], [0., 0., 0.])
];

pub const EAST_FACE: [Vertex; 4] = [
    Vertex::new([HALF, -HALF,  HALF], [0., 0., 0.]),
    Vertex::new([HALF, -HALF, -HALF], [0., 0., 0.]),
    Vertex::new([HALF,  HALF,  HALF], [0., 0., 0.]),
    Vertex::new([HALF,  HALF, -HALF], [0., 0., 0.])
];

pub const UP_FACE: [Vertex; 4] = [
    Vertex::new([-HALF, HALF, HALF], [0., 0., 0.]),
    Vertex::new([ HALF, HALF, HALF], [0., 0., 0.]),
    Vertex::new([-HALF, HALF, -HALF], [0., 0., 0.]),
    Vertex::new([ HALF, HALF, -HALF], [0., 0., 0.])
];

pub const DOWN_FACE: [Vertex; 4] = [
    Vertex::new([-HALF, -HALF, -HALF], [0., 0., 0.]),
    Vertex::new([ HALF, -HALF, -HALF], [0., 0., 0.]),
    Vertex::new([-HALF, -HALF, HALF], [0., 0., 0.]),
    Vertex::new([ HALF, -HALF, HALF], [0., 0., 0.])
];
