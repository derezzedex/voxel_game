use cgmath::Point3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockType{
    Air,
    Dirt,
    Cobblestone
}

pub type ChunkPosition = Point3<isize>;

pub const CHUNKSIZE: usize = 16;
pub struct Chunk{
    blocks: [[[BlockType; CHUNKSIZE]; CHUNKSIZE]; CHUNKSIZE],
}

impl Chunk{
    pub fn new(filler: BlockType) -> Self{
        Self{
            blocks: [[[filler; CHUNKSIZE]; CHUNKSIZE]; CHUNKSIZE],
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType{
        if x >= CHUNKSIZE || y >= CHUNKSIZE || z >= CHUNKSIZE{
            return BlockType::Air;
        }
        self.blocks[x][y][z]
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockType){
        self.blocks[x][y][z] = block;
    }
}
