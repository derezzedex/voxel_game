use dashmap::DashMap;
use crate::game::terrain::block::{Direction, BlockData};

// pub struct ItemData{
//     name: String,
//     max_stack: usize,
// }

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
    pub fn faces(mut self, faces: [[u32; 2]; 6]) -> Self{
        self.faces = Some(faces);
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

    #[allow(dead_code)]
    pub fn by_name(&self, name: &str) -> Option<&BlockData>{
        if let Some(id) = self.ids.get(name){
            return self.by_id(*id);
        }

        return None;
    }
}

pub struct Registry{
    blocks: BlockRegistry,
}

impl Registry{
    pub fn new() -> Self{
        let blocks = BlockRegistry::new();
        Self{
            blocks
        }
    }

    pub fn setup(&mut self){
        let air = BlockDataBuilder::default().all_faces([0, 1]).transparent(true).build();
        self.blocks.add("air", air);

        let missing = BlockDataBuilder::default().all_faces([0, 1]).build();
        self.blocks.add("missing", missing);

        let grass = BlockDataBuilder::default()
            .all_faces([3, 15])
            .face(Direction::Top, [0, 15])
            .face(Direction::Bottom, [2, 15])
            .build();
        self.blocks.add("grass", grass);

        let dirt = BlockDataBuilder::default().all_faces([2, 15]).build();
        self.blocks.add("dirt", dirt);

        let stone = BlockDataBuilder::default().all_faces([1, 15]).build();
        self.blocks.add("stone", stone);

        let bedrock = BlockDataBuilder::default()
            .all_faces([1, 14])
            .breakable(false)
            .build();
        self.blocks.add("bedrock", bedrock);

        let glass = BlockDataBuilder::default()
            .all_faces([0, 14])
            .breakable(false)
            .transparent(true)
            .build();
        self.blocks.add("glass", glass);
    }

    pub fn block_registry(&self) -> &BlockRegistry{
        &self.blocks
    }
}
