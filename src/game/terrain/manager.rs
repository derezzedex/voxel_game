/*
DashMap<ChunkPosition, Chunk>,
DashMap<ChunkPosition, Mesh>,
VecDeque<Chunk>
*/
use dashmap::DashMap;
use dashmap::DashMapRef;
use dashmap::DashMapRefMut;
use threadpool::ThreadPool;
use noise::{NoiseFn, Perlin, Seedable};
use std::borrow::Borrow;
use std::collections::VecDeque;
use std::ops::Deref;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

use crate::engine::mesh::{Mesh, MeshData};
use crate::engine::renderer::Context;
use crate::game::terrain::block::{BlockType, Direction, FaceData};
use crate::game::terrain::block_registry::{BlockDataBuilder, BlockRegistry};
use crate::game::terrain::chunk::{Chunk, ChunkPosition, CHUNK_SIZE};
use crate::utils::texture::{TextureAtlas, TextureCoords};

pub type ChunkRef<'a> = DashMapRef<'a, ChunkPosition, Arc<Chunk>>;
pub type ChunkRefMut<'a> = DashMapRefMut<'a, ChunkPosition, Arc<Chunk>>;

type ChunkMeshMessage = (ChunkPosition, MeshData);
pub struct ChunkMesher{
    threadpool: ThreadPool,
    sender: Sender<ChunkMeshMessage>,
    receiver: Receiver<ChunkMeshMessage>
}

impl ChunkMesher{
    pub fn new() -> Self{
        let (sender, receiver) = std::sync::mpsc::channel();
        let threadpool = ThreadPool::new(1);

        Self{
            threadpool,
            sender,
            receiver
        }
    }

    pub fn get_received_messages(&mut self) -> std::sync::mpsc::TryIter<ChunkMeshMessage>{
        self.receiver.try_iter()
    }
}

pub type ChunkUpdateMessage = (ChunkPosition, Arc<Chunk>);

pub struct ChunkUpdater{
    threadpool: ThreadPool,
    sender: Sender<ChunkUpdateMessage>,
    receiver: Receiver<ChunkUpdateMessage>
}

impl ChunkUpdater{
    pub fn new() -> Self{
        let (sender, receiver) = std::sync::mpsc::channel();
        let threadpool = ThreadPool::new(1);

        Self{
            threadpool,
            sender,
            receiver
        }
    }

    pub fn get_received_messages(&mut self) -> std::sync::mpsc::TryIter<ChunkUpdateMessage>{
        self.receiver.try_iter()
    }
}

pub struct TerrainManager {
    registry: Arc<BlockRegistry>,
    texture_atlas: TextureAtlas,
    chunks: DashMap<ChunkPosition, Arc<Chunk>>,
    meshes: DashMap<ChunkPosition, Mesh>,
    mesher: ChunkMesher,
    updater: ChunkUpdater,
    chunk_queue: VecDeque<ChunkUpdateMessage>,
    dirty_queue: VecDeque<ChunkPosition>,
    clean_queue: VecDeque<ChunkMeshMessage>,
    noise: Arc<Perlin>,
    position: ChunkPosition
}

