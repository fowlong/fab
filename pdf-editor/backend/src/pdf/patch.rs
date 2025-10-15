use base64::{engine::general_purpose::STANDARD, Engine as _};

use crate::types::{PatchOperation, PatchResponse};

use super::{loader::DocumentRecord, write};

pub fn apply_patch(
    record: &mut DocumentRecord,
    ops: Vec<PatchOperation>,
) -> anyhow::Result<PatchResponse> {
    if ops.is_empty() {
        return Ok(PatchResponse {
            ok: true,
            ..Default::default()
        });
    }

    record.revision += 1;
    // Placeholder: we currently keep the PDF unchanged but acknowledge the patch.
    let updated_bytes = write::incremental_snapshot(&record.current_pdf, record.revision)?;
    record.current_pdf = updated_bytes.clone();

    let data_uri = format!(
        "data:application/pdf;base64,{}",
        STANDARD.encode(&updated_bytes)
    );

    Ok(PatchResponse {
        ok: true,
        updatedPdf: Some(data_uri),
        remap: None,
    })
}
