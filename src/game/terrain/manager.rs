/*
DashMap<ChunkPosition, Chunk>,
DashMap<ChunkPosition, Mesh>,
VecDeque<Chunk>
*/
use crate::game::terrain::chunk::BlockPosition;
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
use std::convert::TryInto;

use crate::engine::Vertex;
use crate::engine::mesh::{Mesh, MeshData};
use crate::engine::renderer::Context;
use crate::game::terrain::block::{BlockType, Direction, FaceData};
use crate::game::terrain::block_registry::{BlockDataBuilder, BlockRegistry};
use crate::game::terrain::chunk::{Chunk, ChunkPosition, CHUNK_SIZE};
use crate::utils::texture::{TextureAtlas, TextureCoords, TextureStorage, TextureArray};

use cgmath::VectorSpace;

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

pub type ChunkMap = DashMap<ChunkPosition, Arc<Chunk>>;

pub struct TerrainManager {
    registry: Arc<BlockRegistry>,
    texture_atlas: TextureAtlas,
    texture_storage: TextureStorage,
    chunks: DashMap<ChunkPosition, Arc<Chunk>>,
    meshes: DashMap<ChunkPosition, Mesh>,
    mesher: ChunkMesher,
    updater: ChunkUpdater,
    chunk_queue: VecDeque<ChunkUpdateMessage>,
    dirty_queue: VecDeque<ChunkPosition>,
    view_distance: isize,
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
        let texture_storage = TextureStorage::new(context.get_display(), &path, image::PNG, 16);

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

        //COBBLESTONE
        let dirt_data = BlockDataBuilder::new("cobblestone")
            .orientation(Direction::Up)
            .north_face(texture_atlas.get_coords((0, 14)))
            .build();
        registry.add_block(BlockType::Cobblestone, dirt_data);

        let registry = Arc::new(registry);
        let chunks = DashMap::default();
        let meshes = DashMap::default();
        let dirty_queue = VecDeque::new();
        let clean_queue = VecDeque::new();
        let chunk_queue = VecDeque::new();
        let view_distance = 1;

        let mesher = ChunkMesher::new();
        let updater = ChunkUpdater::new();

        let noise = Arc::new(Perlin::new().set_seed(1102130));

