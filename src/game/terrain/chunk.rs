pub const SIZE: usize = 16;
#[derive(Debug)]
pub struct Chunk {
    blocks: [[[usize; SIZE]; SIZE]; SIZE],
}

impl Chunk {
    pub fn new(filler: usize) -> Self {
        Self {
            blocks: [[[filler; SIZE]; SIZE]; SIZE],
        }
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: usize) {
        self.blocks[x][y][z] = block;
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> usize {
        if x >= SIZE || y >= SIZE || z >= SIZE {
            panic!("Error: {:?}", [x, y, z])
        }

        self.blocks[x][y][z]
    }
}
