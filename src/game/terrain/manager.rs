use dashmap::Iter;
use crate::utils::texture::TextureAtlas;
use cgmath::{Point3, Vector3};
use noise::{NoiseFn, Perlin, Seedable};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::engine::mesh::*;
use crate::game::terrain::block::*;
use crate::game::terrain::block_registry::BlockRegistry;
use crate::game::terrain::chunk::*;
use crate::game::terrain::chunk_mesher::ChunkMesher;
use crate::game::terrain::chunk_manager::{ChunkManager, ChunkRef, ChunkRefMut};

use rayon::iter::ParallelBridge;
use rayon::prelude::*;

pub struct TerrainManager {
    mesher: ChunkMesher,
    manager: ChunkManager,
    position: ChunkPosition,
    noise: Perlin,
}

impl TerrainManager {
    pub fn new(position: ChunkPosition) -> Self {
        let mesher = ChunkMesher::new(1);
        let manager = ChunkManager::new(4);

        let noise = Perlin::new().set_seed(1102130);

        Self {
            mesher,
            manager,
            position,
            noise,
        }
    }

    pub fn test(&mut self, x: isize, y: isize, z: isize){
        let timer = Instant::now();

        let n = 20;

        for i in -n..n{
            let position = ChunkPosition::new(i, i, i);
            self.manager.create_chunk(position);
        }
        println!("Add time: {:?}", timer.elapsed());
        let check_timer = Instant::now();

        for i in -n..n{
            let position = ChunkPosition::new(i, i, i);
            self.manager.get_chunk(position).expect("Chunk not found!");
        }
        println!("Check time: {:?}", check_timer.elapsed());

        println!("Done!");
        println!("Total time: {:?}", timer.elapsed());
    }

    pub fn update_received_meshes(&self, display: &glium::Display){
        self.mesher.update_available_meshes(display);
    }

    pub fn get_meshes<'a>(&'a self) -> Iter<'a, ChunkPosition, Mesh>{
        self.mesher.get_meshes_iter()
    }

    pub fn mesh_dirty_chunks(&mut self, atlas: &TextureAtlas, registry: &BlockRegistry) {
        for chunk_ref in self.manager.get_chunks(){
            let (pos, chunk) = (chunk_ref.key(), chunk_ref.value());
            if chunk.is_dirty() {
                let neighbors = self.manager.get_neighbors(*pos);
                self.mesher.mesh(*pos, chunk, neighbors, atlas, registry);
            }
        }
    }

    pub fn update_chunks(&mut self, position: ChunkPosition, view_distance: isize) {
        /*

            REDO EVERYTHING USING EVMAP

            TODO: Do 1 loop through all chunks in neighboring area [(position.x - view_distance)..(position.x + view_distance)]

            1. Check if position is in bounds
            2. Check if already exists
                2.1 If dont exists, create
                2.2 If in bounds, update
                2.3 If not in bounds, delete
            3. Do a final retain on the HashMap


            TODO: Make all Update Operations ASYNC [ Creation, Remotion and Ticking ]
        */

        let timer = Instant::now();
        for x in (position.x - view_distance)..(position.x + view_distance) {
            for y in (position.y - view_distance)..(position.y + view_distance) {
                for z in (position.z - view_distance)..(position.z + view_distance) {

                    let chunk_pos = ChunkPosition::new(x, y, z);
                    let in_bounds = (x > position.x - view_distance
                        && y > position.y - view_distance
                        && z > position.z - view_distance
                        && x < position.x + view_distance
                        && y < position.y + view_distance
                        && z < position.z + view_distance);

                    if let Some(chunk) = self.manager.get_mut_chunk(chunk_pos){
                        if in_bounds{
                            //update
                        }else{
                            self.manager.remove_chunk(chunk_pos);
                        }
                        continue;
                    }
                    if in_bounds{
                        self.manager.create_chunk(chunk_pos);
                    }
                }
            }
        }
    }
}
