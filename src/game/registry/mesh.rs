pub use crate::engine::mesh::MeshData;
use dashmap::DashMap;

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

    pub fn add(&mut self, name: &str, data: MeshData){
        self.meshes.push(data);
        let id = self.meshes.len() - 1;
        self.ids.insert(String::from(name), id);
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
