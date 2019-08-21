use crate::game::terrain::block_registry::BlockRegistry;
use crate::utils::texture::TextureAtlas;
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

    pub fn mesh(&self, position: ChunkPosition, chunk: &Arc<Chunk>, neighbors: [Option<&Arc<Chunk>>; 6], atlas: &TextureAtlas, registry: &BlockRegistry){

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

                        let directions = [Direction::North, Direction::South, Direction::East, Direction::West, Direction::Up, Direction::Down];

                        for i in 0..directions.len(){
                            if chunk.get_neighbor(x, y, z, directions[i], neighbors[i]) == BlockType::Air{
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
