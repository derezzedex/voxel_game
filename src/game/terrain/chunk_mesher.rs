use crate::game::terrain::chunk_manager::ChunkManager;
use std::collections::VecDeque;
use dashmap::Iter;
use crate::game::terrain::chunk_manager::ChunkRef;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::Arc;

use dashmap::DashMap;
use scoped_threadpool::Pool;
use threadpool::ThreadPool;

use crate::game::terrain::block_registry::BlockRegistry;
use crate::utils::texture::TextureAtlas;
use crate::game::terrain::block::*;
use crate::game::terrain::chunk::*;
use crate::engine::mesh::*;

pub type ChunkMesherMessage = (ChunkPosition, MeshData);
pub struct ChunkMesher{
    threadpool: Pool,
    sender: Sender<ChunkMesherMessage>,
    receiver: Receiver<ChunkMesherMessage>,
    meshes: DashMap<ChunkPosition, Mesh>,

    mesh_queue: VecDeque<ChunkMesherMessage>,
}

impl ChunkMesher{
    pub fn new(thread_number: u32) -> Self{
        let (sender, receiver) = mpsc::channel();
        let threadpool = Pool::new(thread_number);
        let meshes = DashMap::default();

        let mesh_queue = VecDeque::new();

        Self{
            threadpool,
            sender,
            receiver,
            meshes,
            mesh_queue
        }
    }

    pub fn get_meshes_iter<'a>(&'a self) -> Iter<'a, ChunkPosition, Mesh>{
        self.meshes.iter()
    }

    pub fn get_meshes(&self) -> &DashMap<ChunkPosition, Mesh>{
        &self.meshes
    }

    pub fn get_mut_meshes(&mut self) -> &mut DashMap<ChunkPosition, Mesh>{
        &mut self.meshes
    }

    pub fn mesh_queue_number(&self) -> usize{
        self.mesh_queue.len()
    }

    pub fn get_available_meshes(&self) -> mpsc::TryIter<ChunkMesherMessage>{
        self.receiver.try_iter()
    }

    pub fn update_mesh_queue(&mut self){
        let mut new_meshes: VecDeque<_> = self.get_available_meshes().collect();
        // println!("Received: {:?}", new_meshes.len());
        self.mesh_queue.append(&mut new_meshes);
    }

    pub fn dequeue_mesh(&mut self, display: &glium::Display){
        if let Some((pos, mesh)) = self.mesh_queue.pop_front(){
            let built_mesh = mesh.build(display);
            self.meshes.insert(pos, built_mesh);
        }
    }

    // pub fn mesh_available_meshes(&self, display: &glium::Display){
    //     for (pos, mesh) in self.get_available_meshes(){
    //         let built_mesh = mesh.build(display);
    //         self.meshes.insert(pos, built_mesh);
    //     }
    // }

    pub fn mesh(&mut self, position: ChunkPosition, chunk: &Arc<Chunk>, neighbors: [Option<ChunkRef>; 6], atlas: &TextureAtlas, registry: &BlockRegistry){

        let sender = self.sender.clone();

        self.threadpool.scoped(|scope|{
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
                            let neighbor = neighbors[i].as_ref().and_then(|inner| Some(&**inner));
                            if chunk.get_neighbor(x, y, z, directions[i], neighbor) == BlockType::Air{
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
