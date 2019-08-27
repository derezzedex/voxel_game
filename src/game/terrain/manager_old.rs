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
use crate::game::terrain::chunk_updater::{ChunkUpdater, ChunkOperation};

use rayon::iter::ParallelBridge;
use rayon::prelude::*;

pub type ChunkHashMap = HashMap<ChunkPosition, Arc<Chunk>>;
pub type MeshHashMap = HashMap<ChunkPosition, Mesh>;
pub struct TerrainManager {
    mesher: ChunkMesher,
    meshes: MeshHashMap,
    updater: ChunkUpdater,
    chunks: ChunkHashMap,
    position: ChunkPosition,
    noise: Perlin,
}

impl TerrainManager {
    pub fn new(position: ChunkPosition) -> Self {
        let mesher = ChunkMesher::new(1);
        let meshes = HashMap::new();
        let updater = ChunkUpdater::new(8);
        let chunks = HashMap::new();

        let noise = Perlin::new().set_seed(1102130);

        Self {
            mesher,
            meshes,
            updater,
            chunks,
            position,
            noise,
        }
    }

    pub fn block_at(&self, position: Point3<f64>) -> bool {
        let chunk_position = ChunkPosition::new(
            position.x as isize >> 4,
            position.y as isize >> 4,
            position.z as isize >> 4,
        );
        if let Some(chunk) = self.chunks.get(&chunk_position) {
            let block_position = BlockPosition::new(
                position.x as isize,
                position.y as isize,
                position.z as isize,
            )
            .get_offset();
            let (x, y, z) = (
                block_position.x as usize,
                block_position.y as usize,
                block_position.z as usize,
            );

            return chunk.get_blocks()[x as usize][y as usize][z as usize] != BlockType::Air;
        }

        false
    }

    pub fn place_block(&mut self, position: Point3<f64>, face: Vector3<f64>) {
        let block = BlockPosition::new(
            (position.x - face.x) as isize,
            (position.y - face.y) as isize,
            (position.z - face.z) as isize,
        ); //.get_offset();
        let chunk_position = ChunkPosition::new(block.x >> 4, block.y >> 4, block.z >> 4);
        if let Some(chunk) = self.chunks.get_mut(&chunk_position) {
            let chunk = Arc::make_mut(chunk);
            let offset = block.get_offset();
            let (x, y, z) = (offset.x as usize, offset.y as usize, offset.z as usize);
            chunk.place_block(x, y, z, BlockType::Dirt);
        }
    }

    pub fn get_chunk(&self, position: ChunkPosition) -> Option<&Arc<Chunk>> {
        self.chunks.get(&position)
    }

