use fake::{Fake, faker::name::en::Name};
use openapiv3::{OpenAPI, ReferenceOr, Schema, SchemaKind, StatusCode, Type};
use serde_json::{Value, json};

/// 根据指定的 path 和 method 生成对应的 mock 响应。
pub fn generate_mock(spec: &OpenAPI, target_path: &str, target_method: &str) -> Value {
    let Some(item) = spec.paths.paths.get(target_path).and_then(|p| p.as_item()) else {
        return json!({"error": format!("Path not found: {}", target_path)});
    };

    let op = match target_method {
        "GET"     => item.get.as_ref(),
        "POST"    => item.post.as_ref(),
        "PUT"     => item.put.as_ref(),
        "DELETE"  => item.delete.as_ref(),
        "OPTIONS" => item.options.as_ref(),
        "PATCH"   => item.patch.as_ref(),
        "HEAD"    => item.head.as_ref(),
        "TRACE"   => item.trace.as_ref(),
        _         => None,
    };

    let Some(op) = op else {
        return json!({"error": format!("Method {} not allowed on {}", target_method, target_path)});
    };

    let Some(resp_ref) = op.responses.responses.get(&StatusCode::Code(200)) else {
        return json!({"error": format!("No 200 response for {} {}", target_method, target_path)});
    };

    let resp = match resp_ref {
        ReferenceOr::Item(r) => r,
        ReferenceOr::Reference { reference } => {
            let name = reference.strip_prefix("#/components/responses/").unwrap_or("");
            let Some(c) = spec.components.as_ref() else {
                return json!(null);
            };
            let Some(r) = c.responses.get(name) else {
                return json!(null);
            };
            match r {
                ReferenceOr::Item(r) => r,
                _ => return json!(null),
            }
        }
    };

    if let Some((_media_type, media)) = resp.content.iter().next() {
        if let Some(schema_ref) = &media.schema {
            return generate_value(spec, schema_ref);
        }
    }

    json!(null)
}

/// Resolve a reference to a schema.
fn resolve_ref(spec: &OpenAPI, reference: &str) -> Value {
    let name = reference
        .strip_prefix("#/components/schemas/")
        .unwrap_or("");
    spec.components
        .as_ref()
        .and_then(|c| c.schemas.get(name))
        .map(|s| generate_value(spec, s))
        .unwrap_or(json!(null))
}

/// Generate a value from a schema.
fn generate_value_from_schema(spec: &OpenAPI, schema: &Schema) -> Value {
    match &schema.schema_kind {
        SchemaKind::Type(Type::String(_)) => {
            json!(Name().fake::<String>())
        }
        SchemaKind::Type(Type::Integer(_)) => {
            json!(rand::random::<i32>().abs() % 1000)
        }
        SchemaKind::Type(Type::Number(_)) => {
            json!(rand::random::<f64>() * 100.0)
        }
        SchemaKind::Type(Type::Boolean(_)) => {
            json!(rand::random::<bool>())
        }
        SchemaKind::Type(Type::Object(obj)) => {
            let mut map = serde_json::Map::new();
            for (key, prop_schema) in &obj.properties {
                let val = match prop_schema {
                    // prop_schema 是 &ReferenceOr<Box<Schema>>
                    ReferenceOr::Reference { reference } => resolve_ref(spec, reference),
                    ReferenceOr::Item(boxed) => generate_value_from_schema(spec, boxed),
                };
                map.insert(key.clone(), val);
            }
            Value::Object(map)
        }
        SchemaKind::Type(Type::Array(arr)) => {
            let item = arr.items
                .as_ref()
                .map(|i| match i {
                    // i 是 &ReferenceOr<Box<Schema>>
                    ReferenceOr::Reference { reference } => resolve_ref(spec, reference),
                    ReferenceOr::Item(boxed) => generate_value_from_schema(spec, boxed),
                })
                .unwrap_or(json!(null));
            json!([item])
        }
        _ => {
            json!({"todo": "implement schema parsing"})
        }
    }
}

/// Generate a value from a schema reference.
pub fn generate_value(spec: &OpenAPI, schema_ref: &ReferenceOr<Schema>) -> Value {
    match schema_ref {
        ReferenceOr::Reference { reference } => resolve_ref(spec, reference),
        ReferenceOr::Item(schema) => generate_value_from_schema(spec, schema),
    }
}

