use num_enum::TryFromPrimitive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(usize)]
pub enum Direction{
    East = 0,
    West = 1,
    Top = 2,
    Bottom = 3,
    North = 4,
    South = 5
}

#[allow(dead_code)]
#[derive(Default)]
pub struct BlockData{
    faces: [[u32; 2]; 6],
    breakable: bool,
    transparent: bool
}

impl BlockData{
    pub fn new(faces: [[u32; 2]; 6], breakable: bool, transparent: bool) -> Self{
        Self{
            faces,
            breakable,
            transparent
        }
    }

    pub fn get_face(&self, dir: Direction) -> [u32; 2]{
        self.faces[dir as usize]
    }

    pub fn is_transparent(&self) -> bool{
        self.transparent
    }
}
