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

    pub fn setup(&mut self){
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
            self.mesh_chunk(&position);
        }
    }

    fn mesh_chunk(&mut self, position: &ChunkPosition){
        let sender = self.mesher.sender.clone();
        if let Some(chunk) = self.chunks.get(position){
            println!("Meshing: {:?}", position);
            let chunk = chunk.value().clone();
            let position = position.clone();
            let mut input = String::new();

            let mut mask = [BlockType::Air; CHUNKSIZE * CHUNKSIZE];
            self.threadpool.execute(move ||{
                let next_block = |pos: [usize;3], axis: usize, backface: bool| -> BlockType{
                    // println!("Pos: {:?}", pos);
                    let mut position = [pos[0] as isize, pos[1] as isize, pos[2] as isize];
                    // position[axis] -= 1;
                    position[axis] += if backface{ -1 } else { 1 };
                    if position[axis] < 0 { return BlockType::Air }
                    chunk.get_block(position[0] as usize, position[1] as usize, position[2] as usize)
                };
                let mut mesh = MeshData::new();
                let mut du = [0f32, 0., 0.];
                let mut dv = [0f32, 0., 0.];

                for backface in &[false, true]{
                    // println!("Backface: {:?}", backface);
                    for d in 0..3{
                        // match d{
                        //     0 => { // west or east
                        //         if *backface{println!("Side: WEST")} else {println!("Side: EAST")}
                        //     },
                        //     1 => { // down or up
                        //         if *backface{println!("Side: DOWN")} else {println!("Side: UP")}
                        //     },
                        //     2 => { //south or north
                        //         if *backface{println!("Side: SOUTH")} else {println!("Side: NORTH")}
                        //     },
                        //     _ => panic!("Unknown dimension")
                        // }

                        let u = (d + 1) %3;
                        let v = (d + 2) %3;
                        let mut x = [0, 0, 0];
                        let mut q = [0, 0, 0];
                        q[d] = 1;
                        // println!("Q: {:?}", q);

                        x[d] = -1;
                        while x[d] <= CHUNKSIZE as isize{
                            let mut n = 0;
                            for xv in 0..CHUNKSIZE{
                                x[v] = xv as isize;
                                for xu in 0..CHUNKSIZE{
                                    x[u] = xu as isize;

                                    let face1 = if x[d] >= 0 { chunk.get_block(x[0] as usize, x[1] as usize, x[2] as usize) } else { BlockType::Air };
                                    let face2 = if x[d] < CHUNKSIZE as isize-1 { chunk.get_block((x[0]+q[0]) as usize, (x[1]+q[1]) as usize, (x[2]+q[2]) as usize) } else { BlockType::Air };
                                    mask[n] = if (face1 != BlockType::Air && face2 != BlockType::Air && face1 == face2){ BlockType::Air } else if *backface { face2 } else { face1 };
                                    n+=1;
                                }
                            }

                            // x[d] += 1;

                            let mut n = 0;
                            for j in 0..CHUNKSIZE{
                                let mut i = 0;
                                while i < CHUNKSIZE{
                                    next_block([x[0] as usize, x[1] as usize, x[2] as usize], d, *backface);
                                    if mask[n] != BlockType::Air {//&& next_block([x[0] as usize, x[1] as usize, x[2] as usize], d, *backface) == BlockType::Air{
                                        let mut w = 1;
                                        while i+w < CHUNKSIZE && mask[n+w] != BlockType::Air && mask[n+w] == mask[n]{
                                            w+=1;
                                        }

                                        let mut done = false;

                                        let mut h = 1;
                                        while j+h < CHUNKSIZE{
                                            for k in 0..w{
                                                if mask[n+k+h*CHUNKSIZE] == BlockType::Air || mask[n+k+h*CHUNKSIZE] != mask[n]{ done = true; break; }
                                            }
                                            if done { break }
                                            h += 1;
                                        }


                                        // println!();
                                        // println!("Mask[{:?}]: {:?}", n, mask[n]);
                                        // println!("Block: {:?}:{:?}", [x[0] as usize, (15-x[1]) as usize, x[2] as usize], chunk.get_block(x[0] as usize, (15-x[1]) as usize, x[2] as usize));
                                        // println!("Next: {:?}:{:?}", [(x[0]+q[0]) as usize, (15-(x[1]+q[1])) as usize,(x[2]+q[2]) as usize], chunk.get_block((x[0]+q[0]) as usize, (15-(x[1]+q[1])) as usize,(x[2]+q[2]) as usize));
                                        // println!("Next1: {:?}", next_block([x[0] as usize, (15-x[1]) as usize, x[2] as usize], d, *backface));
                                        // println!();

                                        x[u] = i as isize;
                                        x[v] = j as isize;
                                        du = [0., 0., 0.];
                                        du[u] = w as f32;
                                        dv = [0., 0., 0.];
                                        dv[v] = h as f32;

                                        let mut xn = [x[0] as f32, x[1] as f32, x[2] as f32];
                                        xn[d] += 1.;

                                        let get_uv = |w, h|{[
                                            [0.,        0.],
                                            [w as f32,  0.],
                                            [0.,        h as f32],
                                            [w as f32,  h as f32]
                                        ]};
                                        let (ix, uvs) = match d{
                                            0 => { // west or east
                                                if *backface{([0, 2, 1, 3], get_uv(h, w))} else {([2, 0, 3, 1], get_uv(h, w))}
                                            },
                                            1 => { // down or up
                                                if *backface{([0, 2, 1, 3], get_uv(h, w))} else {([2, 0, 3, 1], get_uv(h, w))}
                                            },
                                            2 => { //south or north
                                                if *backface{([1, 0, 3, 2], get_uv(w, h))} else {([0, 1, 2, 3], get_uv(w, h))}
                                            },
                                            _ => panic!("Unknown dimension")
                                        };
                                        let v = [
                                            xn,
                                            [xn[0] + du[0],         xn[1] + du[1],         xn[2] + du[2]],
                                            [xn[0] + dv[0],         xn[1] + dv[1],         xn[2] + dv[2]],
                                            [xn[0] + du[0] + dv[0], xn[1] + du[1] + dv[1], xn[2] + du[2] + dv[2]]
                                        ];

                                        let mut indices = vec![2, 0, 1, 1, 3, 2];

                                        let block = if mask[n] == BlockType::Dirt{
                                            [2, 15]
                                        }else if mask[n] == BlockType::Cobblestone{
                                            [0, 14]
                                        }else{
                                            [0, 0]
                                        };

                                        let vertices = vec![
                                            Vertex::new(v[ix[0]], uvs[0], block),
                                            Vertex::new(v[ix[1]], uvs[1], block),
                                            Vertex::new(v[ix[2]], uvs[2], block),
                                            Vertex::new(v[ix[3]], uvs[3], block)
                                        ];
                                        mesh.add(vertices, indices);

                                        // sender.send((position, mesh.clone()));
                                        // std::io::stdin().read_line(&mut input).expect("Input error");

                                        //zero
                                        for l in 0..h{
                                            for k in 0..w{
                                                mask[n+k+l*CHUNKSIZE] = BlockType::Air;
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
                            x[d] += 1;
                        }

                    }
                }
                // println!("CPosition: {:?}", position);
                // println!("Mesh vertices len: {:?}", mesh.vertices.len());
                if mesh.indices.len() != 0 {
                    println!("Sending!");
                    sender.send((position, mesh));
                }
            });
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
                let mut mesh = MeshData::new();
                let mut du = [0f32, 0., 0.];
                let mut dv = [0f32, 0., 0.];

                for backface in &[false, true]{
                    for d in 0..3{
                        match d{
                            0 => { // west or east
                                if *backface{println!("Side: WEST")} else {println!("Side: EAST")}
                            },
                            1 => { // down or up
                                if *backface{println!("Side: DOWN")} else {println!("Side: UP")}
                            },
                            2 => { //south or north
                                if *backface{println!("Side: SOUTH")} else {println!("Side: NORTH")}
                            },
                            _ => panic!("Unknown dimension")
                        }

                        let u = (d + 1) %3;
                        let v = (d + 2) %3;
                        let mut x = [0, 0, 0];
                        let mut q = [0, 0, 0];
                        q[d] = 1;

                        x[d] = 0;
                        while x[d] < CHUNKSIZE{
                            let mut n = 0;
                            for xv in 0..CHUNKSIZE{
                                x[v] = xv;
                                for xu in 0..CHUNKSIZE{
                                    x[u] = xu;

                                    let face_pos = if *backface { [15-x[0]-q[0], 15-x[1]-q[1], x[2]-q[2]] } else { [15-x[0]+q[0], 15-x[1]+q[1], x[2]+q[2]] };
                                    mask[n] = if chunk.get_block(face_pos[0], face_pos[1], face_pos[2]) == BlockType::Air { chunk.get_block(15-x[0], 15-x[1], x[2])} else { BlockType::Air };

                                    n+=1;
                                }
                            }

                            // x[d] += 1;

                            let mut n = 0;
                            for j in 0..CHUNKSIZE{
                                let mut i = 0;
                                while i < CHUNKSIZE{
                                    if mask[n] != BlockType::Air {//&& next_block([x[0] as usize, x[1] as usize, x[2] as usize], d, *backface) == BlockType::Air{
                                        let mut w = 1;
                                        while i+w < CHUNKSIZE && mask[n+w] != BlockType::Air && mask[n+w] == mask[n]{
                                            w+=1;
                                        }

                                        let mut done = false;

                                        let mut h = 1;
                                        while j+h < CHUNKSIZE{
                                            for k in 0..w{
                                                if mask[n+k+h*CHUNKSIZE] == BlockType::Air || mask[n+k+h*CHUNKSIZE] != mask[n]{ done = true; break; }
                                            }
                                            if done { break }
                                            h += 1;
                                        }

                                        x[u] = i;
                                        x[v] = j;
                                        du = [0., 0., 0.];
                                        du[u] = w as f32;
                                        dv = [0., 0., 0.];
                                        dv[v] = h as f32;

                                        let mut xn = [x[0] as f32, x[1] as f32, x[2] as f32];
                                        xn[d] += 1.;

                                        let get_uv = |w, h|{[
                                            [0.,        0.],
                                            [w as f32,  0.],
                                            [0.,        h as f32],
                                            [w as f32,  h as f32]
                                        ]};
                                        let (ix, uvs) = match d{
                                            0 => { // west or east
                                                if *backface{([0, 2, 1, 3], get_uv(h, w))} else {([2, 0, 3, 1], get_uv(h, w))}
                                            },
                                            1 => { // down or up
                                                if *backface{([0, 2, 1, 3], get_uv(h, w))} else {([2, 0, 3, 1], get_uv(h, w))}
                                            },
                                            2 => { //south or north
                                                if *backface{([1, 0, 3, 2], get_uv(w, h))} else {([0, 1, 2, 3], get_uv(w, h))}
                                            },
                                            _ => panic!("Unknown dimension")
                                        };
                                        let v = [
                                            xn,
                                            [xn[0] + du[0],         xn[1] + du[1],         xn[2] + du[2]],
                                            [xn[0] + dv[0],         xn[1] + dv[1],         xn[2] + dv[2]],
                                            [xn[0] + du[0] + dv[0], xn[1] + du[1] + dv[1], xn[2] + du[2] + dv[2]]
                                        ];

                                        let mut indices = vec![2, 0, 1, 1, 3, 2];

                                        let block = if mask[n] == BlockType::Dirt{
                                            [2, 15]
                                        }else if mask[n] == BlockType::Cobblestone{
                                            [0, 14]
                                        }else{
                                            [0, 0]
                                        };

                                        let vertices = vec![
                                            Vertex::new(v[ix[0]], uvs[0], block),
                                            Vertex::new(v[ix[1]], uvs[1], block),
                                            Vertex::new(v[ix[2]], uvs[2], block),
                                            Vertex::new(v[ix[3]], uvs[3], block)
                                        ];
                                        mesh.add(vertices, indices);

                                        // sender.send((position, mesh.clone()));
                                        // std::io::stdin().read_line(&mut input).expect("Input error");

                                        //zero
                                        for l in 0..h{
                                            for k in 0..w{
                                                mask[n+k+l*CHUNKSIZE] = BlockType::Air;
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
                            x[d] += 1;
                        }

                    }
                }
                // println!("CPosition: {:?}", position);
                // println!("Mesh vertices len: {:?}", mesh.vertices.len());
                if mesh.indices.len() != 0 {
                    println!("Sending!");
                    sender.send((position, mesh));
                }
            });
        }
    }
}
