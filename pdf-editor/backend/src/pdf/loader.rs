use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct DocumentStore {
  docs: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl DocumentStore {
  pub fn insert(&self, doc_id: String, bytes: Vec<u8>) {
    self.docs.lock().insert(doc_id, bytes);
  }

  pub fn get(&self, doc_id: &str) -> Option<Vec<u8>> {
    self.docs.lock().get(doc_id).cloned()
  }
}
