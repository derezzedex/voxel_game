use engine::Direction;

#[allow(dead_code)]
#[derive(Default)]
pub struct BlockData{
    faces: [[u32; 2]; 6],
    mesh: usize,
    breakable: bool,
    transparent: bool
}

impl BlockData{
    pub fn new(faces: [[u32; 2]; 6], mesh: usize, breakable: bool, transparent: bool) -> Self{
        Self{
            faces,
            mesh,
            breakable,
            transparent
        }
    }

    pub fn get_mesh(&self) -> usize{
        self.mesh
    }

    pub fn get_face(&self, dir: Direction) -> [u32; 2]{
        self.faces[dir as usize]
    }

    pub fn is_transparent(&self) -> bool{
        self.transparent
    }
}
