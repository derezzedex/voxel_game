use crate::engine::Vertex;
use crate::engine::mesh::{Mesh, MeshData};
use super::chunk::{ChunkPosition, Chunk, BlockType};
use super::chunk::CHUNKSIZE;

use slice_deque::SliceDeque;
use dashmap::{DashMap};
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

pub type ChunkMap = DashMap<ChunkPosition, Arc<Chunk>>;
pub type ChunkMeshMap = DashMap<ChunkPosition, Mesh>;
pub struct TerrainManager{
    chunks: Arc<ChunkMap>,
    threadpool: ThreadPool,
    mesher: ChunkMesher,
    dirty: SliceDeque<ChunkPosition>,
    meshes: ChunkMeshMap,
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

        Self{
            chunks,
            threadpool,
            mesher,
            dirty,
            meshes,
        }
    }

    pub fn setup(&mut self, display: &glium::Display){
        self.generate_chunk(ChunkPosition::new(0, 0, 0));
    }

    pub fn update_meshes(&mut self, display: &glium::Display){
        self.pop_dirty();
        let received: Vec<_> = self.mesher.receiver.try_iter().collect();
        for (position, data) in &received{
            println!("Building...");
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

    fn generate_chunk(&mut self, position: ChunkPosition){
        let mut chunk = Chunk::new(BlockType::Dirt);
        for z in 0..CHUNKSIZE{
            for y in 0..CHUNKSIZE{
                for x in 0..CHUNKSIZE{
                    if (x == 0 || x == CHUNKSIZE-1) && (y == 0 || y == CHUNKSIZE-1) && (z == 0 || z == CHUNKSIZE-1){
                        chunk.set_block(x, y, z, BlockType::Cobblestone);
                    }
                }
            }
        }
        // chunk.set_block(1, 0, 1, BlockType::Air);
        self.chunks.insert(position, Arc::new(chunk));
        self.dirty.push_back(position);
    }

    fn pop_dirty(&mut self){
        if let Some(position) = self.dirty.pop_front(){
            self.reworked_meshing(&position);
        }
    }

    fn reworked_meshing(&mut self, position: &ChunkPosition){
        let sender = self.mesher.sender.clone();
        if let Some(chunk) = self.chunks.get(position){
            println!("Meshing: {:?}", position);
            let chunk = chunk.value().clone();
            let position = position.clone();
            let mut input = String::new();

            let mut mask = [BlockType::Air; CHUNKSIZE * CHUNKSIZE];
            self.threadpool.execute(move ||{
                let get_block = |x: isize, y: isize, z: isize| -> BlockType{
                    if x < 0 || x > 15{ return BlockType::Air }
                    if y < 0 || y > 15{ return BlockType::Air }
                    if z < 0 || z > 15{ return BlockType::Air }
                    chunk.get_block(x as usize,y as usize,z as usize)
                };
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
                                    let current_block = get_block(current[0], current[1], current[2]);

                                    let (mut w, mut h) = (1, 1);
                                    // if not masked already, not air and facing air
                                    if !mask[d1 as usize][d2 as usize] && current_block != BlockType::Air && get_block(current[0]+dir[0], current[1]+dir[1], current[2]+dir[2]) == BlockType::Air{
                                        mask[d1 as usize][d2 as usize] = true;
                                        let mut next = current;
                                        next[u] += 1;
                                        // if next block is equal current block, start increasing mesh size and not meshed already too...
                                        if current_block == get_block(next[0], next[1], next[2]) && !mask[d1 as usize][(d2+1) as usize]{
                                            w += 1;
                                            mask[d1 as usize][(d2+1) as usize] = true;
                                            for i in d2+2..CHUNKSIZE as isize{ // for each remaining block in the current row
                                                let mut next2 = next;
                                                next2[u] = i;
                                                if get_block(next2[0], next2[1], next2[2]) == current_block{ w += 1; mask[d1 as usize][i as usize] = true; /*println!("mask: {:?}", mask)*/} else { break }
                                            }
                                        }

                                        'row: for j in d1+1..CHUNKSIZE as isize{ // for each row in the remaining rows
                                            let mut next2 = next;
                                            next2[v] = j;
                                            for i in d2..d2+w as isize{ // for each remaining block in the current row
                                                next2[u] = i;
                                                if get_block(next2[0], next2[1], next2[2]) != current_block { break 'row }
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
                    println!("Sending!");
                    sender.send((position, mesh));
                }
            });
        }
    }
}
