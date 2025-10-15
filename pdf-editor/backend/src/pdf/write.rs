use anyhow::Result;

pub fn incremental_snapshot(current: &[u8], revision: u64) -> Result<Vec<u8>> {
    let mut output = current.to_vec();
    if !output.ends_with(b"\n") {
        output.push(b'\n');
    }
    let marker = format!("% Revision {}\n", revision);
    output.extend_from_slice(marker.as_bytes());
    Ok(output)
}
