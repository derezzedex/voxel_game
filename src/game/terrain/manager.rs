use crate::game::terrain::block::BlockData;
use super::chunk::CHUNKSIZE;
use super::chunk::{Chunk, ChunkPosition};
use super::block::Direction;
use crate::engine::mesh::{Mesh, MeshData};
use crate::game::registry::Registry;
use crate::game::terrain::chunk::FromWorld;
use super::mesher::*;

use cgmath::{Point3, Vector3};
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use uvth::{ThreadPool, ThreadPoolBuilder};
use noise::{Fbm, NoiseFn, Seedable};

use std::sync::Arc;
use std::convert::TryFrom;

pub fn range_map(s: f64, a: [f64; 2], b: [f64; 2]) -> f64 {
    b[0] + ((s - a[0]) * (b[1] - b[0])) / (a[1] - a[0])
}


pub const LOAD_DISTANCE: isize = 4;

pub type ChunkRef<'a> = Ref<'a, ChunkPosition, Arc<Chunk>>;
pub type ChunkMap = DashMap<ChunkPosition, Arc<Chunk>>;
pub type ChunkMeshMap = DashMap<ChunkPosition, (Mesh, Option<Mesh>)>;
pub type ChunkPositionSet = DashMap<ChunkPosition, ()>;
pub struct TerrainManager {
    position: ChunkPosition,
    chunks: Arc<ChunkMap>,
    visible_chunks: ChunkPositionSet,
    registry: Arc<Registry>,
    threadpool: ThreadPool,
    mesher: ChunkMesher,
    meshes: ChunkMeshMap,
    noise: Arc<Fbm>,
}

#[allow(dead_code)]
impl TerrainManager {
    pub fn new(registry: &Arc<Registry>) -> Self {
        let chunks = Arc::new(ChunkMap::default());
        let meshes = ChunkMeshMap::default();

        let threadpool = ThreadPoolBuilder::new()
            .num_threads(1) // TODO: Measure performance difference on thread qtd change
            .name("TerrainManager".to_string())
            .build();
        let mesher = ChunkMesher::new();

        let noise = Arc::new(Fbm::new().set_seed(10291302));
        let registry = registry.clone();
        let position = ChunkPosition::new(0, 1, 0);

        let visible_chunks = DashMap::default();

        Self {
            position,
            chunks,
            visible_chunks,
            threadpool,
            registry,
            mesher,
            meshes,
            noise,
        }
    }

    pub fn get_registry(&self) -> &Registry{
        &self.registry
    }

    pub fn setup(&mut self, _display: &glium::Display) {
        println!("Generating {} initial chunks...", (LOAD_DISTANCE * 2 + 1).pow(3));
        let timer = std::time::Instant::now();
        for z in -LOAD_DISTANCE..=LOAD_DISTANCE {
            for y in -LOAD_DISTANCE..=LOAD_DISTANCE{
                for x in -LOAD_DISTANCE..=LOAD_DISTANCE {
                    let position = ChunkPosition::new(x, y, z);
                    let chunks = self.chunks.clone();
                    let noise = self.noise.clone();
                    let registry = self.registry.clone();
                    TerrainManager::generate_chunk(position, chunks, noise, registry);
                }
            }
        }
        println!("Generated {} in {:?}", self.chunks.len(), timer.elapsed());
    }

    pub fn update(&mut self, position: ChunkPosition) {
        if position != self.position{
            self.position = position;
            for z in -LOAD_DISTANCE..=LOAD_DISTANCE{
                for y in -LOAD_DISTANCE..=LOAD_DISTANCE{
                    for x in -LOAD_DISTANCE..=LOAD_DISTANCE{
                        let position = ChunkPosition::new(position.x + x, position.y + y, position.z + z);
                        if !self.chunks.contains_key(&position){
                            self.send_to_generate(position);
                        }
                    }
                }
            }
        }
    }

