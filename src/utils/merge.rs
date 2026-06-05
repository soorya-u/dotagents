use std::path::Path;

use anyhow::{Context, Result, anyhow};
use jsonc_parser::cst::{CstInputValue, CstRootNode};
use serde_json::Value;

use crate::utils::format::MergeFormat;

pub fn merge_optional<T>(
    base: Option<&T>,
    override_val: Option<&T>,
    merge_fn: impl FnOnce(&T, &T) -> T,
) -> Option<T>
where
    T: Clone,
{
    match (base, override_val) {
        (None, None) => None,
        (Some(b), None) => Some(b.clone()),
        (None, Some(o)) => Some(o.clone()),
        (Some(b), Some(o)) => Some(merge_fn(b, o)),
    }
}

pub(crate) fn merge_into_existing(
    target_path: &Path,
    existing_content: &str,
    rendered_content: &str,
) -> Result<String> {
    let format = MergeFormat::from_extension(target_path)
        .ok_or_else(|| anyhow!("unsupported merge format for {}", target_path.display()))?;

    match format {
        MergeFormat::Json => merge_json(existing_content, rendered_content),
        MergeFormat::Jsonc => merge_jsonc(existing_content, rendered_content),
        MergeFormat::Toml => merge_toml(existing_content, rendered_content),
        MergeFormat::Yaml => merge_yaml(existing_content, rendered_content),
    }
}

fn merge_json(existing_content: &str, rendered_content: &str) -> Result<String> {
    let mut existing: Value =
        serde_json::from_str(existing_content).context("failed to parse existing JSON")?;
    let rendered: Value =
        serde_json::from_str(rendered_content).context("failed to parse rendered JSON")?;

    let mut changed = false;
    if let (Value::Object(existing_map), Value::Object(rendered_map)) = (&mut existing, &rendered) {
        for (key, value) in rendered_map {
            let needs_update = existing_map.get(key).map(|v| v != value).unwrap_or(true);
            if needs_update {
                existing_map.insert(key.clone(), value.clone());
                changed = true;
            }
        }
    }

    if !changed {
        return Ok(existing_content.to_string());
    }

    serde_json::to_string_pretty(&existing).context("failed to serialize merged JSON")
}

fn merge_jsonc(existing_content: &str, rendered_content: &str) -> Result<String> {
    let rendered: Value =
        serde_json::from_str(rendered_content).context("failed to parse rendered JSON")?;

    let root = CstRootNode::parse(existing_content, &Default::default())
        .map_err(|e| anyhow!("failed to parse existing JSONC: {}", e))?;

    let root_obj = root.object_value_or_set();

    if let Value::Object(rendered_map) = &rendered {
        for (key, value) in rendered_map {
            let cst_value = serde_value_to_cst(value);
            if let Some(existing_prop) = root_obj.get(key) {
                existing_prop.set_value(cst_value);
            } else {
                root_obj.append(key, cst_value);
            }
        }
    }

    Ok(root.to_string())
}

fn serde_value_to_cst(value: &Value) -> CstInputValue {
    match value {
        Value::Null => CstInputValue::Null,
        Value::Bool(b) => CstInputValue::Bool(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                CstInputValue::Number(i.to_string())
            } else if let Some(f) = n.as_f64() {
                CstInputValue::Number(f.to_string())
            } else {
                CstInputValue::Null
            }
        }
        Value::String(s) => CstInputValue::String(s.clone()),
        Value::Array(arr) => {
            let items: Vec<CstInputValue> = arr.iter().map(serde_value_to_cst).collect();
            CstInputValue::Array(items)
        }
        Value::Object(map) => {
            let entries: Vec<(String, CstInputValue)> = map
                .iter()
                .map(|(k, v)| (k.clone(), serde_value_to_cst(v)))
                .collect();
            CstInputValue::Object(entries)
        }
    }
}

fn merge_toml(existing_content: &str, rendered_content: &str) -> Result<String> {
    let mut existing_doc: toml_edit::DocumentMut = existing_content
        .parse()
        .context("failed to parse existing TOML")?;

    let rendered_doc: toml_edit::DocumentMut = rendered_content
        .parse()
        .context("failed to parse rendered TOML")?;

    for (key, value) in rendered_doc.iter() {
        existing_doc.insert(key, value.clone());
    }

    Ok(existing_doc.to_string())
}

