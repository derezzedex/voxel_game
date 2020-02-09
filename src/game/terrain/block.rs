use crate::engine::Vertex;
use crate::utils::texture::TextureCoords;

pub struct FaceData{
    position: [u8; 3],
    block_type: BlockType,
    direction: Direction,
    coords: TextureCoords
}

impl FaceData{
    pub fn new(position: [u8; 3], block_type: BlockType, direction: Direction, coords: TextureCoords) -> Self{
        Self{
            position,
            block_type,
            direction,
            coords
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

    pub fn get_coordinates(&self) -> &TextureCoords{
        &self.coords
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Direction{
    North = 0,
    South,
    East,
    West,
    Up,
    Down
}

impl From<Direction> for [f32;3]{
    fn from(item: Direction) -> Self{
        match item{
            Direction::North => { [ 0., 0., 1.] },
            Direction::South => { [ 0., 0., -1.] },
            Direction::East=>   { [ 1., 0., 0.] },
            Direction::West =>  { [ -1., 0., 0.] },
            Direction::Up =>    { [ 0., 1., 0.] },
            Direction::Down =>  { [ 0., -1., 0.] }
        }
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
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
