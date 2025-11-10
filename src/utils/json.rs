use serde_json::{Value, json};

pub fn merge_json(a: &Value, b: &Value) -> Value {
    match (a, b) {
        (Value::Object(a_map), Value::Object(b_map)) => {
            let mut merged = a_map.clone();
            for (k, v) in b_map {
                merged
                    .entry(k.clone())
                    .and_modify(|old| *old = merge_json(old, v))
                    .or_insert_with(|| v.clone());
            }
            Value::Object(merged)
        }
        (_, b_val) => b_val.clone(),
    }
}
