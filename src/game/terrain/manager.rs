use crate::engine::Vertex;
use crate::engine::mesh::{Mesh, MeshData};
use super::chunk::{ChunkPosition, Chunk, BlockType};
use super::chunk::CHUNKSIZE;

use slice_deque::SliceDeque;
use dashmap::{DashMap};
use dashmap::mapref::one::Ref;
use cgmath::Point3;
use noise::{Fbm, NoiseFn, Seedable};
use uvth::{ThreadPoolBuilder, ThreadPool};
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

pub type MeshMessage = (ChunkPosition, MeshData);
pub struct ChunkMesher{
    sender: Sender<MeshMessage>,
    receiver: Receiver<MeshMessage>
}

impl ChunkMesher{
    pub fn new() -> Self{
        let (sender, receiver) = mpsc::channel();

        Self{
            sender,
            receiver
        }
    }
}

pub type ChunkRef<'a> = Ref<'a, ChunkPosition, Arc<Chunk>>;
pub type ChunkMap = DashMap<ChunkPosition, Arc<Chunk>>;
pub type ChunkMeshMap = DashMap<ChunkPosition, Mesh>;
pub struct TerrainManager{
    chunks: Arc<ChunkMap>,
    threadpool: ThreadPool,
    mesher: ChunkMesher,
    dirty: SliceDeque<ChunkPosition>,
    meshes: ChunkMeshMap,
    noise: Arc<Fbm>
}

impl TerrainManager{
    pub fn new() -> Self{
        let chunks = Arc::new(ChunkMap::default());
        let meshes = ChunkMeshMap::default();

        let threadpool = ThreadPoolBuilder::new()
            .name("TerrainManager".to_string())
            .build();
        let mesher = ChunkMesher::new();
        let dirty = SliceDeque::new();

        let noise = Arc::new(Fbm::new().set_seed(10291302));

        Self{
            chunks,
            threadpool,
            mesher,
            dirty,
            meshes,
            noise
        }
    }

    pub fn setup(&mut self, display: &glium::Display){
        let distance = 1;
        for z in -distance..distance{
            for y in -distance..=0{
                for x in -distance..distance{
                    self.generate_chunk(ChunkPosition::new(x, y, z));
                }
            }
        }
        // self.generate_chunk(ChunkPosition::new(0, 0, 0));
        // self.generate_chunk(ChunkPosition::new(0, -1, 0));
    }

    pub fn update_meshes(&mut self, display: &glium::Display){
        // self.pop_dirty();
        for c_ref in self.chunks.clone().iter(){
            if !self.meshes.contains_key(c_ref.key()){
                self.reworked_meshing(c_ref.key());
            }
        }

        let received: Vec<_> = self.mesher.receiver.try_iter().collect();
        for (position, data) in &received{
            let mesh = data.build(display);
            self.meshes.insert(*position, mesh);
        }
    }

    pub fn get_chunks(&self) -> &ChunkMap{
        &self.chunks
    }

    pub fn get_meshes(&self) -> &ChunkMeshMap{
        &self.meshes
    }

    fn chunk_neighbors(&self, position: &ChunkPosition) -> [Option<ChunkRef>; 6]{
        let east = self.chunks.get(&Point3::new(position.x+1, position.y, position.z));           // East
        let west = self.chunks.get(&Point3::new(position.x-1, position.y, position.z));           // West
        let top = self.chunks.get(&Point3::new(position.x, position.y + 1, position.z));            // Top
        let bottom = self.chunks.get(&Point3::new(position.x, position.y - 1, position.z));         // Bottom
        let north = self.chunks.get(&Point3::new(position.x, position.y, position.z + 1));      // North
        let south = self.chunks.get(&Point3::new(position.x, position.y, position.z - 1));      // South

        [east, west, top, bottom, north, south]
    }

    fn generate_chunk(&mut self, position: ChunkPosition){
        let chunks = self.chunks.clone();
        let noise = self.noise.clone();
        self.threadpool.execute(move ||{
            let mut chunk = Chunk::new(BlockType::Air);
            for z in 0..CHUNKSIZE{
                for y in 0..CHUNKSIZE{
                    for x in 0..CHUNKSIZE{
                        if ((position.y * CHUNKSIZE as isize + y as isize) as f64) == CHUNKSIZE as f64 - 1.{
                            chunk.set_block(x, y, z, BlockType::Dirt);
                        }else if ((position.y * CHUNKSIZE as isize + y as isize) as f64) < CHUNKSIZE as f64 - 1.{
                            chunk.set_block(x, y, z, BlockType::Cobblestone);
                        }
                    }
                }
            }

            chunks.insert(position, Arc::new(chunk));
        });
        // std::thread::sleep_ms(1000);
        // self.dirty.push_back(position);
    }

    fn pop_dirty(&mut self){
        if let Some(position) = self.dirty.pop_front(){
            self.reworked_meshing(&position);
        }
    }

