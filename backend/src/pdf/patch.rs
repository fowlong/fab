//! JSON patch application helpers.

use anyhow::{anyhow, bail, Result};

use crate::{
    types::{DocumentIR, PageObject, PatchOperation, PatchTarget},
    util::matrix::Matrix2D,
};

/// Apply a list of patch operations to the supplied IR.
pub fn apply_patches(ir: &mut DocumentIR, ops: &[PatchOperation]) -> Result<()> {
    for op in ops {
        tracing::info!(?op, "received patch op");
        match op {
            PatchOperation::Transform {
                target,
                delta_matrix_pt,
                kind,
            } => apply_transform(ir, target, kind, *delta_matrix_pt)?,
            PatchOperation::EditText { .. } => {
                // Editing text is not yet implemented in the stub backend.
            }
            PatchOperation::SetStyle { .. } => {
                // Style operations are accepted but ignored in the MVP backend.
            }
        }
    }
    Ok(())
}

fn apply_transform(
    ir: &mut DocumentIR,
    target: &PatchTarget,
    kind: &str,
    delta_matrix_pt: [f64; 6],
) -> Result<()> {
    let page = ir
        .pages
        .get_mut(target.page)
        .ok_or_else(|| anyhow!("page {} not found", target.page))?;
    let object = page
        .objects
        .iter_mut()
        .find(|object| object_id(object) == target.id.as_str())
        .ok_or_else(|| anyhow!("object {} not found on page {}", target.id, target.page))?;

    let delta = Matrix2D::from_array(delta_matrix_pt);

    match (kind, object) {
        ("text", PageObject::Text(text)) => {
            let updated = delta.concat(Matrix2D::from_array(text.tm)).to_array();
            text.tm = updated;
        }
        ("image", PageObject::Image(image)) => {
            let updated = delta.concat(Matrix2D::from_array(image.cm)).to_array();
            image.cm = updated;
        }
        ("path", PageObject::Path(path)) => {
            let updated = delta.concat(Matrix2D::from_array(path.cm)).to_array();
            path.cm = updated;
        }
        (other_kind, object_kind) => {
            bail!(
                "transform kind {other_kind} does not match target object {:?}",
                object_kind
            );
        }
    }

    Ok(())
}

fn object_id(object: &PageObject) -> &str {
    match object {
        PageObject::Text(text) => &text.id,
        PageObject::Image(image) => &image.id,
        PageObject::Path(path) => &path.id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::matrix::Matrix2D;

    #[test]
    fn apply_patches_is_noop_for_style_operation() {
        let mut ir = DocumentIR::sample();
        let original = ir.clone();
        let ops = vec![PatchOperation::SetStyle {
            target: crate::types::PatchTarget {
                page: 0,
                id: "t:42".into(),
            },
            style: crate::types::StylePayload {
                fill_color: Some([1.0, 0.0, 0.0]),
                ..Default::default()
            },
        }];

        apply_patches(&mut ir, &ops).expect("patch application should succeed");
        assert_eq!(ir, original);
    }

    #[test]
    fn apply_patches_accepts_empty_operations() {
        let mut ir = DocumentIR::sample();
        apply_patches(&mut ir, &[]).expect("empty patch list should succeed");
    }

    #[test]
    fn transform_updates_text_matrix() {
        let mut ir = DocumentIR::sample();
        let original_tm = if let PageObject::Text(text) = &ir.pages[0].objects[0] {
            text.tm
        } else {
            panic!("expected text object in sample IR");
        };

        let delta = [1.0, 0.0, 0.0, 1.0, 5.5, -3.25];
        let ops = vec![PatchOperation::Transform {
            target: PatchTarget {
                page: 0,
                id: "t:42".into(),
            },
            delta_matrix_pt: delta,
            kind: "text".into(),
        }];

        apply_patches(&mut ir, &ops).expect("transform should succeed");

        if let PageObject::Text(text) = &ir.pages[0].objects[0] {
            let expected = Matrix2D::from_array(delta)
                .concat(Matrix2D::from_array(original_tm))
                .to_array();
            assert_eq!(text.tm, expected);
        } else {
            panic!("expected text object in sample IR");
        }
    }

    #[test]
    fn transform_updates_image_matrix() {
        let mut ir = DocumentIR::sample();
        let original_cm = if let PageObject::Image(image) = &ir.pages[0].objects[1] {
            image.cm
        } else {
            panic!("expected image object in sample IR");
        };

        let delta = [0.5, 0.0, 0.0, 0.5, -10.0, 12.0];
        let ops = vec![PatchOperation::Transform {
            target: PatchTarget {
                page: 0,
                id: "img:9".into(),
            },
            delta_matrix_pt: delta,
            kind: "image".into(),
        }];

        apply_patches(&mut ir, &ops).expect("transform should succeed");

        if let PageObject::Image(image) = &ir.pages[0].objects[1] {
            let expected = Matrix2D::from_array(delta)
                .concat(Matrix2D::from_array(original_cm))
                .to_array();
            assert_eq!(image.cm, expected);
        } else {
            panic!("expected image object in sample IR");
        }
    }
}
