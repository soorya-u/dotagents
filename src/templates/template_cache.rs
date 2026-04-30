use std::path::PathBuf;

use crate::prelude::*;

use crate::utils::{
    fs::{hash_content, read_file, write_file},
    path::get_global_template_cache_dir,
};

/// User-level on-disk cache for downloaded provider template files at `<config_dir>/dotagents/cache/templates/<provider>/<filename>`, validated against `registry.json` SHA-256 checksums.
pub(crate) struct TemplateCache {
    base_dir: PathBuf,
}

impl TemplateCache {
    /// Creates a `TemplateCache` rooted at the user-level config template cache directory.
    pub(crate) fn new() -> Result<Self> {
        Ok(Self {
            base_dir: get_global_template_cache_dir()?,
        })
    }

    /// Creates a `TemplateCache` rooted at an arbitrary directory — used in tests.
    #[cfg(test)]
    pub(crate) fn new_at(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    fn file_path(&self, provider: &str, filename: &str) -> PathBuf {
        self.base_dir.join(provider).join(filename)
    }

    /// Returns the absolute path where a cached file would be stored.
    pub(crate) fn path_of(&self, provider: &str, filename: &str) -> PathBuf {
        self.file_path(provider, filename)
    }

    /// Returns `true` when the cached file exists and its content matches `expected_hex`.
    pub(crate) fn checksum_matches(
        &self,
        provider: &str,
        filename: &str,
        expected_hex: &str,
    ) -> bool {
        let path = self.file_path(provider, filename);
        match read_file(&path) {
            Ok(content) => hash_content(&content) == expected_hex,
            Err(_) => false,
        }
    }

    /// Reads a cached file; returns `None` if absent or unreadable.
    pub(crate) fn read(&self, provider: &str, filename: &str) -> Result<Option<String>> {
        let path = self.file_path(provider, filename);
        if !path.exists() {
            return Ok(None);
        }
        match read_file(&path) {
            Ok(content) => Ok(Some(content)),
            Err(e) => {
                debug!("template cache: failed to read {}: {}", path.display(), e);
                Ok(None)
            }
        }
    }

    /// Writes `content` to the cache for the given provider and filename, creating dirs as needed.
    pub(crate) fn write(&self, provider: &str, filename: &str, content: &str) -> Result<()> {
        let path = self.file_path(provider, filename);
        write_file(&path, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_cache() -> (TempDir, TemplateCache) {
        let dir = TempDir::new().unwrap();
        let cache = TemplateCache::new_at(dir.path().to_path_buf());
        (dir, cache)
    }

    // cache miss returns None when file does not exist
    #[test]
    fn read_returns_none_on_cache_miss() {
        let (_dir, cache) = make_cache();
        let result = cache.read("claude", "command.hbs").unwrap();
        assert!(result.is_none());
    }

    // write then read round-trips the content correctly
    #[test]
    fn write_then_read_roundtrips() {
        let (_dir, cache) = make_cache();
        cache
            .write("claude", "command.hbs", "{{command.content}}")
            .unwrap();
        let result = cache.read("claude", "command.hbs").unwrap();
        assert_eq!(result.as_deref(), Some("{{command.content}}"));
    }

    // checksum_matches returns true when content hash equals expected
    #[test]
    fn checksum_matches_returns_true_on_match() {
        let (_dir, cache) = make_cache();
        let content = "{{command.content}}";
        cache.write("claude", "command.hbs", content).unwrap();
        let expected = hash_content(content);
        assert!(cache.checksum_matches("claude", "command.hbs", &expected));
    }

    // checksum_matches returns false when hash differs
    #[test]
    fn checksum_matches_returns_false_on_mismatch() {
        let (_dir, cache) = make_cache();
        cache.write("claude", "command.hbs", "old content").unwrap();
        let wrong_hash = hash_content("different content");
        assert!(!cache.checksum_matches("claude", "command.hbs", &wrong_hash));
    }

    // checksum_matches returns false when file is absent
    #[test]
    fn checksum_matches_returns_false_on_missing_file() {
        let (_dir, cache) = make_cache();
        assert!(!cache.checksum_matches("claude", "missing.hbs", "anyhash"));
    }

    // write creates nested provider subdirectory automatically
    #[test]
    fn write_creates_parent_directories() {
        let (_dir, cache) = make_cache();
        cache.write("new-provider", "mcp.hbs", "content").unwrap();
        let result = cache.read("new-provider", "mcp.hbs").unwrap();
        assert_eq!(result.as_deref(), Some("content"));
    }
}
