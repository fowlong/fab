use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct Document {
  pub bytes: Vec<u8>,
}

#[derive(Default)]
pub struct DocumentStore {
  docs: HashMap<String, Document>,
}

impl DocumentStore {
  pub fn insert(&mut self, bytes: Vec<u8>) -> String {
    let id = Uuid::new_v4().to_string();
    self.docs.insert(id.clone(), Document { bytes });
    id
  }

  pub fn get(&self, id: &str) -> Option<&Document> {
    self.docs.get(id)
  }

  pub fn get_mut(&mut self, id: &str) -> Option<&mut Document> {
    self.docs.get_mut(id)
  }
}
