use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use super::chunk::ChunkPosition;
use crate::engine::mesh::MeshData;

pub type MeshMessage = (ChunkPosition, MeshData);
pub struct ChunkMesher {
    sender: Sender<MeshMessage>,
    receiver: Receiver<MeshMessage>,
}

impl ChunkMesher {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();

        Self { sender, receiver }
    }

    pub fn receive(&self) -> Result<(cgmath::Point3<isize>, MeshData), mpsc::TryRecvError>{
        self.receiver.try_recv()
    }

    pub fn sender(&self) -> Sender<MeshMessage>{
        self.sender.clone()
    }
}