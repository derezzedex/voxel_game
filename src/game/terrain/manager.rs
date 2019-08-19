use std::sync::Arc;
use std::collections::HashMap;
use cgmath::{Point3, Vector3};

use crate::game::terrain::block::*;
use crate::game::terrain::chunk::*;
use crate::engine::mesh::*;
use crate::game::terrain::chunk_mesher::ChunkMesher;

pub type ChunkHashMap = HashMap<ChunkPosition, Arc<Chunk>>;
pub type MeshHashMap = HashMap<ChunkPosition, Mesh>;
pub struct TerrainManager{
    mesher: ChunkMesher,
    chunks: ChunkHashMap,
    meshes: MeshHashMap
}

impl TerrainManager{
    pub fn new() -> Self{
        let mesher = ChunkMesher::new(1);
        let meshes = HashMap::new();
        let chunks = HashMap::new();

        Self{
            mesher,
            chunks,
            meshes
        }
    }

    pub fn block_at(&self, position: Point3<f64>) -> bool{
        let chunk_position = ChunkPosition::new(position.x as isize >> 4, position.y as isize >> 4, position.z as isize >> 4);
        if let Some(chunk) = self.chunks.get(&chunk_position){
            let block_position = BlockPosition::new(position.x as isize, position.y as isize, position.z as isize).get_offset();
            let (x, y, z) = (block_position.x as usize, block_position.y as usize, block_position.z as usize);

            return chunk.get_blocks()[x as usize][y as usize][z as usize] != BlockType::Air;
        }

        false
    }

    pub fn place_block(&mut self, position: Point3<f64>, face: Vector3<f64>){
        let block = self.block_at(position);
    }

    pub fn get_chunk(&self, position: ChunkPosition) -> Option<&Arc<Chunk>>{
        self.chunks.get(&position)
    }

    pub fn get_mut_chunk(&mut self, position: ChunkPosition) -> Option<&mut Arc<Chunk>>{
        self.chunks.get_mut(&position)
    }

    fn get_neighbors(&self, position: ChunkPosition) -> [Option<&Arc<Chunk>>; 6]{
        let north = self.get_chunk(ChunkPosition::new(position.x, position.y, position.z + 1));
        let south = self.get_chunk(ChunkPosition::new(position.x, position.y, position.z - 1));

        let east = self.get_chunk(ChunkPosition::new(position.x + 1, position.y, position.z));
        let west = self.get_chunk(ChunkPosition::new(position.x - 1, position.y, position.z));

        let up = self.get_chunk(ChunkPosition::new(position.x, position.y + 1, position.z));
        let down = self.get_chunk(ChunkPosition::new(position.x, position.y - 1, position.z));

        [north, south, east, west, up, down]
    }

    fn get_neighbors_position(&self, position: ChunkPosition) -> [ChunkPosition; 6]{
        let north = ChunkPosition::new(position.x, position.y, position.z + 1);
        let south = ChunkPosition::new(position.x, position.y, position.z - 1);

        let east = ChunkPosition::new(position.x + 1, position.y, position.z);
        let west = ChunkPosition::new(position.x - 1, position.y, position.z);

        let up = ChunkPosition::new(position.x, position.y + 1, position.z);
        let down = ChunkPosition::new(position.x, position.y - 1, position.z);

        [north, south, east, west, up, down]
    }

    fn dirty_neighbors(&mut self, position: ChunkPosition){
        let mut neighbors = self.get_neighbors_position(position);
        for neighbor_position in neighbors.iter(){
            if let Some(chunk) = self.chunks.get(neighbor_position){
                chunk.flag_dirty();
            }
        }
    }

    pub fn add_chunk(&mut self, position: ChunkPosition, chunk: Arc<Chunk>){
        self.dirty_neighbors(position);
        self.chunks.insert(position, chunk);
    }

    pub fn create_chunk_at(&mut self, position: [isize; 3]){
        let position = ChunkPosition::new(position[0], position[1], position[2]);
        let chunk = Chunk::new(BlockType::Dirt);
        self.add_chunk(position, Arc::new(chunk));
    }

    pub fn mesh_dirty_chunks(&mut self){
        for (pos, chunk) in &self.chunks{
            if chunk.is_dirty(){
                let neighbors = self.get_neighbors(*pos);
                self.mesh(*pos, chunk, neighbors);

                chunk.flag_clean();
            }
        }
    }

    fn mesh(&self, position: ChunkPosition, chunk: &Arc<Chunk>, neighbors: [Option<&Arc<Chunk>>; 6]){
        self.mesher.mesh(position, chunk, neighbors);
    }

    // pub fn update_chunk(&mut self, position: ChunkPosition, chunk: &mut Chunk){
    //     self.chunks.entry(position).and_modify(|c| c = chunk);
    // }

    pub fn update_received_meshes(&mut self, display: &glium::Display){
        // let mut flagged_clean = Vec::new();
        for (pos, mesh) in self.mesher.get_available_meshes(){
            let built_mesh = mesh.build(display);
            if self.meshes.contains_key(&pos){
                self.meshes.entry(pos).and_modify(|mesh| { *mesh = built_mesh });
            }else{
                self.meshes.insert(pos, built_mesh);
            }

        }

    }

    pub fn get_meshes(&self) -> &MeshHashMap{
        &self.meshes
    }
}
