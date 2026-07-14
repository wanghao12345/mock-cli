use std::collections::HashMap;

use fake::{Fake, faker::internet::en::SafeEmail, faker::name::en::Name};
use openapiv3::{
    OpenAPI, ReferenceOr, Schema, SchemaKind, StatusCode, StringFormat, Type,
    VariantOrUnknownOrEmpty,
};
use serde_json::{Value, json};

/// Maximum recursion depth. Prevents stack overflow when `$ref` forms a cycle
/// (e.g. A -> B -> A). When reached, generation returns null so the process
/// stays alive instead of crashing.
const MAX_DEPTH: usize = 10;

/// Generate a mock response.
///
/// Returns `(HTTP status code, body)`:
/// - path not found        -> 404
/// - method not allowed     -> 405
/// - no 200 response defined -> 501
/// - success                -> 200 + schema-driven fake data
///
/// `path_params` is the map of path parameters extracted from the request
/// (parameter name -> raw string value). When a field in the response schema
/// shares its name with a path parameter, the parameter value is injected
/// into that field (coerced to the schema type) so the mock stays consistent
/// with the request context — e.g. hitting `/pets/{petId}` echoes the `petId`
/// value back inside the `petId` field.
pub fn generate_mock(
    spec: &OpenAPI,
    target_path: &str,
    target_method: &str,
    path_params: &HashMap<String, String>,
) -> (u16, Value) {
    let Some(item) = spec.paths.paths.get(target_path).and_then(|p| p.as_item()) else {
        return (
            404,
            json!({"error": format!("Path not found: {}", target_path)}),
        );
    };

    let op = match target_method {
        "GET" => item.get.as_ref(),
        "POST" => item.post.as_ref(),
        "PUT" => item.put.as_ref(),
        "DELETE" => item.delete.as_ref(),
        "OPTIONS" => item.options.as_ref(),
        "PATCH" => item.patch.as_ref(),
        "HEAD" => item.head.as_ref(),
        "TRACE" => item.trace.as_ref(),
        _ => None,
    };

    let Some(op) = op else {
        return (
            405,
            json!({"error": format!("Method {} not allowed on {}", target_method, target_path)}),
        );
    };

    let Some(resp_ref) = op.responses.responses.get(&StatusCode::Code(200)) else {
        return (
            501,
            json!({"error": format!("No 200 response for {} {}", target_method, target_path)}),
        );
    };

    let resp = match resp_ref {
        ReferenceOr::Item(r) => r,
        ReferenceOr::Reference { reference } => {
            let name = reference
                .strip_prefix("#/components/responses/")
                .unwrap_or("");
            let Some(c) = spec.components.as_ref() else {
                return (200, json!(null));
            };
            let Some(r) = c.responses.get(name) else {
                return (200, json!(null));
            };
            match r {
                ReferenceOr::Item(r) => r,
                _ => return (200, json!(null)),
            }
        }
    };

    if let Some((_media_type, media)) = resp.content.iter().next() {
        if let Some(schema_ref) = &media.schema {
            return (200, generate_value(spec, schema_ref, path_params, 0));
        }
    }

    (200, json!(null))
}

/// Resolve a `$ref` pointer and continue generating a value.
/// `depth` is incremented on each hop to bound recursion (cycle safety).
fn resolve_ref(
    spec: &OpenAPI,
    reference: &str,
    path_params: &HashMap<String, String>,
    depth: usize,
) -> Value {
    let name = reference.strip_prefix("#/components/schemas/").unwrap_or("");
    spec.components
        .as_ref()
        .and_then(|c| c.schemas.get(name))
        .map(|s| generate_value(spec, s, path_params, depth))
        .unwrap_or(json!(null))
}

/// Coerce a raw path-parameter string into the JSON value matching the schema type.
/// - integer -> i64
/// - number  -> f64
/// - boolean -> bool
/// - other   -> the raw string
fn coerce_path_param(val: &str, schema: &Schema) -> Value {
    match &schema.schema_kind {
        SchemaKind::Type(Type::Integer(_)) => {
            val.parse::<i64>().map(|n| json!(n)).unwrap_or_else(|_| json!(val))
        }
        SchemaKind::Type(Type::Number(_)) => {
            val.parse::<f64>().map(|n| json!(n)).unwrap_or_else(|_| json!(val))
        }
        SchemaKind::Type(Type::Boolean(_)) => {
            json!(matches!(val.to_lowercase().as_str(), "true" | "1"))
        }
        _ => json!(val),
    }
}

