use engine::Vertex;
pub use cgmath::Point3;
use engine::mesh;
use dashmap::DashMap;

//TODO: Use AABB3 from collision
pub struct Hitbox{
    pub min: Point3<f32>,
    pub max: Point3<f32>
}

impl Hitbox{
    pub fn new(min: Point3<f32>, max: Point3<f32>) -> Self{
        Self{
            min,
            max
        }
    }
}

//TODO: Add Option<Aabb3> to mesh::MeshData instead of wrapper
pub struct MeshData{
    mesh: mesh::MeshData,
    hitbox: Hitbox,
    transparent: bool
}

impl MeshData{
    pub fn new(v: Vec<Vertex>, i: Vec<u32>, hitbox: Hitbox, transparent: bool) -> Self{
        Self{
            mesh: mesh::MeshData::raw(v, i),
            hitbox,
            transparent
        }
    }

    pub fn get_mesh(&self) -> &mesh::MeshData{
        &self.mesh
    }

    pub fn get_hitbox(&self) -> &Hitbox{
        &self.hitbox
    }

    pub fn is_transparent(&self) -> bool{
        self.transparent
    }
}

pub struct MeshRegistry{
    ids: DashMap<String, usize>,
    meshes: Vec<MeshData>,
}

#[allow(dead_code)]
impl MeshRegistry{
    pub fn new() -> Self{
        let ids = DashMap::default();
        let meshes = Vec::new();

        Self{
            ids,
            meshes
        }
    }

    pub fn add(&mut self, name: &str, data: MeshData) -> usize{
        self.meshes.push(data);
        let id = self.meshes.len() - 1;
        self.ids.insert(String::from(name), id);
        id
    }

    pub fn id_of(&self, name: &str) -> Option<usize>{
        if let Some(id_ref) = self.ids.get(name){
            return Some(*id_ref.value());
        }

        return None;
    }

    pub fn by_id(&self, id: usize) -> Option<&MeshData>{
        self.meshes.get(id)
    }

    pub fn by_name(&self, name: &str) -> Option<&MeshData>{
        if let Some(id) = self.ids.get(name){
            return self.by_id(*id);
        }

        return None;
    }
}