    pub fn dirty_chunk(&mut self, position: &ChunkPosition){
        self.send_to_mesh(&position);
        let neighbors = [
            Point3::new(position.x + 1, position.y, position.z),
            Point3::new(position.x - 1, position.y, position.z),
            Point3::new(position.x, position.y + 1, position.z),
            Point3::new(position.x, position.y - 1, position.z),
            Point3::new(position.x, position.y, position.z + 1),
            Point3::new(position.x, position.y, position.z - 1)
        ];
        for chunk in &neighbors{
            self.send_to_mesh(chunk);
        }
    }

    pub fn mesh_chunks(&mut self, display: &glium::Display, timer: &std::time::Instant) {
        //temp
        let mut remove_list = Vec::new();
        for c_ref in self.chunks.clone().iter(){
            let position = c_ref.key();
            if (position.x >= self.position.x - LOAD_DISTANCE && position.x <= self.position.x + LOAD_DISTANCE)
            && (position.y >= self.position.y - LOAD_DISTANCE && position.y <= self.position.y + LOAD_DISTANCE)
            && (position.z >= self.position.z - LOAD_DISTANCE && position.z <= self.position.z + LOAD_DISTANCE){
                if !self.meshes.contains_key(position) && !self.visible_chunks.contains_key(position){
                    // self.send_to_mesh(position);
                    self.dirty_chunk(position);
                    self.visible_chunks.insert(*position, ());
                }
            }else{
                self.meshes.remove(position);
                self.visible_chunks.remove(position);
                // self.chunks.remove(position);
                remove_list.push(position.clone());
            }
        }

        for chunk in remove_list{
            self.chunks.remove(&chunk);
        }

        let min_fps = 60; // chunk meshing will not get the fps lower than this number
        while timer.elapsed() < std::time::Duration::from_millis(1000/min_fps){
            if let Ok((position, opaque, transparent)) = self.mesher.receive(){
                let opaque_mesh = opaque.build(display);
                let transparent_mesh = if let Some(mesh) = transparent{
                    Some(mesh.build(display))
                }else{ None };
                self.meshes.insert(position, (opaque_mesh, transparent_mesh));
            }else{
                break;
            }
        }
    }

    pub fn get_chunks(&self) -> &Arc<ChunkMap> {
        &self.chunks
    }

    pub fn get_meshes(&self) -> &ChunkMeshMap {
        &self.meshes
    }

    pub fn block_at(&self, x: f32, y: f32, z: f32) -> Option<(usize, &BlockData)>{
        let chunk_pos = ChunkPosition::from_world(x, y, z);
        if let Some(chunk_ref) = self.chunks.get(&chunk_pos){
            let b_pos = [(((x % CHUNKSIZE as f32) + CHUNKSIZE as f32) % CHUNKSIZE as f32) as usize, (((y % CHUNKSIZE as f32) + CHUNKSIZE as f32) % CHUNKSIZE as f32) as usize, (((z % CHUNKSIZE as f32) + CHUNKSIZE as f32) % CHUNKSIZE as f32) as usize];
            let block = chunk_ref.value().get_block(b_pos[0], b_pos[1], b_pos[2]);
            let data = self.registry.block_registry().by_id(block).expect(&format!("Unknown block: {}", block));
            return Some((block, data));
        }

        None
    }

