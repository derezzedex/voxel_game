use dashmap::DashMap;
pub use crate::terrain::block::{BlockData};
pub use engine::Direction;

pub struct BlockDataBuilder{
    faces: Option<[[u32; 2]; 6]>,
    mesh: Option<usize>,
    breakable: Option<bool>,
    transparent: Option<bool>
}

impl Default for BlockDataBuilder{
    fn default() -> Self{
        Self{
            faces: Some([[0, 0]; 6]),
            mesh: Some(0),
            breakable: Some(true),
            transparent: Some(false)
        }
    }
}

impl BlockDataBuilder{
    pub fn faces(mut self, faces: [[u32; 2]; 6]) -> Self{
        self.faces = Some(faces);
        self
    }

    pub fn mesh(mut self, id: usize) -> Self{
        self.mesh = Some(id);
        self
    }

    pub fn all_faces(self, face: [u32; 2]) -> Self{
        self.faces([face; 6])
    }

    pub fn face(mut self, dir: Direction, face: [u32; 2]) -> Self{
        if let Some(mut faces) = self.faces{
            faces[dir as usize] = face;
            self.faces = Some(faces);
        }else{
            let mut faces = [[0, 0]; 6];
            faces[dir as usize] = face;
            self.faces = Some(faces);
        }
        self
    }

    pub fn breakable(mut self, breakable: bool) -> Self{
        self.breakable = Some(breakable);
        self
    }

    #[allow(dead_code)]
    pub fn transparent(mut self, transparent: bool) -> Self{
        self.transparent = Some(transparent);
        self
    }

    pub fn build(self) -> BlockData{
        BlockData::new(self.faces.expect("Missing faces"), self.mesh.expect("Missing mesh id"), self.breakable.expect("Missing breakable"), self.transparent.expect("Missing transparent"))
    }
}

pub struct BlockRegistry{
    ids: DashMap<String, usize>,
    blocks: Vec<BlockData>,
}

impl BlockRegistry{
    pub fn new() -> Self{
        let ids = DashMap::default();
        let blocks = Vec::new();

        Self{
            ids,
            blocks
        }
    }

    pub fn add(&mut self, name: &str, data: BlockData){
        self.blocks.push(data);
        let id = self.blocks.len() - 1;
        self.ids.insert(String::from(name), id);
    }

    pub fn id_of(&self, name: &str) -> Option<usize>{
        if let Some(id_ref) = self.ids.get(name){
            return Some(*id_ref.value());
        }

        return None;
    }

    pub fn by_id(&self, id: usize) -> Option<&BlockData>{
        self.blocks.get(id)
    }

    #[allow(dead_code)]
    pub fn by_name(&self, name: &str) -> Option<&BlockData>{
        if let Some(id) = self.ids.get(name){
            return self.by_id(*id);
        }

        return None;
    }
}
