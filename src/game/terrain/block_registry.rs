use crate::utils::texture::TextureCoords;
use std::collections::HashMap;
use crate::game::terrain::block::{BlockType, Direction};

const DEFAULT_GAME_NAME: &str = "voxel_game";

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct BlockId{
    id: String,
}

impl BlockId{
    pub fn new(block_name: &str) -> Self{
        let mut id = String::from(DEFAULT_GAME_NAME);
        id.push_str(":");
        id.push_str(block_name);
        Self{
            id
        }
    }
}

pub struct BlockDataBuilder{
    block_name: BlockId,
    coordinates: Option<Vec<TextureCoords>>,
    orientation: Option<Direction>
}

impl BlockDataBuilder{
    pub fn new(name: &str) -> Self{
        Self{
            block_name: BlockId::new(name),
            coordinates: None,
            orientation: None
        }
    }

    fn add_cordinate(&mut self, dir: Direction, coord: TextureCoords){
        let mut buffer = vec![coord; 6];
        if let Some(coords) = &mut self.coordinates{
            let dir = dir as usize;
            if dir <= coords.len(){
                coords[dir] = coord;
            }
        }else{
            self.coordinates = Some(buffer);
        }
    }

    pub fn orientation(mut self, ori: Direction) -> Self{
        self.orientation = Some(ori);
        self
    }

    pub fn top_face(mut self, coords: TextureCoords) -> Self{
        self.add_cordinate(Direction::Up, coords);
        self
    }

    pub fn bottom_face(mut self, coords: TextureCoords) -> Self{
        self.add_cordinate(Direction::Down, coords);
        self
    }

    pub fn east_face(mut self, coords: TextureCoords) -> Self{
        self.add_cordinate(Direction::East, coords);
        self
    }

    pub fn west_face(mut self, coords: TextureCoords) -> Self{
        self.add_cordinate(Direction::West, coords);
        self
    }

    pub fn north_face(mut self, coords: TextureCoords) -> Self{
        self.add_cordinate(Direction::North, coords);
        self
    }

    pub fn south_face(mut self, coords: TextureCoords) -> Self{
        self.add_cordinate(Direction::South, coords);
        self
    }

    pub fn build(mut self) -> BlockData{
        BlockData::new(self.block_name, self.coordinates.expect("texture coordinates not found"), self.orientation.expect("Needs orientation!"))
    }
}

pub struct BlockData{
    block_name: BlockId,
    tex_coord: Vec<TextureCoords>,
    ori: Direction
}

impl BlockData{
    pub fn new(block_name: BlockId, tex_coord: Vec<TextureCoords>, ori: Direction) -> Self{
        Self{
            tex_coord,
            block_name,
            ori
        }
    }

    pub fn get_coords(&self, ori: Direction) -> &TextureCoords{
        let dir = ori as usize;
        if dir <= self.tex_coord.len(){
            &self.tex_coord[ori as usize]
        }else{
            &self.tex_coord[0]
        }
    }
}

pub struct BlockRegistry{
    blocks: HashMap<BlockType, BlockData>,
}

impl BlockRegistry{
    pub fn new() -> Self{
        Self{
            blocks: HashMap::new(),
        }
    }

    pub fn add_block(&mut self, name: BlockType, data: BlockData){
        self.blocks.insert(name, data);
    }

    pub fn get_block(&self, name: BlockType) -> Option<&BlockData>{
        self.blocks.get(&name)
    }
}