    pub fn set_block(&mut self, x: f32, y: f32, z: f32, block: usize){
        let c_pos = ChunkPosition::from_world(x, y, z);
        let block = if let Some(mut chunk) = self.chunks.get_mut(&c_pos){
            let chunk = Arc::make_mut(chunk.value_mut());
            let b_pos = Point3::new((((x % CHUNKSIZE as f32) + CHUNKSIZE as f32) % CHUNKSIZE as f32) as usize, (((y % CHUNKSIZE as f32) + CHUNKSIZE as f32) % CHUNKSIZE as f32) as usize, (((z % CHUNKSIZE as f32) + CHUNKSIZE as f32) % CHUNKSIZE as f32) as usize);
            chunk.set_block(b_pos.x, b_pos.y, b_pos.z, block);

            Some(b_pos)
        }else{
            None
        };

        if let Some(block) = block{
            if block.x + 1 > CHUNKSIZE{
                self.dirty_chunk(&ChunkPosition::new(c_pos.x + 1, c_pos.y, c_pos.z));
            }else if block.x as isize - 1 < 0{
                self.dirty_chunk(&ChunkPosition::new(c_pos.x - 1, c_pos.y, c_pos.z));
            }

            if block.y + 1 > CHUNKSIZE{
                self.dirty_chunk(&ChunkPosition::new(c_pos.x, c_pos.y + 1, c_pos.z));
            }else if block.y as isize - 1 < 0{
                self.dirty_chunk(&ChunkPosition::new(c_pos.x, c_pos.y - 1, c_pos.z));
            }

            if block.z + 1 > CHUNKSIZE{
                self.dirty_chunk(&ChunkPosition::new(c_pos.x, c_pos.y, c_pos.z + 1));
            }else if block.z as isize - 1 < 0{
                self.dirty_chunk(&ChunkPosition::new(c_pos.x, c_pos.y, c_pos.z - 1));
            }
            self.dirty_chunk(&c_pos);
        }
    }

    fn chunk_neighbors(&self, position: &ChunkPosition) -> [Option<ChunkRef>; 6] {
        let east = self
            .chunks
            .get(&Point3::new(position.x + 1, position.y, position.z)); // East
        let west = self
            .chunks
            .get(&Point3::new(position.x - 1, position.y, position.z)); // West
        let top = self
            .chunks
            .get(&Point3::new(position.x, position.y + 1, position.z)); // Top
        let bottom = self
            .chunks
            .get(&Point3::new(position.x, position.y - 1, position.z)); // Bottom
        let north = self
            .chunks
            .get(&Point3::new(position.x, position.y, position.z + 1)); // North
        let south = self
            .chunks
            .get(&Point3::new(position.x, position.y, position.z - 1)); // South

        [east, west, top, bottom, north, south]
    }

    fn send_to_generate(&mut self, position: ChunkPosition){
        let chunks = self.chunks.clone();
        let noise = self.noise.clone();
        let registry = self.registry.clone();
        self.threadpool.execute(move || TerrainManager::generate_chunk(position, chunks, noise, registry));
    }

    fn generate_chunk(position: ChunkPosition, chunks: Arc<ChunkMap>, noise: Arc<Fbm>, registry: Arc<Registry>) {
        let mut chunk = Chunk::new(0);
        let grass = registry.block_registry().id_of("grass").unwrap_or(1);
        let sand = registry.block_registry().id_of("sand").unwrap_or(1);
        let dirt = registry.block_registry().id_of("dirt").unwrap_or(1);
        let stone = registry.block_registry().id_of("stone").unwrap_or(1);
        // let bedrock = registry.block_registry().id_of("bedrock").unwrap_or(1);
        let water = registry.block_registry().id_of("water").unwrap_or(1);

        let frequency = 0.005f64;

        for z in 0..CHUNKSIZE {
            for y in 0..CHUNKSIZE {
                for x in 0..CHUNKSIZE {
                    let nx = (position.x * CHUNKSIZE as isize + x as isize) as f64;
                    let ny = (position.y * CHUNKSIZE as isize + y as isize) as f64;
                    let nz = (position.z * CHUNKSIZE as isize + z as isize) as f64;

                    let elevation = 6. * noise.get([nx * frequency, nz * frequency]);
                    let height = range_map(
                        elevation,
                        [-1., 1.],
                        [0., CHUNKSIZE as f64],
                    )
                    .round();

                    if ny <= -(CHUNKSIZE as f64) + 1. && ny < -height + 1.{
                        chunk.set_block(x, y, z, sand);
                    } else if ny == -height{
                        chunk.set_block(x, y, z, grass);
                    } else if ny <= -(CHUNKSIZE as f64) && ny >= -height{
                        chunk.set_block(x, y, z, water);
                    } else if ny < -height - 3. {
                        chunk.set_block(x, y, z, stone);
                    } else if ny < -height {
                        chunk.set_block(x, y, z, dirt);
                    }
                }
            }
        }

        chunks.insert(position, Arc::new(chunk));
    }

