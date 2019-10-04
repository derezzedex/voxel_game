use cgmath::Vector3;
use crate::game::terrain::block::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub const CHUNK_SIZE: usize = 16;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct BlockPosition{
    pub x: isize,
    pub y: isize,
    pub z: isize
}

impl BlockPosition{
    pub fn new(x: isize, y: isize, z: isize) -> Self{
        Self{
            x,
            y,
            z
        }
    }

    pub fn to_chunk(&self) -> ChunkPosition{
        let chunk_size = CHUNK_SIZE as isize;
        let x = self.x / chunk_size;
        let y = self.y / chunk_size;
        let z = self.z / chunk_size;

        ChunkPosition{
            x,
            y,
            z
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct ChunkPosition{
    pub x: isize,
    pub y: isize,
    pub z: isize
}

impl ChunkPosition{
    pub fn new(x: isize, y: isize, z: isize) -> Self{
        Self{
            x,
            y,
            z
        }
    }

    pub fn get_neighbors(&self) -> [ChunkPosition; 6]{
        let north = ChunkPosition::new(self.x, self.y, self.z + 1);
        let south = ChunkPosition::new(self.x, self.y, self.z - 1);

        let east = ChunkPosition::new(self.x + 1, self.y, self.z);
        let west = ChunkPosition::new(self.x - 1, self.y, self.z);

        let up = ChunkPosition::new(self.x, self.y + 1, self.z);
        let down = ChunkPosition::new(self.x, self.y - 1, self.z);

        [north, south, east, west, up, down]
    }

    pub fn to_block(&self) -> BlockPosition{
        let chunk_size = CHUNK_SIZE as isize;
        let x = ((self.x % chunk_size) + chunk_size) % chunk_size;
        let y = ((self.y % chunk_size) + chunk_size) % chunk_size;
        let z = ((self.z % chunk_size) + chunk_size) % chunk_size;

        BlockPosition{
            x,
            y,
            z
        }
    }
}

pub type ChunkBlocks = [[[BlockType; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Chunk{
    blocks: ChunkBlocks,
    dirty: bool,
}

impl Chunk{
    pub fn new(filler: BlockType) -> Self{
        Self{
            blocks: [[[filler; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
            dirty: true,
        }
    }

    pub fn new_air() -> Self{
        Chunk::new(BlockType::Air)
    }

    pub fn remove_block(&mut self, x: usize, y: usize, z: usize){
        self.blocks[x][y][z] = BlockType::Air;
        self.flag_dirty();
    }

    pub fn place_block(&mut self, x: usize, y: usize, z: usize, block: BlockType){
        self.blocks[x][y][z] = block;
        self.flag_dirty();
    }

    pub fn get_blocks(&self) -> &ChunkBlocks{
        &self.blocks
    }

    pub fn get_mut_blocks(&mut self) -> &mut ChunkBlocks{
        &mut self.blocks
    }

    pub fn is_dirty(&self) -> bool{
        self.dirty
    }

    pub fn flag_dirty(&mut self){
        self.dirty = true;
    }

    pub fn flag_clean(&mut self){
        self.dirty = false;
    }

    pub fn get_neighbor(&self, x: usize, y: usize, z: usize, dir: Direction, neighbor: Option<&Arc<Chunk>>) -> BlockType{
        match dir{
            Direction::North => {
                if (z + 1 >= CHUNK_SIZE){
                    match neighbor{
                        Some(chunk) => return chunk.get_blocks()[x][y][0],
                        None => return BlockType::Air
                    }
                }else{
                    return self.blocks[x][y][z + 1]
                }
            },
            Direction::South => {
                if (z as isize - 1 < 0){
                    match neighbor{
                        Some(chunk) => return chunk.get_blocks()[x][y][CHUNK_SIZE-1],
                        None => return BlockType::Air
                    }
                }else{
                    return self.blocks[x][y][z - 1]
                }
            },
            Direction::East => {
                if (x + 1 >= CHUNK_SIZE){
                    match neighbor{
                        Some(chunk) => return chunk.get_blocks()[0][y][z],
                        None => return BlockType::Air
                    }
                }else{
                    return self.blocks[x + 1][y][z]
                }
            },
            Direction::West => {
                if (x as isize - 1 < 0){
                    match neighbor{
                        Some(chunk) => return chunk.get_blocks()[CHUNK_SIZE-1][y][z],
                        None => return BlockType::Air
                    }
                }else{
                    return self.blocks[x - 1][y][z]
                }
            },
            Direction::Up => {
                if (y + 1 >= CHUNK_SIZE){
                    match neighbor{
                        Some(chunk) => return chunk.get_blocks()[x][0][z],
                        None => {
                            // println!("\tB[{}][{}][{}][{:?}]", x, y, z, dir);
                            return BlockType::Air
                        }
                    }
                }else{
                    return self.blocks[x][y + 1][z]
                }
            },
            Direction::Down => {
                if (y as isize - 1 < 0){
                    match neighbor{
                        Some(chunk) => return chunk.get_blocks()[x][CHUNK_SIZE-1][z],
                        None => return BlockType::Air
                    }
                }else{
                    return self.blocks[x][y - 1][z]
                }
            },
        }
    }
}
