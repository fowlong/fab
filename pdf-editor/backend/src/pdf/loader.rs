use anyhow::Context;

pub struct LoadedDocument {
    pub bytes: Vec<u8>,
}

pub fn load_from_bytes(bytes: &[u8]) -> anyhow::Result<LoadedDocument> {
    if bytes.is_empty() {
        anyhow::bail!("empty pdf payload");
    }
    Ok(LoadedDocument {
        bytes: bytes.to_vec(),
    })
}

pub fn sanitize(_doc: &mut LoadedDocument) -> anyhow::Result<()> {
    // Placeholder for future sanitisation steps (e.g. removing OpenAction scripts).
    Ok(())
}

pub fn prepare_document(bytes: &[u8]) -> anyhow::Result<LoadedDocument> {
    let mut doc = load_from_bytes(bytes).context("failed to load document")?;
    sanitize(&mut doc)?;
    Ok(doc)
}