fn merge_yaml(existing_content: &str, rendered_content: &str) -> Result<String> {
    let mut existing: serde_yaml::Value =
        serde_yaml::from_str(existing_content).context("failed to parse existing YAML")?;
    let rendered: serde_yaml::Value =
        serde_yaml::from_str(rendered_content).context("failed to parse rendered YAML")?;

    let mut changed = false;
    if let (serde_yaml::Value::Mapping(existing_map), serde_yaml::Value::Mapping(rendered_map)) =
        (&mut existing, &rendered)
    {
        for (key, value) in rendered_map {
            let needs_update = existing_map.get(key).map(|v| v != value).unwrap_or(true);
            if needs_update {
                existing_map.insert(key.clone(), value.clone());
                changed = true;
            }
        }
    }

    if !changed {
        return Ok(existing_content.to_string());
    }

    serde_yaml::to_string(&existing).context("failed to serialize merged YAML")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_optional_both_none() {
        let result: Option<i32> = merge_optional(None, None, |a, b| a + b);
        assert_eq!(result, None);
    }

    #[test]
    fn test_merge_optional_only_base() {
        let base = 5;
        let result = merge_optional(Some(&base), None, |a, b| a + b);
        assert_eq!(result, Some(5));
    }

    #[test]
    fn test_merge_optional_only_override() {
        let override_val = 10;
        let result = merge_optional(None, Some(&override_val), |a, b| a + b);
        assert_eq!(result, Some(10));
    }

    #[test]
    fn test_merge_optional_both_present() {
        let base = 5;
        let override_val = 10;
        let result = merge_optional(Some(&base), Some(&override_val), |a, b| a + b);
        assert_eq!(result, Some(15));
    }

    // existing keys preserved, rendered keys win
    #[test]
    fn merge_json_preserves_existing_keys() {
        let existing = r#"{"model": "gemini-2.5", "mcpServers": {"old": {}}}"#;
        let rendered = r#"{"mcpServers": {"new": {"command": "npx"}}}"#;
        let result = merge_json(existing, rendered).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["model"], "gemini-2.5");
        assert!(parsed["mcpServers"]["new"].is_object());
        assert!(
            !parsed["mcpServers"]
                .as_object()
                .unwrap()
                .contains_key("old")
        );
    }

    // rendered wins on scalar conflict
    #[test]
    fn merge_json_rendered_wins_on_conflict() {
        let existing = r#"{"key": "old"}"#;
        let rendered = r#"{"key": "new"}"#;
        let result = merge_json(existing, rendered).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["key"], "new");
    }

    // arrays replaced wholesale
    #[test]
    fn merge_json_arrays_replaced() {
        let existing = r#"{"items": [1, 2, 3]}"#;
        let rendered = r#"{"items": [4, 5]}"#;
        let result = merge_json(existing, rendered).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["items"], json!([4, 5]));
    }

    // top-level keys replaced entirely (shallow merge, no recursive deep merge)
    #[test]
    fn merge_json_top_level_key_replaced() {
        let existing = r#"{"outer": {"keep": true, "replace": "old"}, "other": 1}"#;
        let rendered = r#"{"outer": {"replace": "new"}}"#;
        let result = merge_json(existing, rendered).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["outer"], json!({"replace": "new"}));
        assert_eq!(parsed["other"], 1);
    }

    // empty existing gets rendered keys
    #[test]
    fn merge_json_empty_existing() {
        let existing = r#"{}"#;
        let rendered = r#"{"key": "value"}"#;
        let result = merge_json(existing, rendered).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["key"], "value");
    }

    // TOML sections preserved
    #[test]
    fn merge_toml_preserves_sections() {
        let existing = "[model]\nname = \"mistral\"\n\n[other]\nkey = \"value\"";
        let rendered = "[[mcp_servers]]\nname = \"server\"\ncommand = \"npx\"";
        let result = merge_toml(existing, rendered).unwrap();
        assert!(result.contains("name = \"mistral\""));
        assert!(result.contains("key = \"value\""));
        assert!(result.contains("[[mcp_servers]]"));
    }

    // TOML rendered wins on conflict
    #[test]
    fn merge_toml_rendered_wins_on_conflict() {
        let existing = "[section]\nkey = \"old\"";
        let rendered = "[section]\nkey = \"new\"";
        let result = merge_toml(existing, rendered).unwrap();
        assert!(result.contains("key = \"new\""));
        assert!(!result.contains("key = \"old\""));
    }

    // TOML comments preserved
    #[test]
    fn merge_toml_preserves_comments() {
        let existing = "# This is a comment\n[section]\nkey = \"value\"";
        let rendered = "[new_section]\nnew_key = \"new_value\"";
        let result = merge_toml(existing, rendered).unwrap();
        assert!(result.contains("# This is a comment"));
        assert!(result.contains("key = \"value\""));
        assert!(result.contains("new_key = \"new_value\""));
    }

    // JSONC comments preserved when updating keys
    #[test]
    fn merge_jsonc_preserves_comments() {
        let existing = "{\n  // important comment\n  \"keep\": true,\n  \"update\": \"old\"\n}";
        let rendered = r#"{"update": "new"}"#;
        let result = merge_jsonc(existing, rendered).unwrap();
        assert!(result.contains("// important comment"));
        assert!(result.contains("\"keep\": true"));
        assert!(result.contains("\"update\": \"new\""));
    }

    // JSONC adds new keys while preserving existing
    #[test]
    fn merge_jsonc_adds_new_keys() {
        let existing = r#"{"existing": 1}"#;
        let rendered = r#"{"new_key": 2}"#;
        let result = merge_jsonc(existing, rendered).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["existing"], 1);
        assert_eq!(parsed["new_key"], 2);
    }

    // JSONC existing keys outside rendered scope untouched
    #[test]
    fn merge_jsonc_existing_keys_untouched() {
        let existing = "{\n  // comment\n  \"untouched\": \"value\",\n  \"modified\": \"old\"\n}";
        let rendered = r#"{"modified": "new"}"#;
        let result = merge_jsonc(existing, rendered).unwrap();
        assert!(result.contains("\"untouched\": \"value\""));
        assert!(result.contains("\"modified\": \"new\""));
        assert!(result.contains("// comment"));
    }

    // unsupported format returns error
    #[test]
    fn merge_into_existing_unsupported_format() {
        let result = merge_into_existing(Path::new("file.md"), "content", "rendered");
        assert!(result.is_err());
    }

    // YAML existing keys preserved, rendered keys win
    #[test]
    fn merge_yaml_preserves_existing_keys() {
        let existing = "model: gemini-2.5\nmcpServers:\n  old: {}\n";
        let rendered = "mcpServers:\n  new:\n    command: npx\n";
        let result = merge_yaml(existing, rendered).unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();
        assert_eq!(parsed["model"], "gemini-2.5");
        assert!(parsed["mcpServers"]["new"].is_mapping());
        assert!(
            !parsed["mcpServers"]
                .as_mapping()
                .unwrap()
                .contains_key(&serde_yaml::Value::String("old".to_string()))
        );
    }

    // YAML rendered wins on scalar conflict
    #[test]
    fn merge_yaml_rendered_wins_on_conflict() {
        let existing = "key: old\n";
        let rendered = "key: new\n";
        let result = merge_yaml(existing, rendered).unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();
        assert_eq!(parsed["key"], "new");
    }

    // YAML arrays replaced wholesale
    #[test]
    fn merge_yaml_arrays_replaced() {
        let existing = "items:\n  - 1\n  - 2\n  - 3\n";
        let rendered = "items:\n  - 4\n  - 5\n";
        let result = merge_yaml(existing, rendered).unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();
        let items = parsed["items"].as_sequence().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], 4);
        assert_eq!(items[1], 5);
    }

    // YAML nested objects merged
    #[test]
    fn merge_yaml_nested_objects() {
        let existing = "outer:\n  keep: true\n  replace: old\nother: 1\n";
        let rendered = "outer:\n  replace: new\n";
        let result = merge_yaml(existing, rendered).unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();
        assert_eq!(parsed["outer"]["replace"], "new");
        assert_eq!(parsed["other"], 1);
    }

    // YAML empty existing gets rendered keys
    #[test]
    fn merge_yaml_empty_existing() {
        let existing = "{}\n";
        let rendered = "key: value\n";
        let result = merge_yaml(existing, rendered).unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();
        assert_eq!(parsed["key"], "value");
    }
}
