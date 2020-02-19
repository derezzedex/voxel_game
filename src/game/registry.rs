use dashmap::DashMap;
use crate::game::terrain::block::{Direction, BlockData};

pub struct ItemData{
    name: String,
    max_stack: usize,
}

pub struct BlockDataBuilder{
    faces: Option<[[u32; 2]; 6]>,
    breakable: Option<bool>,
    transparent: Option<bool>
}

impl Default for BlockDataBuilder{
    fn default() -> Self{
        Self{
            faces: Some([[0, 0]; 6]),
            breakable: Some(true),
            transparent: Some(false)
        }
    }
}

impl BlockDataBuilder{
    pub fn new() -> Self{
        Self{
            faces: None,
            breakable: None,
            transparent: None
        }
    }

    pub fn faces(mut self, faces: [[u32; 2]; 6]) -> Self{
        self.faces = Some(faces);
        self
    }

    pub fn all_faces(mut self, face: [u32; 2]) -> Self{
        self.faces = Some([face; 6]);
        self
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

    pub fn transparent(mut self, transparent: bool) -> Self{
        self.transparent = Some(transparent);
        self
    }

    pub fn build(self) -> BlockData{
        BlockData::new(self.faces.expect("Missing faces"), self.breakable.expect("Missing breakable"), self.transparent.expect("Missing transparent"))
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

    pub fn by_name(&self, name: &str) -> Option<&BlockData>{
        if let Some(id) = self.ids.get(name){
            return self.by_id(*id);
        }

        return None;
    }
}

pub struct Registry{
    block: BlockRegistry,
}

impl Registry{
    pub fn new() -> Self{
        let block = BlockRegistry::new();
        Self{
            block
        }
    }

    pub fn block_registry_mut(&mut self) -> &mut BlockRegistry{
        &mut self.block
    }

    pub fn block_registry(&self) -> &BlockRegistry{
        &self.block
    }
}
