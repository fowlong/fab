use anyhow::Context;

pub fn load_document(bytes: &[u8]) -> anyhow::Result<lopdf::Document> {
    lopdf::Document::load_mem(bytes).context("failed to parse PDF document")
}
