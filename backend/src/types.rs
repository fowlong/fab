use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PdfRef {
    pub obj: u32,
    pub gen: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextGlyph {
    pub gid: u32,
    pub dx: f64,
    pub dy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    pub bt_span: Span,
    #[serde(rename = "Tm")]
    pub tm: [f64; 6],
    pub font: FontInfo,
    pub unicode: String,
    pub glyphs: Vec<TextGlyph>,
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImageObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    #[serde(rename = "xObject")]
    pub x_object: String,
    #[serde(rename = "cm")]
    pub cm: [f64; 6],
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PathObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    pub operations: Vec<String>,
    #[serde(rename = "cm")]
    pub cm: [f64; 6],
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Span {
    pub start: u64,
    pub end: u64,
    pub stream_obj: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FontInfo {
    pub res_name: String,
    pub size: f64,
    #[serde(rename = "type")]
    pub font_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PageIR {
    pub index: usize,
    pub width_pt: f64,
    pub height_pt: f64,
    pub objects: Vec<PageObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum PageObject {
    #[serde(rename = "text")]
    Text(TextObject),
    #[serde(rename = "image")]
    Image(ImageObject),
    #[serde(rename = "path")]
    Path(PathObject),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentIR {
    pub pages: Vec<PageIR>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum PatchOperation {
    #[serde(rename_all = "camelCase")]
    Transform {
        target: PatchTarget,
        #[serde(rename = "deltaMatrixPt")]
        delta_matrix_pt: [f64; 6],
        kind: String,
    },
    #[serde(rename_all = "camelCase")]
    EditText {
        target: PatchTarget,
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        font_pref: Option<FontPreference>,
    },
    #[serde(rename_all = "camelCase")]
    SetStyle {
        target: PatchTarget,
        style: StylePayload,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FontPreference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefer_existing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StylePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<[f64; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_color: Option<[f64; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity_fill: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity_stroke: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_pdf: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remap: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl DocumentIR {
    pub fn sample() -> Self {
        Self {
            pages: vec![PageIR {
                index: 0,
                width_pt: 595.276,
                height_pt: 841.89,
                objects: vec![
                    PageObject::Text(TextObject {
                        id: "t:42".into(),
                        pdf_ref: PdfRef { obj: 187, gen: 0 },
                        bt_span: Span {
                            start: 12_034,
                            end: 12_345,
                            stream_obj: 155,
                        },
                        tm: [1.0, 0.0, 0.0, 1.0, 100.2, 700.5],
                        font: FontInfo {
                            res_name: "F2".into(),
                            size: 10.5,
                            font_type: "Type0".into(),
                        },
                        unicode: "Invoice #01234".into(),
                        glyphs: vec![
                            TextGlyph {
                                gid: 123,
                                dx: 500.0,
                                dy: 0.0,
                            },
                            TextGlyph {
                                gid: 87,
                                dx: 480.0,
                                dy: 0.0,
                            },
                        ],
                        bbox: [98.4, 688.0, 210.0, 705.0],
                    }),
                    PageObject::Image(ImageObject {
                        id: "img:9".into(),
                        pdf_ref: PdfRef { obj: 200, gen: 0 },
                        x_object: "Im7".into(),
                        cm: [120.0, 0.0, 0.0, 90.0, 300.0, 500.0],
                        bbox: [300.0, 500.0, 420.0, 590.0],
                    }),
                ],
            }],
        }
    }
}

#[cfg(test)]
mod schema_tests {
    use super::*;
    use serde_json::Value;

    fn resolve_ref<'a>(root: &'a Value, reference: &str) -> Option<&'a Value> {
        let trimmed = reference.strip_prefix("#/")?;
        let mut current = root;
        for part in trimmed.split('/') {
            current = current.get(part)?;
        }
        Some(current)
    }

    fn collect_errors(schema: &Value, instance: &Value, root: &Value, path: &str) -> Vec<String> {
        let mut errors = Vec::new();
        validate(schema, instance, root, path, &mut errors);
        errors
    }

    fn validate(
        schema: &Value,
        instance: &Value,
        root: &Value,
        path: &str,
        errors: &mut Vec<String>,
    ) {
        if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
            if let Some(target) = resolve_ref(root, reference) {
                validate(target, instance, root, path, errors);
            } else {
                errors.push(format!("{path}: unresolved reference {reference}"));
            }
            return;
        }

        if let Some(candidates) = schema.get("oneOf").and_then(Value::as_array) {
            if candidates
                .iter()
                .any(|candidate| collect_errors(candidate, instance, root, path).is_empty())
            {
                return;
            }
            errors.push(format!("{path}: did not match any schema in oneOf"));
            return;
        }

        if let Some(expected_type) = schema.get("type").and_then(Value::as_str) {
            let type_matches = match expected_type {
                "object" => instance.is_object(),
                "array" => instance.is_array(),
                "string" => instance.is_string(),
                "number" => instance.is_number(),
                "integer" => instance.as_i64().is_some(),
                "boolean" => instance.is_boolean(),
                _ => true,
            };
            if !type_matches {
                errors.push(format!("{path}: expected type {expected_type}"));
                return;
            }
        }

        if let Some(expected) = schema.get("const") {
            if instance != expected {
                errors.push(format!("{path}: expected constant {expected}"));
            }
        }

        if let Some(enum_values) = schema.get("enum").and_then(Value::as_array) {
            if !enum_values.iter().any(|value| value == instance) {
                errors.push(format!("{path}: value not in enum"));
            }
        }

        if let Some(array) = instance.as_array() {
            if let Some(min_items) = schema.get("minItems").and_then(Value::as_u64) {
                if array.len() < min_items as usize {
                    errors.push(format!("{path}: expected at least {min_items} items"));
                }
            }
            if let Some(max_items) = schema.get("maxItems").and_then(Value::as_u64) {
                if array.len() > max_items as usize {
                    errors.push(format!("{path}: expected at most {max_items} items"));
                }
            }
            if let Some(items_schema) = schema.get("items") {
                for (index, item) in array.iter().enumerate() {
                    let item_path = format!("{path}/{index}");
                    validate(items_schema, item, root, &item_path, errors);
                }
            }
        }

        if let Some(number) = instance.as_f64() {
            if let Some(minimum) = schema.get("minimum").and_then(Value::as_f64) {
                if number < minimum {
                    errors.push(format!("{path}: value below minimum {minimum}"));
                }
            }
            if let Some(maximum) = schema.get("maximum").and_then(Value::as_f64) {
                if number > maximum {
                    errors.push(format!("{path}: value above maximum {maximum}"));
                }
            }
        }

        if let Some(object) = instance.as_object() {
            if let Some(required) = schema.get("required").and_then(Value::as_array) {
                for key in required.iter().filter_map(Value::as_str) {
                    if !object.contains_key(key) {
                        errors.push(format!("{path}/{key}: missing required property"));
                    }
                }
            }

            if matches!(
                schema.get("additionalProperties").and_then(Value::as_bool),
                Some(false)
            ) {
                let allowed: std::collections::HashSet<_> = schema
                    .get("properties")
                    .and_then(Value::as_object)
                    .map(|map| map.keys().cloned().collect())
                    .unwrap_or_default();
                for key in object.keys() {
                    if !allowed.contains(key) {
                        errors.push(format!(
                            "{path}/{key}: additional properties are not allowed"
                        ));
                    }
                }
            }

            if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
                for (key, value) in object {
                    if let Some(property_schema) = properties.get(key) {
                        let property_path = format!("{path}/{key}");
                        validate(property_schema, value, root, &property_path, errors);
                    }
                }
            }
        }
    }

    fn validate_schema(contents: &str, instance: &Value) -> Vec<String> {
        let schema: Value = serde_json::from_str(contents).expect("valid schema json");
        collect_errors(&schema, instance, &schema, "#")
    }

    #[test]
    fn document_ir_sample_matches_schema() {
        let instance = serde_json::to_value(DocumentIR::sample()).expect("serialise IR");
        let errors = validate_schema(
            include_str!("../../shared/schema/ir.schema.json"),
            &instance,
        );
        assert!(errors.is_empty(), "schema validation failed: {errors:?}");
    }

    #[test]
    fn patch_operations_match_schema() {
        let ops = vec![PatchOperation::Transform {
            target: PatchTarget {
                page: 0,
                id: "t:42".into(),
            },
            delta_matrix_pt: [1.0, 0.0, 0.0, 1.0, 2.5, -3.0],
            kind: "text".into(),
        }];
        let instance = serde_json::to_value(&ops).expect("serialise patch ops");
        let errors = validate_schema(
            include_str!("../../shared/schema/patch.schema.json"),
            &instance,
        );
        assert!(errors.is_empty(), "schema validation failed: {errors:?}");
    }

    #[test]
    fn patch_schema_rejects_invalid_payload() {
        let invalid = serde_json::json!([{ "op": "transform", "target": { "page": 0 } }]);
        let errors = validate_schema(
            include_str!("../../shared/schema/patch.schema.json"),
            &invalid,
        );
        assert!(!errors.is_empty());
    }
}
