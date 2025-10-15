use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct StoredDocument {
    id: Uuid,
    filename: Option<String>,
    original_bytes: Vec<u8>,
    latest_bytes: Vec<u8>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl StoredDocument {
    pub fn new(filename: Option<String>, bytes: Vec<u8>) -> Self {
        let timestamp = OffsetDateTime::now_utc();
        Self {
            id: Uuid::new_v4(),
            filename,
            original_bytes: bytes.clone(),
            latest_bytes: bytes,
            created_at: timestamp,
            updated_at: timestamp,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }

    pub fn original_bytes(&self) -> &[u8] {
        &self.original_bytes
    }

    pub fn latest_bytes(&self) -> &[u8] {
        &self.latest_bytes
    }

    pub fn set_latest_bytes(&mut self, bytes: Vec<u8>) {
        self.latest_bytes = bytes;
        self.updated_at = OffsetDateTime::now_utc();
    }

    pub fn updated_at(&self) -> OffsetDateTime {
        self.updated_at
    }
}
