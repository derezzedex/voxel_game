use dashmap::IterMut;
use dashmap::Iter;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::collections::VecDeque;

use std::time::Instant;

use dashmap::{DashMap, DashMapRef, DashMapRefMut};
use std::ops::Deref;

use noise::{NoiseFn, Perlin, Seedable};
use scoped_threadpool::Pool;
use threadpool::ThreadPool;

use crate::game::terrain::block::BlockType;
use crate::game::terrain::chunk::{Chunk, ChunkPosition, CHUNK_SIZE};

pub type ChunkRef<'a> = DashMapRef<'a, ChunkPosition, Arc<Chunk>>;
pub type ChunkRefMut<'a> = DashMapRefMut<'a, ChunkPosition, Arc<Chunk>>;


pub enum ChunkOperation{
    Addition,
    Remotion,
    Update
}

pub type ChunkUpdaterMessage = (ChunkPosition, Option<Arc<Chunk>>, ChunkOperation);

pub struct ChunkManager {
    threadpool: ThreadPool,
    sender: Sender<ChunkUpdaterMessage>,
    receiver: Receiver<ChunkUpdaterMessage>,
    chunks: Arc<DashMap<ChunkPosition, Arc<Chunk>>>,

    chunk_queue: VecDeque<ChunkUpdaterMessage>,

    noise: Perlin,
}

impl ChunkManager {
    pub fn new(thread_number: u32) -> Self {
        let noise = Perlin::new().set_seed(1102130);

        let (sender, receiver) = mpsc::channel();
        let threadpool = ThreadPool::new(thread_number as usize);

        let chunk_queue = VecDeque::new();

        let chunks = Arc::new(DashMap::default());

        Self {
            threadpool,
            sender,
            receiver,
            chunks,
            chunk_queue,
            noise
        }
    }

    pub fn get_chunks<'a>(&'a self) -> Iter<'a, ChunkPosition, Arc<Chunk>>{
        self.chunks.iter()
    }

    pub fn get_mut_chunks<'a>(&'a self) -> IterMut<'a, ChunkPosition, Arc<Chunk>>{
        self.chunks.iter_mut()
    }

    pub fn chunk_exists(&self, position: ChunkPosition) -> bool{
        self.chunks.contains_key(&position)
    }

    pub fn get_chunk(&self, position: ChunkPosition) -> Option<ChunkRef>{
        self.chunks.get(&position)
    }

    pub fn get_mut_chunk(&self, position: ChunkPosition) -> Option<ChunkRefMut>{
        self.chunks.get_mut(&position)
    }

    pub fn add_chunk(&self, position: ChunkPosition, chunk: Arc<Chunk>){
        self.dirty_neighbors(position);
        self.chunks.insert(position, chunk);
    }

    pub fn remove_chunk(&self, position: ChunkPosition){
        self.chunks.remove(&position);
    }

    pub fn get_available_chunks(&self) -> mpsc::TryIter<ChunkUpdaterMessage>{
        self.receiver.try_iter()
    }

    pub fn update_chunk_queue(&mut self){
        let mut new_operations: VecDeque<_> = self.get_available_chunks().collect();
        // println!("Received: {:?}", new_operations.len());
        self.chunk_queue.append(&mut new_operations);
    }

    pub fn chunk_queue_number(&self) -> usize{
        self.chunk_queue.len()
    }

    pub fn dequeue_chunk(&mut self){
        if let Some((pos, chunk, op)) = self.chunk_queue.pop_front(){
            match op{
                ChunkOperation::Addition => self.add_chunk(pos, chunk.expect("Couldn't get unwrap chunk from Manager.")),
                ChunkOperation::Remotion => self.remove_chunk(pos),
                ChunkOperation::Update => (),
            }
        }
    }

    pub fn async_remove_chunk(&mut self, position: ChunkPosition){
        let sender = self.sender.clone();
        let chunk = self.get_chunk(position);

        if let Some(chunk) = chunk{
            self.threadpool.execute(move ||{
                sender.send((position, None, ChunkOperation::Remotion));
            });
        }
    }

    pub fn async_create_chunk(&mut self, position: ChunkPosition){
        let sender = self.sender.clone();
        let noise = self.noise.clone();

        self.threadpool.execute(move ||{
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

            sender.send((position, Some(Arc::new(chunk)), ChunkOperation::Addition));
        });
    }

    pub fn get_neighbors(&self, position: ChunkPosition) -> [Option<ChunkRef>; 6]{
        let north = self.get_chunk(ChunkPosition::new(position.x, position.y, position.z + 1));
        let south = self.get_chunk(ChunkPosition::new(position.x, position.y, position.z - 1));

        let east = self.get_chunk(ChunkPosition::new(position.x + 1, position.y, position.z));
        let west = self.get_chunk(ChunkPosition::new(position.x - 1, position.y, position.z));

        let up = self.get_chunk(ChunkPosition::new(position.x, position.y + 1, position.z));
        let down = self.get_chunk(ChunkPosition::new(position.x, position.y - 1, position.z));

        [north, south, east, west, up, down]
    }

    pub fn dirty_neighbors(&self, position: ChunkPosition){
        for neighbor in position.get_neighbors().iter(){
            if let Some(mut chunk) = self.chunks.get_mut(&position){
                self.clean_chunk(&mut chunk);
            }
        }
    }

    pub fn dirty_chunk(&self, chunk: &mut Arc<Chunk>){
        let chunk = Arc::make_mut(chunk);
        chunk.flag_dirty();
    }

    pub fn clean_chunk(&self, chunk: &mut Arc<Chunk>){
        let chunk = Arc::make_mut(chunk);
        chunk.flag_clean();
    }
}
