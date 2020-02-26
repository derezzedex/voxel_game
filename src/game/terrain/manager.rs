use super::chunk::CHUNKSIZE;
use super::chunk::{Chunk, ChunkPosition};
use super::block::Direction;
use crate::engine::mesh::{Mesh, MeshData};
use crate::game::registry::Registry;
use crate::game::terrain::chunk::FromWorld;
use crate::engine::Vertex;
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


pub const LOAD_DISTANCE: isize = 6;
pub const HALF: f32 = 0.5;

pub type ChunkRef<'a> = Ref<'a, ChunkPosition, Arc<Chunk>>;
pub type ChunkMap = DashMap<ChunkPosition, Arc<Chunk>>;
pub type ChunkMeshMap = DashMap<ChunkPosition, Mesh>;
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
        .num_threads(1)
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

    pub fn setup(&mut self, _display: &glium::Display) {
        for z in -LOAD_DISTANCE..=LOAD_DISTANCE {
            for y in -LOAD_DISTANCE..=LOAD_DISTANCE{
                for x in -LOAD_DISTANCE..=LOAD_DISTANCE {
                    self.generate_chunk(ChunkPosition::new(x, y, z));
                }
            }
        }
        // self.generate_chunk(ChunkPosition::new(0, -1, 0));
    }

    pub fn update(&mut self, position: ChunkPosition) {
        if position != self.position{
            self.position = position;
            for z in -LOAD_DISTANCE..=LOAD_DISTANCE{
                for y in -LOAD_DISTANCE..=LOAD_DISTANCE{
                    for x in -LOAD_DISTANCE..=LOAD_DISTANCE{
                        let position = ChunkPosition::new(position.x + x, position.y + y, position.z + z);
                        if !self.chunks.contains_key(&position){
                            self.generate_chunk(position);
                        }
                    }
                }
            }
        }
    }

    pub fn dirty_chunk(&mut self, position: ChunkPosition){
        self.meshes.remove(&position);
        self.visible_chunks.remove(&position);
    }

    pub fn mesh_chunks(&mut self, display: &glium::Display) {
        //temp
        for c_ref in self.chunks.clone().iter(){
            let position = c_ref.key();
            if (position.x >= self.position.x - LOAD_DISTANCE && position.x <= self.position.x + LOAD_DISTANCE)
            && (position.y >= self.position.y - LOAD_DISTANCE && position.y <= self.position.y + LOAD_DISTANCE)
            && (position.z >= self.position.z - LOAD_DISTANCE && position.z <= self.position.z + LOAD_DISTANCE){
                if !self.meshes.contains_key(position) && !self.visible_chunks.contains_key(position){
                    self.send_to_mesh(position);
                    self.visible_chunks.insert(*position, ());
                }
            }else{
                self.meshes.remove(position);
                self.visible_chunks.remove(position);
                // self.chunks.remove(position);
            }
        }

        if let Ok((position, data)) = self.mesher.receive(){
            // let time = std::time::Instant::now();
            let mesh = data.build(display);
            // println!("Buffer creation: {:?}", time.elapsed());
            self.meshes.insert(position, mesh);
        }
    }

    pub fn get_chunks(&self) -> &Arc<ChunkMap> {
        &self.chunks
    }

    pub fn get_meshes(&self) -> &ChunkMeshMap {
        &self.meshes
    }

    pub fn block_at(&self, x: f64, y: f64, z: f64) -> Option<usize>{
        let chunk_pos = ChunkPosition::from_world(x, y, z);
        if let Some(chunk_ref) = self.chunks.get(&chunk_pos){
            let b_pos = [x % CHUNKSIZE as f64, y % CHUNKSIZE as f64, z % CHUNKSIZE as f64];
            return Some(chunk_ref.value().get_block(b_pos[0].abs() as usize, b_pos[1].abs() as usize, b_pos[2].abs() as usize));
        }

        None
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

    fn generate_chunk(&mut self, position: ChunkPosition) {
        let chunks = self.chunks.clone();
        let noise = self.noise.clone();
        let registry = self.registry.clone();
        self.threadpool.execute(move || {
            let mut chunk = Chunk::new(0);
            let grass = registry.block_registry().id_of("grass").unwrap_or(1);
            let dirt = registry.block_registry().id_of("dirt").unwrap_or(1);
            let stone = registry.block_registry().id_of("stone").unwrap_or(1);
            let bedrock = registry.block_registry().id_of("bedrock").unwrap_or(1);

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

                        if ny == -height {
                            chunk.set_block(x, y, z, grass);
                        } else if ny == -(CHUNKSIZE as f64) {
                            chunk.set_block(x, y, z, bedrock);
                        } else if ny < -height - 3. {
                            chunk.set_block(x, y, z, stone);
                        } else if ny < -height {
                            chunk.set_block(x, y, z, dirt);
                        }
                    }
                }
            }

            chunks.insert(position, Arc::new(chunk));
        });
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
                .collect();
            let position = position.clone();

            self.threadpool.execute(move || {
                let mut mesh = MeshData::new();
                let air = registry.block_registry().id_of("air").expect("Air missing in Registry!");

                for x in 0..CHUNKSIZE{
                    for y in 0..CHUNKSIZE{
                        for z in 0..CHUNKSIZE{
                            let block = chunk.get_block(x, y, z);

                            if block != air{
                                for direction in &[Direction::East, Direction::West, Direction::Top, Direction::Bottom, Direction::North, Direction::South]{
                                    let facing = Vector3::new(x as isize, y as isize, z as isize) + direction.normal();
                                    let facing = chunk.check_block(facing.x, facing.y, facing.z, neighbors.clone());
                                    let facing_data = registry.block_registry().by_id(facing).expect("Unknown block in chunk");
                                    if facing_data.is_transparent(){
                                        let block = if let Some(block_data) = registry.block_registry().by_id(block as usize) { block_data.get_face(Direction::try_from(*direction).unwrap_or(Direction::East)) } else{ [0, 1] };
                                        let (x, y, z) = (x as f32, y as f32, z as f32);
                                        let vertices = match direction{
                                            Direction::North => vec![
                                                Vertex::new([x+(-HALF), y+(-HALF), z+HALF], [0., 0.], block),
                                                Vertex::new([x+HALF, y+(-HALF), z+HALF], [1., 0.], block),
                                                Vertex::new([x+(-HALF), y+HALF, z+HALF],  [0., 1.], block),
                                                Vertex::new([x+HALF, y+HALF, z+HALF],  [1., 1.], block)
                                            ],
                                            Direction::South => vec![
                                                Vertex::new([x+HALF, y+(-HALF), z+(-HALF)], [0., 0.], block),
                                                Vertex::new([x+(-HALF), y+(-HALF), z+(-HALF)], [1., 0.], block),
                                                Vertex::new([x+HALF, y+HALF, z+(-HALF)],  [0., 1.], block),
                                                Vertex::new([x+(-HALF), y+HALF, z+(-HALF)],  [1., 1.], block)
                                            ],
                                            Direction::West => vec![
                                                Vertex::new([x+(-HALF), y+(-HALF),z+(-HALF)], [0., 0.], block),
                                                Vertex::new([x+(-HALF), y+(-HALF), z+HALF], [1., 0.], block),
                                                Vertex::new([x+(-HALF),y+HALF, z+(-HALF)], [0., 1.], block),
                                                Vertex::new([x+(-HALF), y+HALF,  z+HALF], [1., 1.], block)
                                            ],
                                            Direction::East => vec![
                                                Vertex::new([x+HALF, y+(-HALF),  z+HALF], [0., 0.], block),
                                                Vertex::new([x+HALF, y+(-HALF), z+(-HALF)], [1., 0.], block),
                                                Vertex::new([x+HALF,  y+HALF,  z+HALF], [0., 1.], block),
                                                Vertex::new([x+HALF,  y+HALF, z+(-HALF)], [1., 1.], block)
                                            ],
                                            Direction::Top => vec![
                                                Vertex::new([x+(-HALF), y+HALF, z+HALF], [0., 0.], block),
                                                Vertex::new([x+HALF, y+HALF, z+HALF], [1., 0.], block),
                                                Vertex::new([x+(-HALF), y+HALF, z+(-HALF)], [0., 1.], block),
                                                Vertex::new([x+HALF, y+HALF, z+(-HALF)], [1., 1.], block)
                                            ],
                                            Direction::Bottom => vec![
                                                Vertex::new([x+(-HALF), y+(-HALF), z+(-HALF)], [0., 0.], block),
                                                Vertex::new([x+HALF, y+(-HALF), z+(-HALF)], [1., 0.], block),
                                                Vertex::new([x+(-HALF), y+(-HALF), z+HALF], [0., 1.], block),
                                                Vertex::new([x+HALF, y+(-HALF), z+HALF], [1., 1.], block)
                                            ],
                                        };
                                        let indices = vec![2, 3, 1, 1, 0, 2];
                                        mesh.add(vertices, indices);
                                    }

                                }
                            }
                        }
                    }
                }

                if mesh.indices.len() != 0 {
                    sender
                        .send((position, mesh))
                        .expect("Couldn't send chunk to main thread!");
                }
            });
        }
    }
}
