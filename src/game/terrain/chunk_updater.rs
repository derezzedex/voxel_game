use crate::game::terrain::block::BlockType;
use crate::game::terrain::chunk::{Chunk, ChunkPosition, CHUNK_SIZE};
use noise::{NoiseFn, Perlin, Seedable};
use rayon::ThreadPoolBuilder;
use rayon::ThreadPool as RayonThreadPool;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

use std::collections::VecDeque;

use threadpool::{ThreadPool, Builder};

pub enum ChunkOperation{
    Creation,
    Remotion,
    Update
}

pub type ChunkUpdaterMessage = (ChunkPosition, Arc<Chunk>, ChunkOperation);
pub struct ChunkUpdater {
    threadpool: ThreadPool,
    sender: Sender<ChunkUpdaterMessage>,
    receiver: Receiver<ChunkUpdaterMessage>,
    available: VecDeque<ChunkUpdaterMessage>,
    noise: Perlin,
}

impl ChunkUpdater {
    pub fn new(thread_number: usize) -> Self {
        let noise = Perlin::new().set_seed(1102130);

        let (sender, receiver) = mpsc::channel();
        // let threadpool = ThreadPoolBuilder::new()
        //     // .num_threads(thread_number)
        //     .build()
        //     .expect("Couldn't create ChunkUpdate Threadpool");
        let threadpool = Builder::new()
            .num_threads(thread_number)
            .build();

        Self {
            threadpool,
            sender,
            receiver,
            available: VecDeque::new(),
            noise
        }
    }

    pub fn available_chunk_number(&self) -> usize{
        self.available.len()
    }

    pub fn retrieve_first(&mut self) -> Option<ChunkUpdaterMessage>{
        self.available.pop_front()
    }

    // pub fn retrieve_chunk_at(&mut self, i: usize) -> Option<ChunkUpdaterMessage>{
    //     if i < self.available.len(){
    //         Some(self.available.remove(i))
    //     }else{
    //         None
    //     }
    // }

    pub fn remove_late_chunks(&mut self, position: ChunkPosition, view_distance: isize){
        self.available.retain(|(pos, chunk, op)|{
            (pos.x > position.x - view_distance
            && pos.y > position.y - view_distance
            && pos.z > position.z - view_distance
            && pos.x < position.x + view_distance
            && pos.y < position.y + view_distance
            && pos.z < position.z + view_distance)
        });
    }

    pub fn update_available_chunks(&mut self){
        // add new chunks
        let mut available: VecDeque<ChunkUpdaterMessage> = self.get_available_chunks().collect();
        self.available.append(&mut available);
    }

    pub fn get_available_chunks(&self) -> mpsc::TryIter<ChunkUpdaterMessage> {
        self.receiver.try_iter()
    }

    pub fn new_chunk(&self, position: ChunkPosition) {
        let sender = self.sender.clone();
        let noise = self.noise.clone();

        self.threadpool.execute(move ||{
            let mut chunk = Chunk::new(BlockType::Air);

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

            sender.send((position, Arc::new(chunk), ChunkOperation::Creation));
        });
    }
}
