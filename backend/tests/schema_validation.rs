use pdf_editor_backend::types::{DocumentIR, PatchOperation, PatchTarget};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

fn load_schema(name: &str) -> Value {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let schema_path = base.join("../shared/schema").join(name);
    let contents = fs::read_to_string(schema_path).expect("schema should exist");
    serde_json::from_str(&contents).expect("schema should parse")
}

fn validate(schema: &Value, data: &Value, root: &Value) -> bool {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        return validate(resolve_ref(root, reference), data, root);
    }

    if let Some(options) = schema.get("oneOf").and_then(Value::as_array) {
        return options.iter().any(|variant| validate(variant, data, root));
    }

    if let Some(expected) = schema.get("const") {
        return expected == data;
    }

    if let Some(enum_values) = schema.get("enum").and_then(Value::as_array) {
        if !enum_values.iter().any(|value| value == data) {
            return false;
        }
    }

    if let Some(ty) = schema.get("type") {
        if !matches_type(ty, data) {
            return false;
        }
    }

    if let Some(minimum) = schema.get("minimum").and_then(Value::as_f64) {
        if data.as_f64().map_or(false, |value| value < minimum) {
            return false;
        }
    }
    if let Some(maximum) = schema.get("maximum").and_then(Value::as_f64) {
        if data.as_f64().map_or(false, |value| value > maximum) {
            return false;
        }
    }

    if let Some(array) = data.as_array() {
        if let Some(min_items) = schema.get("minItems").and_then(Value::as_u64) {
            if array.len() < min_items as usize {
                return false;
            }
        }
        if let Some(max_items) = schema.get("maxItems").and_then(Value::as_u64) {
            if array.len() > max_items as usize {
                return false;
            }
        }
        if let Some(items_schema) = schema.get("items") {
            if !array.iter().all(|item| validate(items_schema, item, root)) {
                return false;
            }
        }
    }

    if let Some(object) = data.as_object() {
        if let Some(required) = schema.get("required").and_then(Value::as_array) {
            for key in required.iter().filter_map(Value::as_str) {
                if !object.contains_key(key) {
                    return false;
                }
            }
        }

        if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
            for (key, value) in object {
                if let Some(property_schema) = properties.get(key) {
                    if !validate(property_schema, value, root) {
                        return false;
                    }
                } else if schema
                    .get("additionalProperties")
                    .map_or(false, |flag| matches!(flag, Value::Bool(false)))
                {
                    return false;
                }
            }
        }
    }

    true
}

fn matches_type(schema_type: &Value, data: &Value) -> bool {
    match schema_type {
        Value::String(value) => match value.as_str() {
            "object" => data.is_object(),
            "array" => data.is_array(),
            "number" => data.is_number(),
            "integer" => data.as_i64().is_some(),
            "string" => data.is_string(),
            "boolean" => data.is_boolean(),
            _ => true,
        },
        Value::Array(types) => types.iter().any(|ty| matches_type(ty, data)),
        _ => true,
    }
}

fn resolve_ref<'a>(root: &'a Value, reference: &str) -> &'a Value {
    let mut current = root;
    if let Some(path) = reference.strip_prefix("#/") {
        for part in path.split('/') {
            current = current
                .get(part)
                .unwrap_or_else(|| panic!("unresolved $ref {reference}"));
        }
        current
    } else {
        panic!("unsupported $ref: {reference}");
    }
}

#[test]
fn document_ir_conforms_to_schema() {
    let schema = load_schema("ir.schema.json");
    let value = serde_json::to_value(DocumentIR::sample()).unwrap();
    assert!(validate(&schema, &value, &schema));
}

#[test]
fn patch_operations_conform_to_schema() {
    let schema = load_schema("patch.schema.json");
    let ops = vec![PatchOperation::Transform {
        target: PatchTarget {
            page: 0,
            id: "t:42".into(),
        },
        delta_matrix_pt: [1.0, 0.0, 0.0, 1.0, 4.0, -3.0],
        kind: "text".into(),
    }];
    let value = serde_json::to_value(&ops).unwrap();
    assert!(validate(&schema, &value, &schema));

    let invalid = json!([{ "op": "transform", "target": { "page": -1, "id": 7 } }]);
    assert!(!validate(&schema, &invalid, &schema));
}