    fn reworked_meshing(&mut self, position: &ChunkPosition){
        if let Some(chunk) = self.chunks.get(position){
            let sender = self.mesher.sender.clone();
            // println!("Meshing: {:?}", position);
            let chunk = chunk.value().clone();
            let neighbors: Vec<Option<Arc<Chunk>>> = self.chunk_neighbors(position).iter().map(|n_ref| n_ref.as_ref().and_then(|inner| Some(Arc::clone(inner)))).collect();
            let position = position.clone();
            let mut input = String::new();

            let mut mask = [BlockType::Air; CHUNKSIZE * CHUNKSIZE];
            self.threadpool.execute(move ||{
                let mut mesh = MeshData::new();

                for backface in &[false, true]{
                    for dim in 0..3{
                        let u = (dim + 1) % 3;
                        let v = (dim + 2) % 3;
                        let mut dir = [0, 0, 0];
                        dir[dim] = if !*backface {-1} else{1};

                        let mut current = [0isize, 0, 0];
                        // goes through each 'layer' of blocks in that dim
                        for layer in 0..CHUNKSIZE as isize{
                            let mut mask = [[false; CHUNKSIZE]; CHUNKSIZE];
                            current[dim] = layer; //sets the current layer
                            for d1 in 0..CHUNKSIZE as isize{
                                current[v] = d1;
                                for d2 in 0..CHUNKSIZE as isize{
                                    current[u] = d2;
                                    let current_block = chunk.check_block(current[0], current[1], current[2], neighbors.clone());

                                    let (mut w, mut h) = (1, 1);
                                    // if not masked already, not air and facing air
                                    if !mask[d1 as usize][d2 as usize] && current_block != BlockType::Air && chunk.check_block(current[0]+dir[0], current[1]+dir[1], current[2]+dir[2], neighbors.clone()) == BlockType::Air{
                                        mask[d1 as usize][d2 as usize] = true;
                                        let mut next = current;
                                        next[u] += 1;
                                        // if next block is equal current block, start increasing mesh size and not meshed already too...
                                        if current_block == chunk.check_block(next[0], next[1], next[2], neighbors.clone()) && !mask[d1 as usize][(d2+1) as usize]{
                                            w += 1;
                                            mask[d1 as usize][(d2+1) as usize] = true;
                                            for i in d2+2..CHUNKSIZE as isize{ // for each remaining block in the current row
                                                let mut next2 = next;
                                                next2[u] = i;
                                                if chunk.check_block(next2[0], next2[1], next2[2], neighbors.clone()) == current_block{ w += 1; mask[d1 as usize][i as usize] = true; /*println!("mask: {:?}", mask)*/} else { break }
                                            }
                                        }

                                        'row: for j in d1+1..CHUNKSIZE as isize{ // for each row in the remaining rows
                                            let mut next2 = next;
                                            next2[v] = j;
                                            for i in d2..d2+w as isize{ // for each remaining block in the current row
                                                next2[u] = i;
                                                if chunk.check_block(next2[0], next2[1], next2[2], neighbors.clone()) != current_block { break 'row }
                                            }
                                            for i in d2..d2+w as isize{
                                                mask[j as usize][i as usize] = true;
                                            }
                                            h += 1;
                                        }

                                        let get_uv = |w, h|{[
                                            [0.,        0.],
                                            [w as f32,  0.],
                                            [0.,        h as f32],
                                            [w as f32,  h as f32]
                                        ]};

                                        let (w, h) = (w as f32, h as f32);
                                        let (ix, uvs) = match dim{
                                            0 => { // east or west
                                                if *backface{([0, 2, 1, 3], get_uv(h, w))} else {([2, 0, 3, 1], get_uv(h, w))}
                                            },
                                            1 => { // up or down
                                                if *backface{([0, 2, 1, 3], get_uv(h, w))} else {([2, 0, 3, 1], get_uv(h, w))} //3, 1, 2, 0
                                            },
                                            2 => { //north or south
                                                if *backface{([1, 0, 3, 2], get_uv(w, h))} else {([0, 1, 2, 3], get_uv(w, h))}
                                            },
                                            _ => panic!("Unknown dimension")
                                        };

                                        let mut x = [current[0] as f32, current[1] as f32, current[2]  as f32];
                                        if *backface { x[dim] += 1.; }
                                        let mut du = [0., 0., 0.];
                                        du[u] = w;
                                        let mut dv = [0., 0., 0.];
                                        dv[v] = h;

                                        let block = if current_block == BlockType::Dirt{
                                            [2, 15]
                                        }else if current_block == BlockType::Cobblestone{
                                            [0, 14]
                                        }else{
                                            [0, 0]
                                        };

                                        let v = [
                                            x,
                                            [x[0] + du[0],         x[1] + du[1],         x[2] + du[2]],
                                            [x[0] + dv[0],         x[1] + dv[1],         x[2] + dv[2]],
                                            [x[0] + du[0] + dv[0], x[1] + du[1] + dv[1], x[2] + du[2] + dv[2]]
                                        ];

                                        let vertices = vec![
                                            Vertex::new(v[ix[0]], uvs[0], block),
                                            Vertex::new(v[ix[1]], uvs[1], block),
                                            Vertex::new(v[ix[2]], uvs[2], block),
                                            Vertex::new(v[ix[3]], uvs[3], block)
                                        ];

                                        let mut indices = vec![2, 3, 1, 1, 0, 2];
                                        mesh.add(vertices, indices);
                                    }

                                }
                            }

                        }
                    }
                }
                if mesh.indices.len() != 0 {
                    // println!("Sending!");
                    sender.send((position, mesh));
                }
            });
        }
    }
}
