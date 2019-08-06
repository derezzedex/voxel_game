use crate::game::terrain::block::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

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
}

pub const CHUNK_SIZE: usize = 16;
pub type ChunkBlocks = [[[BlockType; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

#[derive(Clone, Debug)]
pub struct Chunk{
    blocks: ChunkBlocks,
    // position: ChunkPosition,
    dirty: Arc<AtomicBool>,
}

impl Chunk{
    pub fn new(filler: BlockType) -> Self{
        Self{
            blocks: [[[filler; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
            // position,
            dirty: Arc::new(AtomicBool::new(true)),
        }
    }
    //
    // pub fn get_position(&self) -> &ChunkPosition{
    //     &self.position
    // }

    pub fn get_blocks(&self) -> &ChunkBlocks{
        &self.blocks
    }

    pub fn get_mut_blocks(&mut self) -> &mut ChunkBlocks{
        &mut self.blocks
    }

    pub fn is_dirty(&self) -> bool{
        self.dirty.load(Ordering::Relaxed)
    }

    pub fn flag_dirty(&self){
        self.dirty.store(true, Ordering::Relaxed);
    }

    pub fn flag_clean(&self){
        self.dirty.store(false, Ordering::Relaxed);
    }

    pub fn get_neighbor(&self, x: usize, y: usize, z: usize, dir: Direction, neighbor: Option<Arc<Chunk>>) -> BlockType{
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
