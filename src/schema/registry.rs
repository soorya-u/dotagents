use std::collections::HashMap;

use anyhow::{Result, anyhow};
use serde::Deserialize;

use crate::utils::http::fetch_template;

/// The full provider registry, deserialised from `registry.json`.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Registry {
    pub providers: HashMap<String, ProviderRegistryEntry>,
}

/// A single provider's entry in the registry.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ProviderRegistryEntry {
    /// URL path to the provider's `provider.toml`, relative to the registry base URL.
    pub path: String,

    /// SHA-256 checksums keyed by filename, used for template-source cache invalidation.
    pub checksums: Option<HashMap<String, String>>,

    /// Human-readable display name (e.g. "Claude Code").
    pub name: Option<String>,

    /// Documentation URL for the provider.
    pub url: Option<String>,
}

impl Registry {
    /// Fetches and deserialises the registry from `url`.
    pub(crate) fn fetch(url: &str) -> Result<Self> {
        let body = fetch_template(url)
            .map_err(|e| anyhow!("Failed to fetch registry from {}: {}", url, e))?;
        serde_json::from_str(&body)
            .map_err(|e| anyhow!("Failed to parse registry JSON from {}: {}", url, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // full entry with checksums deserialises correctly
    #[test]
    fn deserialise_entry_with_checksums() {
        let json = r#"{
            "providers": {
                "claude": {
                    "path": "/templates/claude/provider.toml",
                    "checksums": {
                        "command.hbs": "abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234",
                        "provider.toml": "1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd"
                    }
                }
            }
        }"#;
        let registry: Registry = serde_json::from_str(json).unwrap();
        let entry = registry.providers.get("claude").unwrap();
        assert_eq!(entry.path, "/templates/claude/provider.toml");
        let checksums = entry.checksums.as_ref().unwrap();
        assert!(checksums.contains_key("command.hbs"));
        assert!(checksums.contains_key("provider.toml"));
    }

    // entry without checksums field deserialises with None
    #[test]
    fn deserialise_entry_without_checksums() {
        let json = r#"{
            "providers": {
                "cursor": {
                    "path": "/templates/cursor/provider.toml"
                }
            }
        }"#;
        let registry: Registry = serde_json::from_str(json).unwrap();
        let entry = registry.providers.get("cursor").unwrap();
        assert!(entry.checksums.is_none());
    }

    // entry with name and url deserialises correctly
    #[test]
    fn deserialise_entry_with_name_and_url() {
        let json = r#"{
            "providers": {
                "gemini": {
                    "path": "/v1/templates/gemini/provider.toml",
                    "checksums": {},
                    "name": "Gemini CLI",
                    "url": "https://google-gemini.github.io/cli"
                }
            }
        }"#;
        let registry: Registry = serde_json::from_str(json).unwrap();
        let entry = registry.providers.get("gemini").unwrap();
        assert_eq!(entry.name.as_deref(), Some("Gemini CLI"));
        assert_eq!(
            entry.url.as_deref(),
            Some("https://google-gemini.github.io/cli")
        );
    }

    // entry without name and url fields deserialises with None
    #[test]
    fn deserialise_entry_without_name_and_url() {
        let json = r#"{
            "providers": {
                "claude": {
                    "path": "/v1/templates/claude/provider.toml"
                }
            }
        }"#;
        let registry: Registry = serde_json::from_str(json).unwrap();
        let entry = registry.providers.get("claude").unwrap();
        assert!(entry.name.is_none());
        assert!(entry.url.is_none());
    }

    // registry with mixed entries (some with name/url, some without) parses correctly
    #[test]
    fn deserialise_mixed_entries() {
        let json = r#"{
            "providers": {
                "claude": {
                    "path": "/v1/templates/claude/provider.toml"
                },
                "gemini": {
                    "path": "/v1/templates/gemini/provider.toml",
                    "checksums": {},
                    "name": "Gemini CLI",
                    "url": "https://google-gemini.github.io/cli"
                }
            }
        }"#;
        let registry: Registry = serde_json::from_str(json).unwrap();
        let claude = registry.providers.get("claude").unwrap();
        assert!(claude.name.is_none());
        assert!(claude.url.is_none());
        let gemini = registry.providers.get("gemini").unwrap();
        assert_eq!(gemini.name.as_deref(), Some("Gemini CLI"));
        assert_eq!(
            gemini.url.as_deref(),
            Some("https://google-gemini.github.io/cli")
        );
    }

    // unknown extra fields in entry are ignored (forward compatibility)
    #[test]
    fn deserialise_ignores_unknown_fields() {
        let json = r#"{
            "providers": {
                "codex": {
                    "path": "/templates/codex/provider.toml",
                    "future_field": "ignored",
                    "checksums": {}
                }
            },
            "$schema": "https://example.com/schema.json"
        }"#;
        let result = serde_json::from_str::<Registry>(json);
        assert!(
            result.is_ok(),
            "should ignore unknown fields, got: {:?}",
            result
        );
    }
}
