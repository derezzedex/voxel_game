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

pub type ChunkUpdaterMessage = (ChunkPosition, Arc<Chunk>);

pub struct ChunkManager {
    threadpool: ThreadPool,
    sender: Sender<ChunkUpdaterMessage>,
    receiver: Receiver<ChunkUpdaterMessage>,
    chunks: Arc<DashMap<ChunkPosition, Arc<Chunk>>>,

    noise: Perlin,
}

impl ChunkManager {
    pub fn new(thread_number: u32) -> Self {
        let noise = Perlin::new().set_seed(1102130);

        let (sender, receiver) = mpsc::channel();
        let threadpool = ThreadPool::new(thread_number as usize);

        let chunks = Arc::new(DashMap::default());

        Self {
            threadpool,
            sender,
            receiver,
            chunks,
            noise
        }
    }

    pub fn get_chunks<'a>(&'a self) -> Iter<'a, ChunkPosition, Arc<Chunk>>{
        self.chunks.iter()
    }

    pub fn get_mut_chunks<'a>(&'a self) -> IterMut<'a, ChunkPosition, Arc<Chunk>>{
        self.chunks.iter_mut()
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

    pub fn update_available_chunks(&self, display: &glium::Display){
        for (pos, chunk) in self.get_available_chunks(){
            self.add_chunk(pos, chunk);
        }
    }

    pub fn create_chunk(&mut self, position: ChunkPosition){
        let timer = Instant::now();
        let sender = self.sender.clone();
        let noise = self.noise.clone();
        let map = self.chunks.clone();
        println!("Setup: {:?}", timer.elapsed());
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

            // self.add_chunk(position, Arc::new(chunk));
            map.insert(position, Arc::new(chunk));
            // sender.send((position, Arc::new(chunk)));
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
