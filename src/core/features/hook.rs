use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::constants::file::HOOKS_FILE;
use crate::core::features::traits::FeatureTrait;
use crate::utils::path::get_application_dir;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HookType {
    Command,
    Prompt,
    Http,
    McpTool,
    Agent,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub(crate) struct HookFeature {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hooks: Vec<HookEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct HookEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matcher: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(rename = "type")]
    pub hook_type: HookType,
    // command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "commandWindows")]
    pub command_windows: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    // common
    #[serde(skip_serializing_if = "Option::is_none", rename = "onFailure")]
    pub on_failure: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "loopLimit")]
    pub loop_limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "statusMessage")]
    pub status_message: Option<String>,
    #[serde(default, rename = "async")]
    pub async_: Option<bool>,
    // prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    // http
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    // mcp_tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
    // passthrough
    #[serde(default, flatten, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, Value>,
}

fn default_true() -> bool {
    true
}

impl HookFeature {
    pub fn from_json(json: &str) -> Result<Self> {
        let result = serde_json5::from_str::<HookFeature>(json)
            .context("failed to parse hooks from JSONC")?;
        Ok(result)
    }

    pub fn to_json(&self) -> Result<String> {
        let result =
            serde_json::to_string_pretty(self).context("failed to serialize hooks to JSON")?;
        Ok(result)
    }

    pub fn from_application() -> Result<Self> {
        let dir = get_application_dir()?;
        let config_path = dir.join(HOOKS_FILE);
        let config =
            fs::read_to_string(&config_path).context("failed to read hooks config file")?;
        Self::from_json(&config)
    }

    pub(crate) fn mock() -> &'static str {
        crate::constants::mocks::HOOKS_MOCK
    }
}

impl FeatureTrait for HookFeature {
    fn from_string(value: &str) -> Result<Self> {
        Self::from_json(value)
    }

    fn to_string(&self) -> Result<String> {
        self.to_json()
    }

    fn to_value(&self) -> Value {
        let mut grouped: HashMap<String, Vec<Value>> = HashMap::new();
        for entry in &self.hooks {
            if !entry.enabled {
                continue;
            }
            let mut v = serde_json::to_value(entry).unwrap_or(json!({}));
            if let Value::Object(ref mut map) = v {
                map.remove("enabled");
            }
            grouped.entry(entry.event.clone()).or_default().push(v);
        }
        json!(grouped)
    }

    fn resolve_source_path(_name: Option<&str>) -> Result<PathBuf> {
        let dir = get_application_dir()?;
        Ok(dir.join(HOOKS_FILE))
    }

    fn is_symlinkable(&self) -> bool {
        false
    }

    fn is_provider_agnostic() -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_jsonc() {
        let json = r#"{
            "hooks": [
                {
                    "name": "block-rm",
                    "event": "PreToolUse",
                    "type": "command",
                    "command": "./x.sh",
                    "timeout": 5000,
                    "matcher": "Bash"
                }
            ]
        }"#;
        let feature = HookFeature::from_json(json).unwrap();
        assert_eq!(feature.hooks.len(), 1);
        assert_eq!(feature.hooks[0].event, "PreToolUse");
        assert_eq!(feature.hooks[0].hook_type, HookType::Command);
    }

    #[test]
    fn test_roundtrip() {
        let json =
            r#"{"hooks":[{"name":"test","event":"Stop","type":"command","command":"echo hi"}]}"#;
        let feature = HookFeature::from_json(json).unwrap();
        let serialized = feature.to_json().unwrap();
        let deserialized = HookFeature::from_json(&serialized).unwrap();
        assert_eq!(feature.hooks.len(), deserialized.hooks.len());
    }

    #[test]
    fn test_to_value_groups_by_event_and_excludes_disabled() {
        let json = r#"{
            "hooks": [
                {"event": "PreToolUse", "type": "command", "command": "a", "enabled": true},
                {"event": "PreToolUse", "type": "command", "command": "b", "enabled": false},
                {"event": "Stop", "type": "command", "command": "c"}
            ]
        }"#;
        let feature = HookFeature::from_json(json).unwrap();
        let val = feature.to_value();
        let pre = val.get("PreToolUse").unwrap().as_array().unwrap();
        assert_eq!(pre.len(), 1);
        assert!(val.get("Stop").is_some());
        assert!(
            val.get("PreToolUse").unwrap().as_array().unwrap()[0]
                .get("enabled")
                .is_none()
        );
    }

    #[test]
    fn test_unknown_event_accepted() {
        let json = r#"{"hooks":[{"event":"Interrupt","type":"command","command":"x"}]}"#;
        let feature = HookFeature::from_json(json).unwrap();
        assert_eq!(feature.hooks[0].event, "Interrupt");
    }

    #[test]
    fn test_invalid_type_rejected() {
        let json = r#"{"hooks":[{"event":"PreToolUse","type":"webhook","command":"x"}]}"#;
        let result = HookFeature::from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_extra_passthrough() {
        // loopLimit is canonical; extra is for provider-specific passthrough only
        let json = r#"{"hooks":[{"event":"PreToolUse","type":"command","command":"x","foo":"bar","someProviderThing":123}]}"#;
        let feature = HookFeature::from_json(json).unwrap();
        let extra = &feature.hooks[0].extra;
        assert_eq!(extra.get("foo").unwrap().as_str().unwrap(), "bar");
        assert_eq!(
            extra.get("someProviderThing").unwrap().as_u64().unwrap(),
            123
        );
        // canonical loopLimit should not be in extra
        assert!(extra.get("loopLimit").is_none());
    }
}
