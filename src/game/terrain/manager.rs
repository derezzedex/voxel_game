use dashmap::Iter;
use crate::utils::texture::TextureAtlas;
use cgmath::{Point3, Vector3};
use noise::{NoiseFn, Perlin, Seedable};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Instant, Duration};

use crate::engine::mesh::*;
use crate::game::terrain::block::*;
use crate::game::terrain::block_registry::BlockRegistry;
use crate::game::terrain::chunk::*;
use crate::game::terrain::chunk_mesher::ChunkMesher;
use crate::game::terrain::chunk_manager::{ChunkManager, ChunkRef, ChunkRefMut};

use rayon::iter::ParallelBridge;
use rayon::prelude::*;

pub type ChunkIter<'a> = Iter<'a, ChunkPosition, Arc<Chunk>>;

pub struct TerrainManager {
    mesher: ChunkMesher,
    manager: ChunkManager,
    position: ChunkPosition,
    noise: Perlin,
}

impl TerrainManager {
    pub fn new(position: ChunkPosition) -> Self {
        let mesher = ChunkMesher::new(1);
        let manager = ChunkManager::new(1);

        let noise = Perlin::new().set_seed(1102130);

        Self {
            mesher,
            manager,
            position,
            noise,
        }
    }

    pub fn test(&mut self, x: isize, y: isize, z: isize, atlas: &TextureAtlas, registry: &BlockRegistry, display: &glium::Display){
        let timer = Instant::now();

        let n = 10;
        println!("Chunks: {:?}", n);

        // ---------------- CREATE CHUNKS ----------------
        for i in -n..n{
            let position = ChunkPosition::new(i, i, i);
            self.manager.async_create_chunk(position);
        }
        println!("Async create time: {:?}", timer.elapsed());

        // Wait a bit...
        std::thread::sleep_ms(5000);

        let addition_timer = Instant::now();
        let mut counter = 0;
        self.manager.update_chunk_queue();
        while self.manager.chunk_queue_number() > 0{
            self.manager.dequeue_chunk();
            self.manager.update_chunk_queue();
            counter += 1;
        }
        println!("Dequeued {} chunks in {:?}", counter, addition_timer.elapsed());

        let check_timer = Instant::now();
        for i in -n..n{
            let position = ChunkPosition::new(i, i, i);
            self.manager.get_chunk(position).expect("Chunk not found!");
        }
        println!("Check time: {:?}", check_timer.elapsed());
        println!();

        // ---------------- MESH CHUNKS ----------------
        let mesh_timer = Instant::now();
        self.mesh_dirty_chunks(atlas, registry);
        println!("Async mesh time: {:?}", mesh_timer.elapsed());

        // Wait a bit...
        std::thread::sleep_ms(5000);

        let mesher_timer = Instant::now();
        let mut counter = 0;
        self.mesher.update_mesh_queue();
        while self.mesher.mesh_queue_number() > 0{
            self.mesher.dequeue_mesh(display);
            self.mesher.update_mesh_queue();
            counter += 1;
        }
        println!("Dequeued {} meshes in {:?}", counter, mesher_timer.elapsed());
        println!();

        // ---------------- REMOVE CHUNKS ----------------
        let remove_timer = Instant::now();
        for i in -n..n{
            let position = ChunkPosition::new(i, i, i);
            self.manager.async_remove_chunk(position);
        }
        println!("Async remove time: {:?}", remove_timer.elapsed());

        // Wait a bit...
        std::thread::sleep_ms(5000);

        let remotion_timer = Instant::now();
        let mut counter = 0;
        self.manager.update_chunk_queue();
        while self.manager.chunk_queue_number() > 0{
            self.manager.update_chunk_queue();
            self.manager.dequeue_chunk();
            counter += 1;
        }
        println!("Removed {} chunks in {:?}", counter, remotion_timer.elapsed());
        println!();

        println!("Done!");
        println!("Total time: {:?}", timer.elapsed() - Duration::from_secs(15));
    }

    pub fn update_received_meshes(&mut self, display: &glium::Display){
        self.mesher.update_mesh_queue();
        let mesher_timer = Instant::now();
        while mesher_timer.elapsed() < Duration::from_millis(1) && self.mesher.mesh_queue_number() != 0{
            self.mesher.dequeue_mesh(display);
            // self.manager.update_chunk_queue();
        }
    }

    pub fn get_meshes<'a>(&'a self) -> Iter<'a, ChunkPosition, Mesh>{
        self.mesher.get_meshes_iter()
    }

    pub fn queue_number(&self) -> usize{
        self.manager.chunk_queue_number()
    }
    
    pub fn mesh_dirty_chunks(&mut self, atlas: &TextureAtlas, registry: &BlockRegistry) {
        self.mesher.mesh_chunks(&mut self.manager, atlas, registry);
        // for chunk_ref in self.manager.get_chunks(){
        //     let (pos, chunk) = (chunk_ref.key(), chunk_ref.value());
        //     if chunk.is_dirty() {
        //         let neighbors = self.manager.get_neighbors(*pos);
        //         self.mesher.mesh(*pos, chunk, neighbors, atlas, registry);
        //     }
        // }
    }

    pub fn update_chunks(&mut self, position: ChunkPosition, view_distance: isize) {
        // let timer = Instant::now();
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

                    if !self.manager.chunk_exists(chunk_pos) && in_bounds{
                        self.manager.async_create_chunk(chunk_pos);
                    }else{
                        if let Some(chunk) = self.manager.get_mut_chunk(chunk_pos){
                            if in_bounds{
                                //update
                            }else{
                                self.manager.remove_chunk(chunk_pos);
                            }
                        }
                    }
                }
            }
        }

        // use some time to dequeue some ops
        self.manager.update_chunk_queue();
        // println!("Queue number: {:?}", self.manager.chunk_queue_number());
        let op_timer = Instant::now();
        while op_timer.elapsed() < Duration::from_millis(1) && self.manager.chunk_queue_number() != 0{
            self.manager.dequeue_chunk();
            // self.manager.update_chunk_queue();
        }
    }
}