impl TerrainManager {
    pub fn new(position: ChunkPosition, context: &Context) -> Self {
        // Create and setup the texture atlas
        let cargo = env!("CARGO_MANIFEST_DIR");
        let path = std::path::Path::new(cargo)
            .join("res")
            .join("img")
            .join("texture")
            .join("atlas.png");
        let texture_atlas = TextureAtlas::new(context.get_display(), &path, 16);

        // Create block registry, which contains the block proprierties
        let mut registry = BlockRegistry::new();

        // AIR
        let air_data = BlockDataBuilder::new("air")
            .orientation(Direction::Up)
            .north_face(texture_atlas.get_coords((3, 0)))
            .top_face(texture_atlas.get_coords((0, 0)))
            .bottom_face(texture_atlas.get_coords((2, 0)))
            .build();
        registry.add_block(BlockType::Air, air_data);

        // DIRT
        let dirt_data = BlockDataBuilder::new("dirt")
            .orientation(Direction::Up)
            .north_face(texture_atlas.get_coords((3, 15)))
            .top_face(texture_atlas.get_coords((0, 15)))
            .bottom_face(texture_atlas.get_coords((2, 15)))
            .build();
        registry.add_block(BlockType::Dirt, dirt_data);

        let registry = Arc::new(registry);
        let chunks = DashMap::default();
        let meshes = DashMap::default();
        let dirty_queue = VecDeque::new();
        let clean_queue = VecDeque::new();
        let chunk_queue = VecDeque::new();

        let mesher = ChunkMesher::new();
        let updater = ChunkUpdater::new();

        let noise = Arc::new(Perlin::new().set_seed(1102130));

        Self {
            registry,
            texture_atlas,
            chunks,
            meshes,
            mesher,
            updater,
            dirty_queue,
            clean_queue,
            chunk_queue,
            position,
            noise
        }
    }

