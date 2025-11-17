use serde_json::{Value, json};

pub fn merge_json(a: Option<&Value>, b: Option<&Value>) -> Value {
    match (a, b) {
        (Some(Value::Object(a_map)), Some(Value::Object(b_map))) => {
            let mut merged = a_map.clone();
            for (k, v) in b_map {
                merged
                    .entry(k.clone())
                    .and_modify(|old| *old = merge_json(Some(old), Some(v)))
                    .or_insert_with(|| v.clone());
            }
            Value::Object(merged)
        }
        (Some(_), Some(b_val)) => b_val.clone(),
        (Some(a_val), None) => a_val.clone(),
        (None, Some(b_val)) => b_val.clone(),
        (None, None) => json!({}),
    }
}
