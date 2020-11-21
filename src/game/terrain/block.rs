#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    East = 0,
    West = 1,
    Top = 2,
    Bottom = 3,
    North = 4,
    South = 5,
}

pub type AtlasCoord = [u32; 2];
#[allow(dead_code)]
pub struct BlockData {
    faces: [AtlasCoord; 6],
    breakable: bool,
    transparent: bool,
}

impl BlockData {
    pub fn new(faces: [AtlasCoord; 6], breakable: bool, transparent: bool) -> Self {
        Self {
            faces,
            breakable,
            transparent,
        }
    }

    pub fn get_face(&self, dir: Direction) -> AtlasCoord {
        self.faces[dir as usize]
    }
}