    pub fn update_chunks(&mut self, position: ChunkPosition, view_distance: isize) {
        // calculate new chunks to be loaded
        // if distance bigger than current view_distance, clear all loaded chunks and load all surrounding chunks
        // let timer = Instant::now();
        if self.position != position{
            println!("Position: {:?}", position);
            self.position = position;
            for z in (position.z - view_distance)..(position.z + view_distance) {
                for y in (position.y - view_distance)..(position.y + view_distance) {
                    for x in (position.x - view_distance)..(position.x + view_distance) {

                        let chunk_pos = ChunkPosition::new(x, y, z);
                        let in_bounds = (x > position.x - view_distance
                            && y > position.y - view_distance
                            && z > position.z - view_distance
                            && x < position.x + view_distance
                            && y < position.y + view_distance
                            && z < position.z + view_distance);
                        if !self.chunks.contains_key(&chunk_pos) && in_bounds{
                            // should create chunk
                            self.create_chunk(chunk_pos);
                            self.queue_chunk(chunk_pos);
                        }else{
                            if in_bounds{
                                //update
                                // if let Some(chunk) = self.chunks.get_mut(&chunk_pos){
                                // }
                            }else{
                                // println!("Removing {:?}", chunk_pos);
                                self.meshes.remove(&chunk_pos);
                                self.chunks.remove(&chunk_pos);
                                self.dirty_neighbors(chunk_pos);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn sampled_atlas(&self) -> glium::uniforms::Sampler<'_, glium::texture::Texture2d> {
        self.texture_atlas
            .get_texture()
            .sampled()
            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
    }

    pub fn create_chunk(&mut self, position: ChunkPosition){
        let noise = self.noise.clone();
        let sender = self.updater.sender.clone();
        self.updater.threadpool.execute(move ||{
            let mut chunk = Chunk::new_air();
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

                        let mut h = 6. * noise.get([1. * nx, 1. * nz]);
                        h += 2. * noise.get([2.01 * nx, 2.01 * nz]);
                        h += 1. * noise.get([4.01 * nx, 4.01 * nz]);
                        h += 0.5 * noise.get([2.1 * nx, 2.1 * nz]);

                        if (position.y * CHUNK_SIZE as isize + y as isize) as f64 > h {
                            continue;
                        } else {
                            chunk.get_mut_blocks()[x][y][z] = BlockType::Dirt;
                        }
                    }
                }
            }
            sender.send((position, Arc::new(chunk)));
        });

    }

    pub fn queue_chunk(&mut self, position: ChunkPosition) {
        self.dirty_queue.push_back(position);
    }

    pub fn update_queues(&mut self) {

        let mut received_chunks: VecDeque<_> = self.updater.get_received_messages().collect();
        for chunk in &received_chunks{
            self.dirty_queue.push_back(chunk.deref().0);
        }
        self.chunk_queue.append(&mut received_chunks);

        let mut received_meshes: VecDeque<_> = self.mesher.get_received_messages().collect();
        self.clean_queue.append(&mut received_meshes);
    }

    pub fn mesh_chunk(&mut self) {
        if let Some(position) = self.dirty_queue.pop_front() {
            if let Some(ref chunk) = self.chunks.get(&position) {
                let sender = self.mesher.sender.clone();
                let chunk = chunk.deref().clone();
                let registry = self.registry.clone();
                let mut neighbors: Vec<_> = self
                    .chunk_neighbors(position)
                    .iter()
                    .map(|c_ref| c_ref.as_ref().and_then(|inner| Some(Arc::clone(&**inner))))
                    .collect();


                self.mesher.threadpool.execute(move ||{
                    let mut mesh = MeshData::new();
                    for x in 0..CHUNK_SIZE{
                        for y in 0..CHUNK_SIZE{
                            for z in 0..CHUNK_SIZE{
                                let block_type = chunk.get_blocks()[x][y][z];

                                if block_type == BlockType::Air{
                                    continue;
                                }

                                let directions = [Direction::North, Direction::South, Direction::East, Direction::West, Direction::Up, Direction::Down];

                                for i in 0..directions.len(){
                                    if chunk.get_neighbor(x, y, z, directions[i], neighbors[i].as_ref()) == BlockType::Air{
                                        let coords = registry.get_block(block_type).expect("Block not found when meshing...").get_coords(directions[i]);
                                        let face_data = FaceData::new([x as u8, y as u8, z as u8], block_type, directions[i], *coords);
                                        mesh.add_face(face_data);

                                    }
                                }

                            }
                        }
                    }

                    sender.send((position, mesh));
                });
            }
        }
    }

    pub fn dirty_queue_size(&self) -> usize{
        self.dirty_queue.len()
    }

    pub fn chunk_queue_size(&self) -> usize{
        self.chunk_queue.len()
    }

    pub fn get_meshes<'a>(&'a self) -> dashmap::Iter<'a, ChunkPosition, Mesh>{
        self.meshes.iter()
    }

    pub fn build_chunk(&mut self, display: &glium::Display) {
        if let Some((position, data)) = self.clean_queue.pop_front() {
            let mesh = data.build(display);
            self.meshes.insert(position, mesh);
        }
    }

    pub fn unqueue_created_chunk(&mut self){
        if let Some((position, chunk)) = self.chunk_queue.pop_front(){
            self.chunks.insert(position, chunk);
        }
    }

    pub fn get_chunk(&self, position: ChunkPosition) -> Option<ChunkRef> {
        self.chunks.get(&position)
    }

    pub fn dirty_neighbors(&mut self, position: ChunkPosition){
        self.queue_chunk(ChunkPosition::new(position.x, position.y, position.z + 1)); //north
        self.queue_chunk(ChunkPosition::new(position.x, position.y, position.z - 1)); //south
        self.queue_chunk(ChunkPosition::new(position.x + 1, position.y, position.z)); //east
        self.queue_chunk(ChunkPosition::new(position.x - 1, position.y, position.z)); //west
        self.queue_chunk(ChunkPosition::new(position.x, position.y + 1, position.z)); //up
        self.queue_chunk(ChunkPosition::new(position.x, position.y - 1, position.z)); //down
    }

    pub fn chunk_neighbors(&self, position: ChunkPosition) -> [Option<ChunkRef>; 6] {
        let north = self.get_chunk(ChunkPosition::new(position.x, position.y, position.z + 1));
        let south = self.get_chunk(ChunkPosition::new(position.x, position.y, position.z - 1));

        let east = self.get_chunk(ChunkPosition::new(position.x + 1, position.y, position.z));
        let west = self.get_chunk(ChunkPosition::new(position.x - 1, position.y, position.z));

        let up = self.get_chunk(ChunkPosition::new(position.x, position.y + 1, position.z));
        let down = self.get_chunk(ChunkPosition::new(position.x, position.y - 1, position.z));

        [north, south, east, west, up, down]
    }
}