        Self {
            registry,
            texture_atlas,
            texture_storage,
            chunks,
            meshes,
            mesher,
            updater,
            dirty_queue,
            clean_queue,
            chunk_queue,
            view_distance,
            position,
            noise
        }
    }

    pub fn test_terrain(&mut self){
        let position = ChunkPosition::new(0, -1, 1);
        let sender = self.updater.sender.clone();
        let mut chunk = Chunk::new_air();
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    // if x == 0 || y == 0 || z == 0 || x == CHUNK_SIZE || y == CHUNK_SIZE || z == CHUNK_SIZE{
                    // if x%2 == 0 && y%2 == 0 && z%2 == 0{
                        // chunk.get_mut_blocks()[x][y][z] = BlockType::Cobblestone;
                    // }else{
                        chunk.get_mut_blocks()[x][y][z] = BlockType::Dirt;
                    // }
                }
            }
        }

        chunk.get_mut_blocks()[0][15][15] = BlockType::Cobblestone;
        sender.send((position, Arc::new(chunk)));

        // self.create_chunk(chunk_pos);
        // self.queue_chunk(chunk_pos);
    }

    pub fn block_at(&self, position: BlockPosition) -> bool{
        if let Some(ref chunk) = self.chunks.get(&position.to_chunk()){
            chunk.has_block_at(position)
        }else{
            false
        }
    }

    pub fn generate_image(&self, display: &glium::Display) -> glium::texture::Texture2d{
        let (w, h) = (1024, 1024);
        let scale = 16.;
        let mut content = vec![vec![(255u8, 255, 255); w]; h];

        let black = cgmath::Vector3::new(0f64, 0., 0.);
        let white = cgmath::Vector3::new(255f64, 255., 255.);

        for x in 0..w{
            for y in 0..h{
                let (fx, fy) = (x as f64, y as f64);
                let sx = fx / scale;
                let sy = fy / scale;

                let mut value = 6. * self.noise.get([1. * sx, 1. * sy]);
                value += 2. * self.noise.get([2.01 * sx, 2.01 * sy]);
                value += 1. * self.noise.get([4.01 * sx, 4.01 * sy]);
                value += 0.5 * self.noise.get([2.1 * sx, 2.1 * sy]);
                let color = black.lerp(white, value);
                content[x][y] = (color.x.round() as u8, color.y.round() as u8, color.z.round() as u8);
            }
        }

        glium::texture::Texture2d::new(display, content).expect("Couldn't create new texture!")
    }


    pub fn update_chunks(&mut self, position: ChunkPosition) {
        // calculate new chunks to be loaded
        // if distance bigger than current view_distance, clear all loaded chunks and load all surrounding chunks
        // let timer = Instant::now();
        if self.position != position{
            self.position = position;

            for z in (position.z - self.view_distance)..(position.z + self.view_distance) {
                for y in (position.y - self.view_distance)..(position.y + self.view_distance) {
                    for x in (position.x - self.view_distance)..(position.x + self.view_distance) {
                        let chunk_pos = ChunkPosition::new(x, y, z);
                        if !self.chunks.contains_key(&chunk_pos){
                            self.create_chunk(chunk_pos);
                            // self.queue_chunk(chunk_pos);
                        }
                    }
                }
            }

            let positions: Vec<ChunkPosition> = self.chunks.iter().map(|c_ref| c_ref.key().clone()).collect();
            for chunk_pos in positions{
                // let in_bounds = (
                //        chunk_pos.x > position.x - view_distance
                //     && chunk_pos.y > position.y - view_distance
                //     && chunk_pos.z > position.z - view_distance
                //     && chunk_pos.x < position.x + view_distance
                //     && chunk_pos.y < position.y + view_distance
                //     && chunk_pos.z < position.z + view_distance);

                let in_bounds = (position.x - self.view_distance <= chunk_pos.x && position.x + self.view_distance >= chunk_pos.x) &&
                                (position.y - self.view_distance <= chunk_pos.y && position.y + self.view_distance >= chunk_pos.y) &&
                                (position.z - self.view_distance <= chunk_pos.z && position.z + self.view_distance >= chunk_pos.z);

                    if !in_bounds{
                        // println!("Removing chunk!");
                        self.meshes.remove(&chunk_pos);
                        self.chunks.remove(&chunk_pos);
                        self.dirty_neighbors(chunk_pos);
                    }
            }
        }
    }

    pub fn sampled_atlas(&self) -> glium::uniforms::Sampler<'_, TextureArray> {
        // self.texture_atlas
            // .get_texture()
        self.texture_storage
            .get_array()
            .sampled()
            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
            // .wrap_function(glium::uniforms::SamplerWrapFunction::Repeat)
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
                        } else if (position.y * CHUNK_SIZE as isize + y as isize) as f64 == h.floor() {
                            chunk.get_mut_blocks()[x][y][z] = BlockType::Dirt;
                        } else {
                            chunk.get_mut_blocks()[x][y][z] = BlockType::Cobblestone;
                        }
                    }
                }
            }
            // println!("CREATED CHUNK AT: {:?}", position);
            sender.send((position, Arc::new(chunk)));
        });
    }

    pub fn queue_chunk(&mut self, position: ChunkPosition) {
        if  (self.position.x - self.view_distance <= position.x && self.position.x + self.view_distance >= position.x) &&
            (self.position.y - self.view_distance <= position.y && self.position.y + self.view_distance >= position.y) &&
            (self.position.z - self.view_distance <= position.z && self.position.z + self.view_distance >= position.z) {

            self.dirty_queue.push_back(position);
        }
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

                // println!("RECEIVED CHUNK({:?}) TO MESH", position);
                let mut mask = [BlockType::Air; CHUNK_SIZE * CHUNK_SIZE];
                self.mesher.threadpool.execute(move ||{
                    println!("----------------------------------------------");
                    println!("Position: {:?}", position);
                    let mut mesh = MeshData::new();
                    let mut du = [0f32, 0., 0.];
                    let mut dv = [0f32, 0., 0.];

                    for backFace in &[true, false]{
                        for d in 0..3{
                            let u = (d + 1) %3;
                            let v = (d + 2) %3;
                            let mut x = [0, 0, 0];
                            let mut q = [0, 0, 0];
                            q[d] = 1;

                            let side = match d{
                                0 => { if *backFace {Direction::West}else{Direction::East} },
                                1 => { if *backFace {Direction::Down}else{Direction::Up} },
                                2 => { if *backFace {Direction::South}else{Direction::North} },
                                _ => { panic!("Strange side.")}
                            };
                            println!("Dimension: {:?}", side as Direction);

                            x[d] = 0;
                            while x[d] < CHUNK_SIZE as isize{
                                // println!("xd: {:?}", x[d]);
                                let mut n = 0;
                                for xv in 0..CHUNK_SIZE{
                                    x[v] = xv as isize;
                                    for xu in 0..CHUNK_SIZE{
                                        x[u] = xu as isize;

                                        let face1 = if x[d] >= 0 { chunk.get_block(x[0] as usize, x[1] as usize, x[2] as usize) } else { BlockType::Air };
                                        let face2 = if x[d] < CHUNK_SIZE as isize-1 { chunk.get_block((x[0]+q[0]) as usize, (x[1]+q[1]) as usize, (x[2]+q[2]) as usize) } else { BlockType::Air };
                                        mask[n] = if (face1 != BlockType::Air && face2 != BlockType::Air && face1 == face2){ BlockType::Air } else if *backFace { face2 } else { face1 };
                                        // println!("n: {:?} -> {:?}", n, mask[n]);
                                        n+=1;
                                    }
                                }

                                // let block = chunk.get_neighbor(aa[0].try_into().expect("neighbor checking 0"), aa[1].try_into().expect("neighbor checking 0"), aa[2].try_into().expect("neighbor checking 0"), side, neighbors[side as usize].as_ref());
                                x[d] += 1;
                                let mut n = 0;
                                for j in 0..CHUNK_SIZE{
                                    let mut i = 0;
                                    while i < CHUNK_SIZE{
                                        if mask[n] != BlockType::Air{

                                            let mut w = 1;
                                            while i+w < CHUNK_SIZE && mask[n+w] != BlockType::Air && mask[n+w] == mask[n]{
                                                w+=1;
                                            }

                                            let mut done = false;

                                            let mut h = 1;
                                            while j+h < CHUNK_SIZE{
                                                for k in 0..w{
                                                    if mask[n+k+h*CHUNK_SIZE] == BlockType::Air || mask[n+k+h*CHUNK_SIZE] != mask[n]{ done = true; break; }
                                                }
                                                if done { break }
                                                h += 1;
                                            }

                                            println!("Block: {:?}", x);
                                            // if n_block == BlockType::Air{
                                            x[u] = i as isize;
                                            x[v] = j as isize;
                                            du = [0., 0., 0.];
                                            du[u] = w as f32;
                                            dv = [0., 0., 0.];
                                            dv[v] = h as f32;

                                            let xn = [x[0] as f32, x[1] as f32, x[2] as f32];
                                            // let color = [0., 0., 0.];

                                            let block = registry.get_block(mask[n]).expect("Block not found when meshing...").get_coords(side);
                                            let coords = block.greedy_ready();
                                            let normal: [f32; 3] = [block.offset[0] as f32, block.offset[1] as f32, 0.]; //face direction to normal
                                            // println!("Block[{:?}]: {:?}", w, h);
                                            let get_uv = |w, h|{[
                                                [0.,        0.],
                                                [w as f32,  0.],
                                                [0.,        h as f32],
                                                [w as f32,  h as f32]
                                            ]};
                                            let (ix, uvs) = match side{
                                                Direction::North => {([0, 1, 2, 3], get_uv(w, h))},
                                                Direction::South => {([1, 0, 3, 2], get_uv(w, h))},
                                                Direction::West  => {([0, 2, 1, 3], get_uv(h, w))},
                                                Direction::East  => {([2, 0, 3, 1], get_uv(h, w))},
                                                Direction::Up    => {([2, 0, 3, 1], get_uv(h, w))}, //1, 3, 0, 2
                                                Direction::Down  => {([0, 2, 1, 3], get_uv(h, w))},
                                            };
                                            let v = [
                                                xn,
                                                [xn[0] + du[0],         xn[1] + du[1],         xn[2] + du[2]],
                                                [xn[0] + dv[0],         xn[1] + dv[1],         xn[2] + dv[2]],
                                                [xn[0] + du[0] + dv[0], xn[1] + du[1] + dv[1], xn[2] + du[2] + dv[2]]
                                            ];

                                            let vertices = [
                                                Vertex::normal_new(v[ix[0]], uvs[0], normal),
                                                Vertex::normal_new(v[ix[1]], uvs[1], normal),
                                                Vertex::normal_new(v[ix[2]], uvs[2], normal),
                                                Vertex::normal_new(v[ix[3]], uvs[3], normal)
                                            ];
                                            mesh.add_face_raw(vertices, *backFace, mask[n]);
                                            // }
                                            // else{
                                            //     println!("Block at: {:?} Type: {:?} Side: {:?} NeighborType: {:?}", x, block, side, n_block);
                                            // }

                                            //zero
                                            for l in 0..h{
                                                for k in 0..w{
                                                    mask[n+k+l*CHUNK_SIZE] = BlockType::Air;
                                                }
                                            }

                                            i += w;
                                            n += w;
                                        }else{
                                            i += 1;
                                            n += 1;
                                        }
                                    }
                                }

                            }

                        }
                    }
                    // println!("CPosition: {:?}", position);
                    // println!("Mesh vertices len: {:?}", mesh.vertices.len());
                    if mesh.indices.len() != 0 {
                        sender.send((position, mesh));
                    }
                });

                // self.mesher.threadpool.execute(move ||{
                //     let mut mesh = MeshData::new();
                //     for x in 0..CHUNK_SIZE{
                //         for y in 0..CHUNK_SIZE{
                //             for z in 0..CHUNK_SIZE{
                //                 let block_type = chunk.get_blocks()[x][y][z];
                //
                //                 if block_type == BlockType::Air{
                //                     continue;
                //                 }
                //
                //                 let directions = [Direction::North, Direction::South, Direction::East, Direction::West, Direction::Up, Direction::Down];
                //
                //                 for i in 0..directions.len(){
                //                     if chunk.get_neighbor(x, y, z, directions[i], neighbors[i].as_ref()) == BlockType::Air{
                //                         let coords = registry.get_block(block_type).expect("Block not found when meshing...").get_coords(directions[i]);
                //                         let face_data = FaceData::new([x as u8, y as u8, z as u8], block_type, directions[i], *coords);
                //                         mesh.add_face(face_data);
                //
                //                     }
                //                 }
                //
                //             }
                //         }
                //     }
                //
                //     println!("CPosition: {:?}", position);
                //     println!("Mesh vertices len: {:?}", mesh.vertices.len());
                //     sender.send((position, mesh));
                // });
            }
        }
    }

    pub fn clean_queue_size(&self) -> usize{
        self.clean_queue.len()
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

    pub fn get_chunks(&self) -> &ChunkMap{
        &self.chunks
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
