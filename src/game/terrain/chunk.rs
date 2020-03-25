use cgmath::Point3;
use std::sync::Arc;

pub trait FromWorld{
    fn from_world(x: f32, y: f32, z: f32) -> Self;
}

pub type ChunkPosition = Point3<isize>;
impl FromWorld for ChunkPosition{
    fn from_world(x: f32, y: f32, z: f32) -> ChunkPosition{
        ChunkPosition::new((x / (CHUNKSIZE) as f32).floor() as isize, (y / (CHUNKSIZE) as f32).floor() as isize, (z / (CHUNKSIZE) as f32).floor() as isize)
    }
}

pub const CHUNKSIZE: usize = 16;
#[derive(Debug, Clone)]
pub struct Chunk{
    blocks: [[[usize; CHUNKSIZE]; CHUNKSIZE]; CHUNKSIZE],
}

impl Chunk{
    pub fn new(filler: usize) -> Self{
        Self{
            blocks: [[[filler; CHUNKSIZE]; CHUNKSIZE]; CHUNKSIZE],
        }
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: usize){
        self.blocks[x][y][z] = block;
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> usize{
        if x >= CHUNKSIZE { panic!("Accessing invalid block: {:?}", [x, y, z])}
        if y >= CHUNKSIZE { panic!("Accessing invalid block: {:?}", [x, y, z])}
        if z >= CHUNKSIZE { panic!("Accessing invalid block: {:?}", [x, y, z])}
        self.blocks[x][y][z]
    }

    pub fn check_block(&self, x: isize, y: isize, z: isize, neighbors: Vec<Option<Arc<Chunk>>>) -> usize{
        if x >= CHUNKSIZE as isize {
            match &neighbors[0]{
                Some(chunk) => return chunk.get_block(0, y as usize, z as usize),
                None => return 0,
            }
        }
        if x < 0{
            match &neighbors[1]{
                Some(chunk) => return chunk.get_block(CHUNKSIZE-1, y as usize, z as usize),
                None => return 0,
            }
        }

        if y >= CHUNKSIZE as isize {
            match &neighbors[2]{
                Some(chunk) => return chunk.get_block(x as usize, 0, z as usize),
                None => return 0,
            }
        }
        if y < 0{
            match &neighbors[3]{
                Some(chunk) => return chunk.get_block(x as usize, CHUNKSIZE-1, z as usize),
                None => return 0,
            }
        }

        if z >= CHUNKSIZE as isize {
            match &neighbors[4]{
                Some(chunk) => return chunk.get_block(x as usize, y as usize, 0),
                None => return 0,
            }
        }
        if z < 0{
            match &neighbors[5]{
                Some(chunk) => return chunk.get_block(x as usize, y as usize, CHUNKSIZE-1),
                None => return 0,
            }
        }
        self.get_block(x as usize, y as usize, z as usize)
    }
}
