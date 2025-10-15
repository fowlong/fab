use anyhow::Result;

use crate::types::DocumentIr;

/// Placeholder incremental writer that simply returns an empty PDF file.
/// The future implementation will append updated objects and rewrite the
/// cross-reference table.
pub fn noop_incremental(_ir: &DocumentIr) -> Result<Vec<u8>> {
    // Minimal PDF header/footer so downstream consumers have a valid file.
    let pdf = b"%PDF-1.7\n%\xB5\xED\xAE\xFB\n1 0 obj\n<< /Type /Catalog >>\nendobj\ntrailer\n<< /Root 1 0 R >>\n%%EOF\n";
    Ok(pdf.to_vec())
}
