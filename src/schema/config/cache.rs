use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use log::debug;
use serde::{Deserialize, Serialize};

use crate::constants::file::CACHE_CONFIG_FILE;
use crate::utils::{
    fs::{read_file, write_file},
    path::get_application_dir,
};

/// Sentinel key used in place of item name for singleton features (mcp, instructions).
pub(crate) const CACHE_SINGLETON_KEY: &str = "_";

/// A single cached deploy entry for one (provider, feature, item) tuple.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct CacheEntry {
    pub hash: String,
    pub target: String,
}

/// The outcome of rendering one feature item during deploy.
#[derive(Debug)]
pub(crate) enum CacheUpdate {
    /// File was written; stores the new hash and absolute target path.
    Written { hash: String, target: String },
    /// File was skipped because rendered content is identical to the last deploy.
    Skipped,
    /// File was skipped because the user manually edited the target.
    UserEditedSkipped { path: PathBuf },
}

/// In-memory representation of `cache.toml`; keyed by `(provider, feature, item)`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub(crate) struct CacheConfig {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub providers: HashMap<String, HashMap<String, HashMap<String, CacheEntry>>>,
}

impl CacheConfig {
    /// Looks up a cache entry for the given (provider, feature, item) triple.
    pub fn get(&self, provider: &str, feature: &str, item: &str) -> Option<&CacheEntry> {
        self.providers
            .get(provider)
            .and_then(|features| features.get(feature))
            .and_then(|items| items.get(item))
    }

    /// Inserts or replaces a cache entry for the given (provider, feature, item) triple.
    pub fn set(&mut self, provider: &str, feature: &str, item: &str, entry: CacheEntry) {
        self.providers
            .entry(provider.to_string())
            .or_default()
            .entry(feature.to_string())
            .or_default()
            .insert(item.to_string(), entry);
    }

    /// Reads `cache.toml` from the application directory; returns empty on missing or parse error.
    pub fn load() -> Result<Self> {
        let app_dir = match get_application_dir() {
            Ok(dir) => dir,
            Err(e) => {
                debug!("cache: failed to locate application dir: {}", e);
                return Ok(Self::default());
            }
        };
        let cache_path = app_dir.join(CACHE_CONFIG_FILE);
        match read_file(&cache_path) {
            Ok(content) => match toml::from_str::<CacheConfig>(&content) {
                Ok(config) => Ok(config),
                Err(e) => {
                    debug!("cache: failed to parse cache.toml, treating as miss: {}", e);
                    Ok(Self::default())
                }
            },
            Err(_) => {
                debug!("cache: cache.toml not found or unreadable, treating as miss");
                Ok(Self::default())
            }
        }
    }

    /// Serializes and writes the cache to `cache.toml` in the application directory.
    pub fn save(&self) -> Result<()> {
        let app_dir =
            get_application_dir().context("failed to locate application dir for cache save")?;
        let cache_path = app_dir.join(CACHE_CONFIG_FILE);
        let content = toml::to_string_pretty(self).context("failed to serialize cache to TOML")?;
        write_file(&cache_path, &content).context("failed to write cache.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // set and get a cache entry round-trips correctly
    #[test]
    fn test_set_and_get() {
        let mut cache = CacheConfig::default();
        let entry = CacheEntry {
            hash: "abc123".to_string(),
            target: "/some/path".to_string(),
        };
        cache.set("claude", "commands", "hello", entry.clone());
        assert_eq!(cache.get("claude", "commands", "hello"), Some(&entry));
    }

    // get returns None for an unknown (provider, feature, item) key
    #[test]
    fn test_get_missing_returns_none() {
        let cache = CacheConfig::default();
        assert!(cache.get("claude", "commands", "hello").is_none());
    }

    // set overwrites an existing entry for the same key
    #[test]
    fn test_set_overwrites_existing() {
        let mut cache = CacheConfig::default();
        cache.set(
            "claude",
            "mcp",
            CACHE_SINGLETON_KEY,
            CacheEntry {
                hash: "old".to_string(),
                target: "/old".to_string(),
            },
        );
        cache.set(
            "claude",
            "mcp",
            CACHE_SINGLETON_KEY,
            CacheEntry {
                hash: "new".to_string(),
                target: "/new".to_string(),
            },
        );
        assert_eq!(
            cache
                .get("claude", "mcp", CACHE_SINGLETON_KEY)
                .unwrap()
                .hash,
            "new"
        );
    }

    // load returns Ok(default) when the application directory cannot be found
    #[test]
    fn test_load_missing_file_returns_default() {
        let result = CacheConfig::load();
        assert!(result.is_ok());
    }

    // load falls back to default when cache.toml contains invalid TOML
    #[test]
    fn test_load_corrupt_toml_returns_default() {
        let corrupt = "not valid toml {]";
        let parsed: std::result::Result<CacheConfig, _> = toml::from_str(corrupt);
        // The TOML parser rejects the corrupt content …
        assert!(parsed.is_err());
        // … and CacheConfig::load() wraps that in Ok(default)
    }
}
