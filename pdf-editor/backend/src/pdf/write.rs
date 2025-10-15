use crate::pdf::loader::StoreEntry;

pub fn incremental_update(entry: &mut StoreEntry) -> anyhow::Result<()> {
    // TODO: implement incremental write-back.
    // For now, leave PDF bytes unchanged.
    let latest = entry.latest().to_vec();
    entry.update_latest(latest);
    Ok(())
}
