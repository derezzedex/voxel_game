use cgmath::Point3;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction{
    North = 0,
    South,
    East,
    West,
    Top,
    Bottom
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockType{
    Air,
    Dirt,
    Cobblestone
}

pub type ChunkPosition = Point3<isize>;

pub const CHUNKSIZE: usize = 16;
// #[derive(Debug)]
pub struct Chunk{
    blocks: [[[BlockType; CHUNKSIZE]; CHUNKSIZE]; CHUNKSIZE],
}

impl Chunk{
    pub fn new(filler: BlockType) -> Self{
        Self{
            blocks: [[[filler; CHUNKSIZE]; CHUNKSIZE]; CHUNKSIZE],
        }
    }
    //
    // pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType{
    //     if x >= CHUNKSIZE || y >= CHUNKSIZE || z >= CHUNKSIZE{
    //         return BlockType::Air;
    //     }
    //     self.blocks[x][y][z]
    // }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockType){
        self.blocks[x][y][z] = block;
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType{
        self.blocks[x][y][z]
    }

    pub fn check_block(&self, x: isize, y: isize, z: isize, neighbors: Vec<Option<Arc<Chunk>>>) -> BlockType{
        if x >= CHUNKSIZE as isize {
            // println!("Block: {:?}", (x, y, z));
            match &neighbors[0]{
                Some(chunk) => return chunk.get_block(0, y as usize, z as usize),
                None => return BlockType::Air,
            }
        }else if x < 0{
            match &neighbors[1]{
                Some(chunk) => return chunk.get_block(CHUNKSIZE-1, y as usize, z as usize),
                None => return BlockType::Air,
            }
        }

        if y >= CHUNKSIZE as isize {
            match &neighbors[2]{
                Some(chunk) => return chunk.get_block(x as usize, 0, z as usize),
                None => return BlockType::Air,
            }
        }else if y < 0{
            match &neighbors[3]{
                Some(chunk) => return chunk.get_block(x as usize, CHUNKSIZE-1, z as usize),
                None => return BlockType::Air,
            }
        }

        if z >= CHUNKSIZE as isize {
            match &neighbors[4]{
                Some(chunk) => return chunk.get_block(x as usize, y as usize, 0),
                None => return BlockType::Air,
            }
        }else if z < 0{
            match &neighbors[5]{
                Some(chunk) => return chunk.get_block(x as usize, y as usize, CHUNKSIZE-1),
                None => return BlockType::Air,
            }
        }
        self.blocks[x as usize][y as usize][z as usize]
    }
}
