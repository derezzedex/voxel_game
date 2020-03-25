pub mod block;
pub mod mesh;
pub use block::{BlockRegistry, BlockDataBuilder, Direction};
pub use mesh::{MeshRegistry, MeshData};
use crate::engine::Vertex;

/// ### Registry
/// Important part of the game, maintains all custom aspects in one place.
/// Examples: Types of blocks, meshes, metadata, etc.
pub struct Registry{
    blocks: BlockRegistry,
    meshes: MeshRegistry,
}

impl Registry{
    pub fn new() -> Self{
        let blocks = BlockRegistry::new();
        let meshes = MeshRegistry::new();

        Self{
            blocks,
            meshes
        }
    }


    // TODO: Don't hardcode the setup of the registries
    pub fn setup(&mut self){
        let block = MeshData::new();
        self.meshes.add("block", block);

        let air = BlockDataBuilder::default().all_faces([0, 1]).transparent(true).build();
        self.blocks.add("air", air);

        let missing = BlockDataBuilder::default().all_faces([0, 1]).build();
        self.blocks.add("missing", missing);

        let grass = BlockDataBuilder::default()
            .all_faces([3, 15])
            .face(Direction::Top, [0, 15])
            .face(Direction::Bottom, [2, 15])
            .build();
        self.blocks.add("grass", grass);

        let dirt = BlockDataBuilder::default().all_faces([2, 15]).build();
        self.blocks.add("dirt", dirt);

        let stone = BlockDataBuilder::default().all_faces([1, 15]).build();
        self.blocks.add("stone", stone);

        let bedrock = BlockDataBuilder::default()
            .all_faces([1, 14])
            .breakable(false)
            .build();
        self.blocks.add("bedrock", bedrock);

        let glass = BlockDataBuilder::default()
            .mesh(1)
            .all_faces([0, 14])
            .breakable(false)
            .transparent(true)
            .build();
        self.blocks.add("glass", glass);

        let mut half_block = MeshData::new();
        half_block.add(
        vec![
            // top (0, 0, 1)
            Vertex::new([-0.5, -0.5,  0.5], [0., 0.], [1, 0]),
            Vertex::new([ 0.5, -0.5,  0.5], [1., 0.], [1, 0]),
            Vertex::new([ 0.5,  0.,   0.5], [1., 1.], [1, 0]),
            Vertex::new([-0.5,  0.,   0.5], [0., 1.], [1, 0]),
            // bottom (0, 0,5 -005
            Vertex::new([-0.5,  0.,  -0.5], [1., 0.], [1, 0]),
            Vertex::new([ 0.5,  0.,  -0.5], [0., 0.], [1, 0]),
            Vertex::new([ 0.5, -0.5, -0.5], [0., 1.], [1, 0]),
            Vertex::new([-0.5, -0.5, -0.5], [1., 1.], [1, 0]),
            // right (1, 00 5005
            Vertex::new([ 0.5, -0.5, -0.5], [0., 0.], [1, 0]),
            Vertex::new([ 0.5,  0.,  -0.5], [1., 0.], [1, 0]),
            Vertex::new([ 0.5,  0.,   0.5], [1., 1.], [1, 0]),
            Vertex::new([ 0.5, -0.5,  0.5], [0., 1.], [1, 0]),
            // left (-1, 00 5005
            Vertex::new([-0.5, -0.5,  0.5], [1., 0.], [1, 0]),
            Vertex::new([-0.5,  0.,   0.5], [0., 0.], [1, 0]),
            Vertex::new([-0.5,  0.,  -0.5], [0., 1.], [1, 0]),
            Vertex::new([-0.5, -0.5, -0.5], [1., 1.], [1, 0]),
            // front (0, 10 5005
            Vertex::new([ 0.5,  0.,  -0.5], [1., 0.], [1, 0]),
            Vertex::new([-0.5,  0.,  -0.5], [0., 0.], [1, 0]),
            Vertex::new([-0.5,  0.,   0.5], [0., 1.], [1, 0]),
            Vertex::new([ 0.5,  0.,   0.5], [1., 1.], [1, 0]),
            // back (0, -10 5005
            Vertex::new([ 0.5, -0.5,  0.5], [0., 0.], [1, 0]),
            Vertex::new([-0.5, -0.5,  0.5], [1., 0.], [1, 0]),
            Vertex::new([-0.5, -0.5, -0.5], [1., 1.], [1, 0]),
            Vertex::new([ 0.5, -0.5, -0.5], [0., 1.], [1, 0]),
        ],
        vec![
            //  0,  1,  2,  2,  3,  0, // top
            //  4,  5,  6,  6,  7,  4, // bottom
            //  8,  9, 10, 10, 11,  8, // right
            // 12, 13, 14, 14, 15, 12, // left
            // 16, 17, 18, 18, 19, 16, // front
            // 20, 21, 22, 22, 23, 20, // back
            20, 23, 22, 22, 21, 20, 16, 19, 18, 18, 17, 16, 12, 15, 14, 14, 13, 12, 8, 11, 10, 10, 9, 8, 4, 7, 6, 6, 5, 4, 0, 3, 2, 2, 1, 0
        ]
        );
        self.meshes.add("half_block", half_block);

        let slab = BlockDataBuilder::default()
            .all_faces([0, 14])
            .breakable(false)
            .transparent(true)
            .build();
        self.blocks.add("slab", slab);
    }

    pub fn block_registry(&self) -> &BlockRegistry{
        &self.blocks
    }

    pub fn mesh_registry(&self) -> &MeshRegistry{
        &self.meshes
    }
}
