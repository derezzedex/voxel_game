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

pub struct BlockDataBuilder {
    faces: Option<[[u32; 2]; 6]>,
    breakable: Option<bool>,
    transparent: Option<bool>,
}

impl Default for BlockDataBuilder {
    fn default() -> Self {
        Self {
            faces: Some([[0, 0]; 6]),
            breakable: Some(true),
            transparent: Some(false),
        }
    }
}

impl BlockDataBuilder {
    pub fn faces(mut self, faces: [[u32; 2]; 6]) -> Self {
        self.faces = Some(faces);
        self
    }

    pub fn all_faces(self, face: [u32; 2]) -> Self {
        self.faces([face; 6])
    }

    pub fn face(mut self, dir: Direction, face: [u32; 2]) -> Self {
        if let Some(mut faces) = self.faces {
            faces[dir as usize] = face;
            self.faces = Some(faces);
        } else {
            let mut faces = [[0, 0]; 6];
            faces[dir as usize] = face;
            self.faces = Some(faces);
        }
        self
    }

    pub fn breakable(mut self, breakable: bool) -> Self {
        self.breakable = Some(breakable);
        self
    }

    #[allow(dead_code)]
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.transparent = Some(transparent);
        self
    }

    pub fn build(self) -> BlockData {
        BlockData::new(
            self.faces.expect("Missing faces"),
            self.breakable.expect("Missing breakable"),
            self.transparent.expect("Missing transparent"),
        )
    }
}