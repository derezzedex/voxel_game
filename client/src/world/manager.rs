use engine::{
    render::renderer::Device,
    render::mesh::{Mesh, Vertex},
    utils::MessageChannel,
};
use crate::world::chunk::{ChunkPosition, Chunk};
use std::sync::Arc;

use dashmap::{DashMap, DashSet, iter::Iter};
use uvth::{ThreadPoolBuilder, ThreadPool};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum MeshPosition{
    ChunkPosition(ChunkPosition),
}

pub struct WorldManager{
    chunks: Arc<DashMap<ChunkPosition, Chunk>>,
    visible: Arc<DashSet<ChunkPosition>>,
    meshes: Arc<DashMap<MeshPosition, Mesh>>,
    threadpool: ThreadPool,
}

impl WorldManager{
    pub fn new() -> Self{
        let chunks = Arc::new(DashMap::new());
        let meshes = Arc::new(DashMap::new());
        let visible = Arc::new(DashSet::new());
        let threadpool = ThreadPoolBuilder::new()
            .name("WorldManager Threadpool".to_string())
            .build();

        Self{
            chunks,
            meshes,
            visible,
            threadpool,
        }
    }

    pub fn setup(&mut self, device: &Arc<Device>){
        let vertices = vec![
            Vertex { position: [-0.5, -0.5, 0.], tex_coord: [0., 1.], },
            Vertex { position: [ 0.5, -0.5, 0.], tex_coord: [1., 1.], },
            Vertex { position: [-0.5,  0.5, 0.], tex_coord: [0., 0.], },
            Vertex { position: [ 0.5,  0.5, 0.], tex_coord: [1., 0.], },
        ];
        let indices = vec![
            0, 1, 2,
            2, 1, 3,
        ];

        let mesh = Mesh::new(device, vertices, indices);
        let position = MeshPosition::ChunkPosition(ChunkPosition::new(0, 0, 0));
        self.meshes.insert(position, mesh);
    }

    pub fn update(&mut self){
    }

    pub fn meshes(&self) -> &DashMap<MeshPosition, Mesh>{
        &self.meshes
    }
}
