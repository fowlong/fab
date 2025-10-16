//! JSON patch application placeholder.

use anyhow::{Context, Result};

use crate::{
    types::{DocumentIR, PageObject, PatchOperation},
    util::matrix::Matrix2D,
};

/// Apply a list of patch operations to the supplied IR.
/// The stubbed implementation simply logs the operations.
pub fn apply_patches(ir: &mut DocumentIR, ops: &[PatchOperation]) -> Result<()> {
    for op in ops {
        tracing::info!(?op, "received patch op");
        match op {
            PatchOperation::Transform {
                target,
                delta_matrix_pt,
                kind,
            } => {
                apply_transform(ir, target.page, &target.id, kind, *delta_matrix_pt)
                    .with_context(|| format!("applying transform to {}", target.id))?;
            }
            PatchOperation::EditText { .. } | PatchOperation::SetStyle { .. } => {
                // No-op in stub
            }
        }
    }
    Ok(())
}

fn apply_transform(
    ir: &mut DocumentIR,
    page_index: usize,
    target_id: &str,
    kind: &str,
    delta_matrix_pt: [f64; 6],
) -> Result<()> {
    let page = ir
        .pages
        .get_mut(page_index)
        .with_context(|| format!("page {page_index} missing"))?;

    let delta = Matrix2D::from_array(delta_matrix_pt);

    for object in &mut page.objects {
        match (object, kind) {
            (PageObject::Text(text), "text") if text.id == target_id => {
                let current = Matrix2D::from_array(text.tm);
                let updated = delta.multiply(current);
                text.tm = updated.to_array();
                return Ok(());
            }
            (PageObject::Image(image), "image") if image.id == target_id => {
                let current = Matrix2D::from_array(image.cm);
                let updated = delta.multiply(current);
                image.cm = updated.to_array();
                return Ok(());
            }
            (PageObject::Path(_), "path") => {
                // Path transforms not modelled in the stub.
            }
            _ => {}
        }
    }

    Err(anyhow::anyhow!("target {target_id} not found on page {page_index}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PatchTarget, StylePayload};

    #[test]
    fn transform_updates_text_matrix() {
        let mut ir = DocumentIR::sample();
        let ops = vec![PatchOperation::Transform {
            target: PatchTarget {
                page: 0,
                id: "t:42".into(),
            },
            delta_matrix_pt: [1.0, 0.0, 0.0, 1.0, 12.0, -24.0],
            kind: "text".into(),
        }];

        apply_patches(&mut ir, &ops).expect("transform should succeed");

        let page = &ir.pages[0];
        let text = match &page.objects[0] {
            crate::types::PageObject::Text(t) => t,
            _ => panic!("expected text object"),
        };

        assert_eq!(text.tm[4], 112.2);
        assert_eq!(text.tm[5], 676.5);
    }

    #[test]
    fn transform_updates_image_matrix() {
        let mut ir = DocumentIR::sample();
        let ops = vec![PatchOperation::Transform {
            target: PatchTarget {
                page: 0,
                id: "img:9".into(),
            },
            delta_matrix_pt: [1.0, 0.0, 0.0, 1.0, -20.0, 10.0],
            kind: "image".into(),
        }];

        apply_patches(&mut ir, &ops).expect("image transform should succeed");

        let page = &ir.pages[0];
        let image = match &page.objects[1] {
            crate::types::PageObject::Image(img) => img,
            _ => panic!("expected image object"),
        };

        assert_eq!(image.cm[4], 280.0);
        assert_eq!(image.cm[5], 510.0);
    }

    #[test]
    fn set_style_remains_noop() {
        let mut ir = DocumentIR::sample();
        let original = ir.clone();
        let ops = vec![PatchOperation::SetStyle {
            target: PatchTarget {
                page: 0,
                id: "t:42".into(),
            },
            style: StylePayload {
                fill_color: Some([1.0, 0.0, 0.0]),
                ..Default::default()
            },
        }];

        apply_patches(&mut ir, &ops).expect("style patch should succeed");
        assert_eq!(ir, original);
    }
}
