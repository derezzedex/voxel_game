use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::Arc;
use rayon::{ThreadPool, ThreadPoolBuilder};

use crate::game::terrain::block::*;
use crate::game::terrain::chunk::*;
use crate::engine::mesh::*;

pub type ChunkMesherMessage = (ChunkPosition, MeshData);
pub struct ChunkMesher{
    threadpool: ThreadPool,
    sender: Sender<ChunkMesherMessage>,
    receiver: Receiver<ChunkMesherMessage>
}

impl ChunkMesher{
    pub fn new(thread_number: usize) -> Self{
        let (sender, receiver) = mpsc::channel();
        let threadpool = ThreadPoolBuilder::new().num_threads(thread_number).build().expect("Could'nt create ChunkMesher Threadpool");

        Self{
            threadpool,
            sender,
            receiver
        }
    }

    pub fn get_available_meshes(&self) -> mpsc::TryIter<ChunkMesherMessage>{
        self.receiver.try_iter()
    }

    pub fn mesh(&self, position: ChunkPosition, chunk: &Arc<Chunk>, neighbors: [Option<&Arc<Chunk>>; 6]){

        let sender = self.sender.clone();
        self.threadpool.install(move ||{
            let mut mesh = MeshData::new();

            for x in 0..CHUNK_SIZE{
                for y in 0..CHUNK_SIZE{
                    for z in 0..CHUNK_SIZE{
                        let block_type = chunk.get_blocks()[x][y][z];
                        if block_type == BlockType::Air{
                            continue;
                        }
                        if chunk.get_neighbor(x, y, z, Direction::North, neighbors[0]) == BlockType::Air{
                            let face_data = FaceData::new([x as u8, y as u8, z as u8], block_type, Direction::North);
                            mesh.add_face(face_data);

                        }
                        if chunk.get_neighbor(x, y, z, Direction::South, neighbors[1]) == BlockType::Air{
                            let face_data = FaceData::new([x as u8, y as u8, z as u8], block_type, Direction::South);
                            mesh.add_face(face_data);

                        }

                        if chunk.get_neighbor(x, y, z, Direction::East, neighbors[2]) == BlockType::Air{
                            let face_data = FaceData::new([x as u8, y as u8, z as u8], block_type, Direction::East);
                            mesh.add_face(face_data);

                        }

                        if chunk.get_neighbor(x, y, z, Direction::West, neighbors[3]) == BlockType::Air{
                            let face_data = FaceData::new([x as u8, y as u8, z as u8], block_type, Direction::West);
                            mesh.add_face(face_data);

                        }

                        if chunk.get_neighbor(x, y, z, Direction::Up, neighbors[4]) == BlockType::Air{
                            let face_data = FaceData::new([x as u8, y as u8, z as u8], block_type, Direction::Up);
                            mesh.add_face(face_data);

                        }

                        if chunk.get_neighbor(x, y, z, Direction::Down, neighbors[5]) == BlockType::Air{
                            let face_data = FaceData::new([x as u8, y as u8, z as u8], block_type, Direction::Down);
                            mesh.add_face(face_data);

                        }
                    }
                }
            }

            sender.send((position, mesh));
        });
    }
}
