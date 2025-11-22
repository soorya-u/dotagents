use serde_json::{Value, json};
use std::mem;

/// Optimized merge that takes ownership to reduce clones
fn merge_json_mut(mut a: Value, b: &Value) -> Value {
    match (&mut a, b) {
        (Value::Object(a_map), Value::Object(b_map)) => {
            // Note: serde_json::Map doesn't have reserve(), but using entry API is still efficient
            for (k, v) in b_map {
                a_map
                    .entry(k.clone())
                    .and_modify(|old| {
                        // Take ownership to avoid clone when recursing
                        let old_val = mem::replace(old, Value::Null);
                        *old = merge_json_mut(old_val, v);
                    })
                    .or_insert_with(|| v.clone());
            }
            a
        }
        (_, b_val) => b_val.clone(),
    }
}

pub fn merge_many_json(values: &[Value]) -> Value {
    if values.is_empty() {
        return json!({});
    }

    // Clone first value once, then mutate in-place
    let mut result = values[0].clone();

    for value in &values[1..] {
        result = merge_json_mut(result, value);
    }

    result
}

pub fn merge_json(a: Option<&Value>, b: Option<&Value>) -> Value {
    match (a, b) {
        (Some(a_val), Some(b_val)) => merge_json_mut(a_val.clone(), b_val),
        (Some(a_val), None) => a_val.clone(),
        (None, Some(b_val)) => b_val.clone(),
        (None, None) => json!({}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_json_both_objects() {
        let a = json!({"name": "Alice", "age": 30});
        let b = json!({"age": 31, "city": "NYC"});
        let result = merge_json(Some(&a), Some(&b));
        assert_eq!(result, json!({"name": "Alice", "age": 31, "city": "NYC"}));
    }

    #[test]
    fn test_merge_json_nested_objects() {
        let a = json!({"user": {"name": "Alice", "age": 30}});
        let b = json!({"user": {"age": 31, "city": "NYC"}});
        let result = merge_json(Some(&a), Some(&b));
        assert_eq!(
            result,
            json!({"user": {"name": "Alice", "age": 31, "city": "NYC"}})
        );
    }

    #[test]
    fn test_merge_json_b_overwrites_non_object() {
        let a = json!({"key": "value1"});
        let b = json!({"key": "value2"});
        let result = merge_json(Some(&a), Some(&b));
        assert_eq!(result, json!({"key": "value2"}));
    }

    #[test]
    fn test_merge_json_only_a() {
        let a = json!({"name": "Alice"});
        let result = merge_json(Some(&a), None);
        assert_eq!(result, json!({"name": "Alice"}));
    }

    #[test]
    fn test_merge_json_only_b() {
        let b = json!({"name": "Bob"});
        let result = merge_json(None, Some(&b));
        assert_eq!(result, json!({"name": "Bob"}));
    }

    #[test]
    fn test_merge_json_both_none() {
        let result = merge_json(None, None);
        assert_eq!(result, json!({}));
    }

    #[test]
    fn test_merge_json_non_object_values() {
        let a = json!("string1");
        let b = json!("string2");
        let result = merge_json(Some(&a), Some(&b));
        assert_eq!(result, json!("string2"));
    }

    #[test]
    fn test_merge_many_json_empty() {
        let result = merge_many_json(&[]);
        assert_eq!(result, json!({}));
    }

    #[test]
    fn test_merge_many_json_single() {
        let values = vec![json!({"name": "Alice"})];
        let result = merge_many_json(&values);
        assert_eq!(result, json!({"name": "Alice"}));
    }

    #[test]
    fn test_merge_many_json_multiple() {
        let values = vec![
            json!({"name": "Alice", "age": 30}),
            json!({"age": 31, "city": "NYC"}),
            json!({"country": "USA"}),
        ];
        let result = merge_many_json(&values);
        assert_eq!(
            result,
            json!({"name": "Alice", "age": 31, "city": "NYC", "country": "USA"})
        );
    }

    #[test]
    fn test_merge_many_json_nested() {
        let values = vec![
            json!({"user": {"name": "Alice"}}),
            json!({"user": {"age": 30}}),
            json!({"user": {"city": "NYC"}}),
        ];
        let result = merge_many_json(&values);
        assert_eq!(
            result,
            json!({"user": {"name": "Alice", "age": 30, "city": "NYC"}})
        );
    }
}