    fn send_to_mesh(&mut self, position: &ChunkPosition) {
        if let Some(chunk) = self.chunks.get(position) {
            let sender = self.mesher.sender();
            let registry = self.registry.clone();

            let chunk = chunk.value().clone();
            let neighbors: Vec<Option<Arc<Chunk>>> = self
                .chunk_neighbors(position)
                .iter()
                .map(|n_ref| n_ref.as_ref().and_then(|inner| Some(Arc::clone(inner))))
                .collect::<Vec<_>>();
            let position = position.clone();

            self.threadpool.execute(move || {
                let mut opaque_mesh = MeshData::new();
                let mut transparent_mesh = MeshData::new();
                let air = registry.block_registry().id_of("air").expect("Air missing in Registry!");
                let block_mesh = registry.mesh_registry().id_of("block").expect("Block mesh missing in Registry!");

                for x in 0..CHUNKSIZE{
                    for y in 0..CHUNKSIZE{
                        for z in 0..CHUNKSIZE{
                            let block = chunk.get_block(x, y, z);

                            if block != air{
                                let b_data = registry.block_registry().by_id(block).expect("Unknown block in chunk");
                                let b_mesh = b_data.get_mesh();
                                match b_data.get_mesh(){
                                    b_mesh if b_mesh == block_mesh => {
                                        for direction in &[Direction::East, Direction::West, Direction::Top, Direction::Bottom, Direction::North, Direction::South]{
                                            let facing = Vector3::new(x as isize, y as isize, z as isize) + direction.normal();
                                            let facing = chunk.check_block(facing.x, facing.y, facing.z, neighbors.clone()); //TODO: Fix this clone
                                            let facing_data = registry.block_registry().by_id(facing).expect("Unknown block in chunk");
                                            if facing_data.is_transparent(){
                                                if facing == block { continue }
                                                let block = if let Some(block_data) = registry.block_registry().by_id(block as usize) { block_data.get_face(Direction::try_from(*direction).unwrap_or(Direction::East)) } else{ [0, 1] };
                                                if b_data.is_transparent(){
                                                    transparent_mesh.add_face(Point3::new(x as f32, y as f32, z as f32), *direction, block);
                                                }else{
                                                    opaque_mesh.add_face(Point3::new(x as f32, y as f32, z as f32), *direction, block);
                                                }
                                            }
                                        }
                                    },
                                    _ => {
                                        let mut custom_mesh = registry.mesh_registry().by_id(b_mesh).expect("Unknown mesh in chunk").get_mesh().clone();
                                        let tex_coord = b_data.get_face(Direction::East);
                                        for vertex in &mut custom_mesh.vertices{
                                            vertex.block = tex_coord;
                                        }

                                        if b_data.is_transparent(){
                                            transparent_mesh.append(custom_mesh);
                                        }else{
                                            opaque_mesh.append(custom_mesh);
                                        }
                                    },
                                }
                            }
                        }
                    }
                }

                if opaque_mesh.vertices.len() != 0 || transparent_mesh.vertices.len() != 0{
                    let transparent_mesh = if transparent_mesh.vertices.len() != 0 { Some(transparent_mesh) } else { None };
                    sender
                        .send((position, opaque_mesh, transparent_mesh))
                        .expect("Couldn't send chunk to main thread!");
                }
            });
        }
    }
}
