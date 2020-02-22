use super::chunk::CHUNKSIZE;
use super::chunk::{Chunk, ChunkPosition};
use crate::engine::mesh::{Mesh, MeshData};
use crate::engine::Vertex;
use crate::game::registry::Registry;
use crate::game::terrain::block::Direction;

use cgmath::Point3;
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use noise::{Fbm, NoiseFn, Seedable};
use std::convert::TryFrom;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use uvth::{ThreadPool, ThreadPoolBuilder};

pub fn range_map(s: f64, a: [f64; 2], b: [f64; 2]) -> f64 {
    b[0] + ((s - a[0]) * (b[1] - b[0])) / (a[1] - a[0])
}

pub type MeshMessage = (ChunkPosition, MeshData);
pub struct ChunkMesher {
    sender: Sender<MeshMessage>,
    receiver: Receiver<MeshMessage>,
}

impl ChunkMesher {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();

        Self { sender, receiver }
    }
}

pub const LOAD_DISTANCE: usize = 2;
pub const VIEW_DISTANCE: usize = 1;

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
            .name("TerrainManager".to_string())
            .build();
        let mesher = ChunkMesher::new();

        let noise = Arc::new(Fbm::new().set_seed(10291302));
        let registry = registry.clone();
        let position = ChunkPosition::new(-10, -10, -10);

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
        let distance = LOAD_DISTANCE as isize;
        for z in -distance..distance {
            for x in -distance..distance {
                self.generate_chunk(ChunkPosition::new(x, -1, z));
            }
        }
    }

    pub fn update(&mut self, position: ChunkPosition) {
        let distance = VIEW_DISTANCE as isize;
        if position != self.position{
            println!("Moved from {:?} to {:?}", self.position, position);
            let mut delta = position - self.position;
            self.position = position;
            delta.x = delta.x.abs();
            delta.z = delta.z.abs();
            println!("Delta: {:?}", delta);
            if delta.x > distance || delta.z > distance {
                // unimplemented!("Too fast");
            }else{
                for i in -distance..distance{
                    if delta.x > delta.z{
                        self.generate_chunk(ChunkPosition::new(position.x + 1, -1, position.z + i));
                    }else{
                        self.generate_chunk(ChunkPosition::new(position.x + i, -1, position.z + 1));
                    }
                }
            }
        }
    }

    pub fn mesh_chunks(&mut self, display: &glium::Display) {
        //temp
        for c_ref in self.chunks.clone().iter(){
            let position = c_ref.key();
            if !self.meshes.contains_key(position) && !self.visible_chunks.contains_key(position){
                self.send_to_mesh(position);
                self.visible_chunks.insert(*position, ());
            }
        }

        let received: Vec<_> = self.mesher.receiver.try_iter().collect();
        for (position, data) in &received {
            let mesh = data.build(display);
            self.meshes.insert(*position, mesh);
        }
    }

    pub fn get_chunks(&self) -> &ChunkMap {
        &self.chunks
    }

    pub fn get_meshes(&self) -> &ChunkMeshMap {
        &self.meshes
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
            for z in 0..CHUNKSIZE {
                for y in 0..CHUNKSIZE {
                    for x in 0..CHUNKSIZE {
                        let nx = (position.x * CHUNKSIZE as isize + x as isize) as f64;
                        let ny = (position.y * CHUNKSIZE as isize + y as isize) as f64;
                        let nz = (position.z * CHUNKSIZE as isize + z as isize) as f64;

                        let height = range_map(
                            2. * noise.get([nx * 0.01, nz * 0.01]),
                            [-1., 1.],
                            [0., CHUNKSIZE as f64 / 2.],
                        )
                        .round();

                        if ny == -height {
                            let block = registry.block_registry().id_of("grass").unwrap_or(1);
                            chunk.set_block(x, y, z, block);
                        } else if ny == -(CHUNKSIZE as f64) {
                            let block = registry.block_registry().id_of("bedrock").unwrap_or(1);
                            chunk.set_block(x, y, z, block);
                        } else if ny < -height - 3. {
                            let block = registry.block_registry().id_of("stone").unwrap_or(1);
                            chunk.set_block(x, y, z, block);
                        } else if ny < -height {
                            let block = registry.block_registry().id_of("dirt").unwrap_or(1);
                            chunk.set_block(x, y, z, block);
                        }
                    }
                }
            }

            chunks.insert(position, Arc::new(chunk));
        });
    }

    fn send_to_mesh(&mut self, position: &ChunkPosition) {
        if let Some(chunk) = self.chunks.get(position) {
            let sender = self.mesher.sender.clone();
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

                for backface in &[false, true] {
                    for dim in 0..3 {
                        let u = (dim + 1) % 3;
                        let v = (dim + 2) % 3;
                        let mut dir = [0, 0, 0];
                        dir[dim] = if !*backface { -1 } else { 1 };

                        let mut current = [0isize, 0, 0];
                        // goes through each 'layer' of blocks in that dim
                        for layer in 0..CHUNKSIZE as isize {
                            let mut mask = [[false; CHUNKSIZE]; CHUNKSIZE];
                            current[dim] = layer; //sets the current layer
                            for d1 in 0..CHUNKSIZE as isize {
                                current[v] = d1;
                                for d2 in 0..CHUNKSIZE as isize {
                                    current[u] = d2;
                                    let current_block = chunk.check_block(
                                        current[0],
                                        current[1],
                                        current[2],
                                        neighbors.clone(),
                                    );

                                    let (mut w, mut h) = (1, 1);
                                    // if not masked already, not air and facing air
                                    if !mask[d1 as usize][d2 as usize]
                                        && current_block != 0
                                        && chunk.check_block(
                                            current[0] + dir[0],
                                            current[1] + dir[1],
                                            current[2] + dir[2],
                                            neighbors.clone(),
                                        ) == 0
                                    {
                                        mask[d1 as usize][d2 as usize] = true;
                                        let mut next = current;
                                        next[u] += 1;
                                        // if next block is equal current block, start increasing mesh size and not meshed already too...
                                        if ((d2 + 1) as usize) < CHUNKSIZE {
                                            if current_block
                                                == chunk.check_block(
                                                    next[0],
                                                    next[1],
                                                    next[2],
                                                    neighbors.clone(),
                                                )
                                                && !mask[d1 as usize][(d2 + 1) as usize]
                                            {
                                                w += 1;
                                                mask[d1 as usize][(d2 + 1) as usize] = true;
                                                for i in d2 + 2..CHUNKSIZE as isize {
                                                    // for each remaining block in the current row
                                                    let mut next2 = next;
                                                    next2[u] = i;
                                                    if chunk.check_block(
                                                        next2[0],
                                                        next2[1],
                                                        next2[2],
                                                        neighbors.clone(),
                                                    ) == current_block
                                                        && !mask[d1 as usize][i as usize]
                                                    {
                                                        w += 1;
                                                        mask[d1 as usize][i as usize] = true;
                                                    /*println!("mask: {:?}", mask)*/
                                                    } else {
                                                        break;
                                                    }
                                                }
                                            }
                                        }

                                        'row: for j in d1 + 1..CHUNKSIZE as isize {
                                            // for each row in the remaining rows
                                            let mut next2 = next;
                                            next2[v] = j;
                                            for i in d2..d2 + w as isize {
                                                // for each remaining block in the current row
                                                next2[u] = i;
                                                if chunk.check_block(
                                                    next2[0],
                                                    next2[1],
                                                    next2[2],
                                                    neighbors.clone(),
                                                ) != current_block
                                                    || mask[i as usize][j as usize]
                                                {
                                                    break 'row;
                                                }
                                            }
                                            for i in d2..d2 + w as isize {
                                                mask[j as usize][i as usize] = true;
                                            }
                                            h += 1;
                                        }

                                        let get_uv = |w, h| {
                                            [
                                                [0., 0.],
                                                [w as f32, 0.],
                                                [0., h as f32],
                                                [w as f32, h as f32],
                                            ]
                                        };

                                        let (w, h) = (w as f32, h as f32);
                                        let (ix, uvs) = match dim {
                                            0 => {
                                                // east or west
                                                if *backface {
                                                    ([0, 2, 1, 3], get_uv(h, w))
                                                } else {
                                                    ([2, 0, 3, 1], get_uv(h, w))
                                                }
                                            }
                                            1 => {
                                                // up or down
                                                if *backface {
                                                    ([0, 2, 1, 3], get_uv(h, w))
                                                } else {
                                                    ([2, 0, 3, 1], get_uv(h, w))
                                                } //3, 1, 2, 0
                                            }
                                            2 => {
                                                //north or south
                                                if *backface {
                                                    ([1, 0, 3, 2], get_uv(w, h))
                                                } else {
                                                    ([0, 1, 2, 3], get_uv(w, h))
                                                }
                                            }
                                            _ => panic!("Unknown dimension"),
                                        };

                                        let mut x = [
                                            current[0] as f32,
                                            current[1] as f32,
                                            current[2] as f32,
                                        ];
                                        if *backface {
                                            x[dim] += 1.;
                                        }
                                        let mut du = [0., 0., 0.];
                                        du[u] = w;
                                        let mut dv = [0., 0., 0.];
                                        dv[v] = h;

                                        let block = if let Some(block_data) =
                                            registry.block_registry().by_id(current_block as usize)
                                        {
                                            block_data.get_face(
                                                Direction::try_from(u).unwrap_or(Direction::East),
                                            )
                                        } else {
                                            [0, 1]
                                        };
                                        // let block = if current_block == BlockType::Dirt{
                                        //     [2, 15]
                                        // }else if current_block == BlockType::Cobblestone{
                                        //     [0, 14]
                                        // }else{
                                        //     [0, 0]
                                        // };

                                        let v = [
                                            x,
                                            [x[0] + du[0], x[1] + du[1], x[2] + du[2]],
                                            [x[0] + dv[0], x[1] + dv[1], x[2] + dv[2]],
                                            [
                                                x[0] + du[0] + dv[0],
                                                x[1] + du[1] + dv[1],
                                                x[2] + du[2] + dv[2],
                                            ],
                                        ];

                                        let vertices = vec![
                                            Vertex::new(v[ix[0]], uvs[0], block),
                                            Vertex::new(v[ix[1]], uvs[1], block),
                                            Vertex::new(v[ix[2]], uvs[2], block),
                                            Vertex::new(v[ix[3]], uvs[3], block),
                                        ];

                                        let indices = vec![2, 3, 1, 1, 0, 2];
                                        mesh.add(vertices, indices);
                                    }
                                }
                            }
                        }
                    }
                }

                if mesh.indices.len() != 0 {
                    // println!("Sending!");
                    sender
                        .send((position, mesh))
                        .expect("Couldn't send chunk to main thread!");
                }
            });
        }
    }
}
