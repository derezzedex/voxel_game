use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Registry<T>{
    ids: HashMap<String, usize>,
    items: Vec<T>,
}

impl<T> Registry<T>{
    pub fn insert<S: Into<String>>(&mut self, name: S, item: T){
        self.items.push(item);
        let id = self.items.len() - 1;
        self.ids.insert(name.into(), id);
    }

    pub fn id_of(&self, name: &str) -> Option<usize> {
        if let Some(id) = self.ids.get(name) {
            return Some(*id);
        }

        None
    }

    pub fn by_id(&self, id: usize) -> Option<&T> {
        self.items.get(id)
    }

    pub fn by_name(&self, name: &str) -> Option<&T> {
        if let Some(id) = self.ids.get(name) {
            return self.by_id(*id);
        }

        return None;
    }
}