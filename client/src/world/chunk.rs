use std::collections::HashMap;
use glam::Vec3;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ChunkPosition{
    x: isize,
    y: isize,
    z: isize
}

impl ChunkPosition{
    pub fn new(x: isize, y: isize, z: isize) -> Self{
        Self{
            x,
            y,
            z,
        }
    }

    pub fn to_world(&self) -> Vec3{
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct BlockPosition{
    x: usize,
    y: usize,
    z: usize,
}

pub enum BlockMeta{}

pub const CHUNK_SIZE: usize = 16;
pub struct Chunk{
    blocks: [[[usize; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    meta: HashMap<BlockPosition, BlockMeta>,
}

impl Chunk{
    pub fn new() -> Self{
        let blocks = [[[0; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
        let meta = HashMap::new();

        Self{
            blocks,
            meta,
        }
    }
}