    pub fn get_mut_chunk(&mut self, position: ChunkPosition) -> Option<&mut Arc<Chunk>> {
        self.chunks.get_mut(&position)
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

                    if let Some(chunk) = self.chunks.get_mut(&chunk_pos){
                        if in_bounds{
                            //update
                        }else{
                            self.remove_chunk(chunk_pos);
                        }
                    }else if in_bounds{
                        self.create_chunk_at(chunk_pos);
                    }
                }
            }
        }
        let elapsed = timer.elapsed();

        // retrieve
        self.updater.remove_late_chunks(position, view_distance);
        self.updater.update_available_chunks();

        let max_time = std::time::Duration::from_millis(50);
        if max_time < max_time{
            let time_left = max_time - elapsed;

            let retrieve_timer = Instant::now();
            while retrieve_timer.elapsed() < time_left && self.updater.available_chunk_number() != 0 {
                if let Some((pos, chunk, op)) = self.updater.retrieve_first() {
                    match op{
                        ChunkOperation::Creation => self.add_chunk(pos, chunk),
                        ChunkOperation::Remotion => self.remove_chunk(pos),
                        _ => (),
                    }
                }
            }
        }
        println!("Update time: {:?}", elapsed);
    }

    // pub fn update_chunk_area(&mut self, position: ChunkPosition, view_distance: isize) {
    //     let timer = Instant::now();
    //     if self.position != position {
    //         // ------------ REMOVE OUT OF SIGHT CHUNKS
    //         let mut remove_list = Vec::new();
    //         for (chunk_pos, _) in &self.chunks {
    //             if !(chunk_pos.x > position.x - view_distance
    //                 && chunk_pos.y > position.y - view_distance
    //                 && chunk_pos.z > position.z - view_distance
    //                 && chunk_pos.x < position.x + view_distance
    //                 && chunk_pos.y < position.y + view_distance
    //                 && chunk_pos.z < position.z + view_distance)
    //             {
    //                 remove_list.push(chunk_pos.clone());
    //             }
    //         }
    //
    //         for pos in remove_list {
    //             self.remove_chunk(pos);
    //         }
    //
    //         println!("Remove time: {:?}", timer.elapsed());
    //
    //         // LOAD IN SIGHT CHUNKS
    //
    //         let timer_load = Instant::now();
    //         for x in (position.x - view_distance)..(position.x + view_distance) {
    //             for y in (position.y - view_distance)..(position.y + view_distance) {
    //                 for z in (position.z - view_distance)..(position.z + view_distance) {
    //                     let chunk_pos = ChunkPosition::new(x, y, z);
    //                     if !self.chunks.contains_key(&chunk_pos) {
    //                         // let c_timer = Instant::now();
    //                         // self.create_chunk_at([x, y, z]);
    //                         self.updater.new_chunk(ChunkPosition::new(x, y, z));
    //                         // println!("Create time: {:?}", c_timer.elapsed());
    //                     }
    //                 }
    //             }
    //         }
    //         println!("Add time: {:?}", timer_load.elapsed());
    //
    //         self.position = position;
    //         println!("Update time: {:?}", timer.elapsed());
    //     }
    //     self.updater.remove_late_chunks(position, view_distance);
    //     self.updater.update_available_chunks();
    //
    //     let time_left = std::time::Duration::from_millis(50) - timer.elapsed();
    //
    //     let retrieve_timer = Instant::now();
    //     while retrieve_timer.elapsed() < time_left && self.updater.available_chunk_number() != 0 {
    //         if let Some((pos, chunk, op)) = self.updater.retrieve_first() {
    //             self.add_chunk(pos, chunk);
    //         }
    //     }
    // }

    fn get_neighbors(&self, position: ChunkPosition) -> [Option<&Arc<Chunk>>; 6] {
        let north = self.get_chunk(ChunkPosition::new(position.x, position.y, position.z + 1));
        let south = self.get_chunk(ChunkPosition::new(position.x, position.y, position.z - 1));

        let east = self.get_chunk(ChunkPosition::new(position.x + 1, position.y, position.z));
        let west = self.get_chunk(ChunkPosition::new(position.x - 1, position.y, position.z));

        let up = self.get_chunk(ChunkPosition::new(position.x, position.y + 1, position.z));
        let down = self.get_chunk(ChunkPosition::new(position.x, position.y - 1, position.z));

        [north, south, east, west, up, down]
    }

    fn get_neighbors_position(&self, position: ChunkPosition) -> [ChunkPosition; 6] {
        let north = ChunkPosition::new(position.x, position.y, position.z + 1);
        let south = ChunkPosition::new(position.x, position.y, position.z - 1);

        let east = ChunkPosition::new(position.x + 1, position.y, position.z);
        let west = ChunkPosition::new(position.x - 1, position.y, position.z);

        let up = ChunkPosition::new(position.x, position.y + 1, position.z);
        let down = ChunkPosition::new(position.x, position.y - 1, position.z);

        [north, south, east, west, up, down]
    }

    fn dirty_neighbors(&mut self, position: ChunkPosition) {
        let mut neighbors = self.get_neighbors_position(position);
        for neighbor_position in neighbors.iter() {
            if let Some(mut chunk) = self.chunks.get_mut(neighbor_position) {
                chunk.flag_dirty();
            }
        }
    }

    pub fn add_chunk(&mut self, position: ChunkPosition, chunk: Arc<Chunk>) {
        self.dirty_neighbors(position);
        self.chunks.insert(position, chunk);
    }

    pub fn remove_chunk(&mut self, position: ChunkPosition) {
        self.dirty_neighbors(position);
        self.chunks.remove(&position);
        self.meshes.remove(&position);
    }

    pub fn generate_chunk(&mut self, position: ChunkPosition, mut chunk: Chunk) -> Chunk {
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let (mut nx, mut nz) = (
                        (position.x * CHUNK_SIZE as isize + x as isize) as f64,
                        (position.z * CHUNK_SIZE as isize + z as isize) as f64,
                    );
                    nx /= CHUNK_SIZE as f64;
                    // nx -= 0.5;
                    nz /= CHUNK_SIZE as f64;
                    // nz -= 0.5;

                    let mut h = 6. * self.noise.get([1. * nx, 1. * nz]);
                    h += 2. * self.noise.get([2.01 * nx, 2.01 * nz]);
                    h += 1. * self.noise.get([4.01 * nx, 4.01 * nz]);
                    h += 0.5 * self.noise.get([2.1 * nx, 2.1 * nz]);

                    if (position.y * CHUNK_SIZE as isize + y as isize) as f64 > h {
                        continue;
                    } else {
                        chunk.get_mut_blocks()[x][y][z] = BlockType::Dirt;
                    }
                }
            }
        }

        chunk
    }

    pub fn create_chunk_at(&mut self, position: ChunkPosition) {
        let mut chunk = Chunk::new(BlockType::Air);
        let chunk = self.generate_chunk(position, chunk);
        self.add_chunk(position, Arc::new(chunk));
    }

    pub fn mesh_dirty_chunks(&mut self, atlas: &TextureAtlas, registry: &BlockRegistry) {
        for (pos, chunk) in &self.chunks {
            if chunk.is_dirty() {
                let neighbors = self.get_neighbors(*pos);
                self.mesh(*pos, chunk, neighbors, atlas, registry);

                chunk.flag_clean();
            }
        }
    }

    fn mesh(
        &self,
        position: ChunkPosition,
        chunk: &Arc<Chunk>,
        neighbors: [Option<&Arc<Chunk>>; 6],
        atlas: &TextureAtlas,
        registry: &BlockRegistry,
    ) {
        self.mesher
            .mesh(position, chunk, neighbors, atlas, registry);
    }

    // pub fn update_chunk(&mut self, position: ChunkPosition, chunk: &mut Chunk){
    //     self.chunks.entry(position).and_modify(|c| c = chunk);
    // }

    pub fn update_received_meshes(&mut self, display: &glium::Display) {
        // let mut flagged_clean = Vec::new();
        for (pos, mesh) in self.mesher.get_available_meshes() {
            let built_mesh = mesh.build(display);
            if self.meshes.contains_key(&pos) {
                self.meshes.entry(pos).and_modify(|mesh| *mesh = built_mesh);
            } else {
                self.meshes.insert(pos, built_mesh);
            }
        }
    }

    pub fn get_meshes(&self) -> &MeshHashMap {
        &self.meshes
    }
}
