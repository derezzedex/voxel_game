use evmap::{ReadHandle, WriteHandle};
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::collections::VecDeque;

use noise::{NoiseFn, Perlin, Seedable};
use threadpool::{ThreadPool, Builder};

use crate::game::terrain::block::BlockType;
use crate::game::terrain::chunk::{Chunk, ChunkPosition, CHUNK_SIZE};

pub enum ChunkOperation{
    Creation,
    Remotion,
    Update
}

pub struct ChunkMapManager{
    read_handle: ReadHandle<ChunkPosition, Arc<Chunk>>,
    write_handle: WriteHandle<ChunkPosition, Arc<Chunk>>
}

impl ChunkMapManager{
    pub fn new() -> Self{
        let (read_handle, write_handle) = evmap::new();

        Self{
            read_handle,
            write_handle
        }
    }

    pub fn add_chunk(&mut self, position: ChunkPosition, chunk: Arc<Chunk>){
        self.write_handle.insert(position, chunk);
    }

    pub fn remove_chunk(&mut self, position: ChunkPosition){
        self.write_handle.empty(position);
    }

    pub fn get_chunk(&mut self, position: ChunkPosition) -> Option<&Arc<Chunk>>{
        if let Some(chunk) = self.read_handle.get_and(&position, |chunks| chunks.get(0)){
            return chunk
        }else{
            return None
        }
    }
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
}