/// Generate a single value from a schema.
fn generate_value_from_schema(
    spec: &OpenAPI,
    schema: &Schema,
    path_params: &HashMap<String, String>,
    depth: usize,
) -> Value {
    // Depth guard: stop recursion on cycles (e.g. A -> B -> A) before we overflow.
    if depth >= MAX_DEPTH {
        return Value::Null;
    }

    match &schema.schema_kind {
        SchemaKind::Type(Type::String(string_type)) => {
            // Enums win first: pick a random allowed value.
            if !string_type.enumeration.is_empty() {
                // rand 0.10 dropped usize from StandardUniform; sample via u64.
                let idx = rand::random::<u64>() as usize % string_type.enumeration.len();
                return match &string_type.enumeration[idx] {
                    Some(s) => json!(s),
                    None => Value::Null,
                };
            }
            // Generate a value that reflects the declared format.
            match &string_type.format {
                // Built-in formats known to openapiv3.
                VariantOrUnknownOrEmpty::Item(fmt) => match fmt {
                    StringFormat::Date => json!(generate_date()),
                    StringFormat::DateTime => json!(generate_datetime()),
                    StringFormat::Password => json!(random_alnum(12)),
                    StringFormat::Byte => json!(random_alnum(8)),
                    StringFormat::Binary => json!(random_alnum(16)),
                },
                // Formats not modeled by StringFormat (email/uuid/uri/...).
                VariantOrUnknownOrEmpty::Unknown(fmt) => match fmt.as_str() {
                    "email" => json!(SafeEmail().fake::<String>()),
                    "uuid" => json!(generate_uuid_v4()),
                    "uri" | "url" => {
                        json!(format!("https://example.com/{}", rand::random::<u32>()))
                    }
                    _ => json!(Name().fake::<String>()),
                },
                VariantOrUnknownOrEmpty::Empty => json!(Name().fake::<String>()),
            }
        }
        // Integer enums: pick a random allowed value before the default random path.
        SchemaKind::Type(Type::Integer(int_type)) => {
            if !int_type.enumeration.is_empty() {
                let idx = rand::random::<u64>() as usize % int_type.enumeration.len();
                return match int_type.enumeration[idx] {
                    Some(n) => json!(n),
                    None => Value::Null,
                };
            }
            // Use u32 instead of i32 to avoid the i32::MIN.abs() overflow panic.
            json!(rand::random::<u32>() % 1000)
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
                // Optional fields stay present in the payload (so the shape is
                // stable for frontend integration) but become null with a small
                // probability, mirroring how real APIs sometimes omit data.
                let is_required = obj.required.iter().any(|r| r == key);
                if !is_required && rand::random::<u8>() % 5 == 0 {
                    map.insert(key.clone(), Value::Null);
                    continue;
                }
                // If the field name matches a path parameter, echo the value
                // back (coerced to the field's schema type).
                if let Some(val) = path_params.get(key) {
                    if let ReferenceOr::Item(s) = prop_schema {
                        map.insert(key.clone(), coerce_path_param(val, s));
                        continue;
                    }
                }
                let val = match prop_schema {
                    ReferenceOr::Reference { reference } => {
                        resolve_ref(spec, reference, path_params, depth + 1)
                    }
                    ReferenceOr::Item(boxed) => {
                        generate_value_from_schema(spec, boxed, path_params, depth + 1)
                    }
                };
                map.insert(key.clone(), val);
            }
            Value::Object(map)
        }
        SchemaKind::Type(Type::Array(arr)) => {
            // Honor min_items / max_items, defaulting to 1..=5.
            let min = arr.min_items.unwrap_or(1).max(1);
            let max = arr.max_items.unwrap_or(5).max(min);
            let count = if min == max {
                min
            } else {
                rand::random_range(min..=max)
            };
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                let item = arr
                    .items
                    .as_ref()
                    .map(|i| match i {
                        ReferenceOr::Reference { reference } => {
                            resolve_ref(spec, reference, path_params, depth + 1)
                        }
                        ReferenceOr::Item(boxed) => {
                            generate_value_from_schema(spec, boxed, path_params, depth + 1)
                        }
                    })
                    .unwrap_or(json!(null));
                items.push(item);
            }
            Value::Array(items)
        }
        _ => json!({"todo": "implement schema parsing"}),
    }
}

/// Generate a value from a schema reference.
pub fn generate_value(
    spec: &OpenAPI,
    schema_ref: &ReferenceOr<Schema>,
    path_params: &HashMap<String, String>,
    depth: usize,
) -> Value {
    match schema_ref {
        ReferenceOr::Reference { reference } => {
            resolve_ref(spec, reference, path_params, depth)
        }
        ReferenceOr::Item(schema) => {
            generate_value_from_schema(spec, schema, path_params, depth)
        }
    }
}

// ===== Helpers: build strings manually with `rand` to avoid pulling extra
//      feature flags into the `fake` crate. =====

/// Build a v4 UUID string (`xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx`).
fn generate_uuid_v4() -> String {
    let a: u32 = rand::random::<u32>();
    let b: u16 = rand::random::<u16>();
    let c: u16 = rand::random::<u16>() & 0x0fff | 0x4000; // version 4
    let d: u16 = rand::random::<u16>() & 0x3fff | 0x8000; // variant
    let e: u64 = rand::random::<u64>();
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        a,
        b,
        c,
        d,
        e & 0xffffffffffff
    )
}

/// Build a `YYYY-MM-DD` date string.
fn generate_date() -> String {
    format!(
        "2024-{:02}-{:02}",
        rand::random_range(1u32..=12),
        rand::random_range(1u32..=28)
    )
}

/// Build an ISO8601 datetime string (e.g. `2024-03-15T14:27:08Z`).
fn generate_datetime() -> String {
    format!(
        "2024-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        rand::random_range(1u32..=12),
        rand::random_range(1u32..=28),
        rand::random_range(0u32..23),
        rand::random_range(0u32..59),
        rand::random_range(0u32..59)
    )
}

/// Build a random alphanumeric string of the given length.
fn random_alnum(len: usize) -> String {
    (0..len)
        .map(|_| {
            let n: u8 = rand::random_range(0..62);
            match n {
                0..=9 => (b'0' + n) as char,
                10..=35 => (b'a' + n - 10) as char,
                _ => (b'A' + n - 36) as char,
            }
        })
        .collect()
}
