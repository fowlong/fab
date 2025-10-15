use std::collections::HashMap;

pub struct DocumentStore {
  counter: u64,
  docs: HashMap<String, Vec<u8>>,
}

impl Default for DocumentStore {
  fn default() -> Self {
    Self {
      counter: 0,
      docs: HashMap::new(),
    }
  }
}

impl DocumentStore {
  pub fn insert(&mut self, data: Vec<u8>) -> String {
    let doc_id = format!("doc-{}", self.counter);
    self.counter += 1;
    self.docs.insert(doc_id.clone(), data);
    doc_id
  }

  pub fn get(&self, id: &str) -> Option<Vec<u8>> {
    self.docs.get(id).cloned()
  }

  pub fn get_mut(&mut self, id: &str) -> Option<&mut Vec<u8>> {
    self.docs.get_mut(id)
  }
}
