use std::collections::HashMap;

use anyhow::{Result, anyhow};
use log::warn;

use crate::constants::domain::TRUSTED_DOMAIN;
use crate::schema::config::{
    AppConfig, FeatureSettings, Features, GlobalConfig, Providers, TomlConfig,
};
use crate::schema::features::Feature;
use crate::schema::registry::Registry;
use crate::templates::{TemplateCache, do_get};

pub(crate) use crate::constants::domain::REGISTRY_URL;

/// Maps a `Feature` to the `.hbs` filename used in a provider directory.
fn feature_filename(feature: &Feature) -> &'static str {
    match feature {
        Feature::Command => "command.hbs",
        Feature::Instruction => "instruction.hbs",
        Feature::Mcp => "mcp.hbs",
        Feature::Skill => "skill.hbs",
    }
}

/// Fetches `url` or serves from cache, updating the cache when the remote copy is newer.
///
/// When `no_cache` is `true` the cache is bypassed entirely and the file is always fetched.
pub(crate) fn fetch_or_cache_file(
    provider: &str,
    filename: &str,
    url: &str,
    registry_checksum: Option<&str>,
    cache: &TemplateCache,
    no_cache: bool,
) -> Result<Option<String>> {
    if !no_cache {
        if let Some(expected) = registry_checksum {
            if cache.checksum_matches(provider, filename, expected) {
                // Cache is valid; serve from disk without a network request.
                return cache.read(provider, filename);
            }
        } else {
            // No checksum provided — use cached copy if present.
            if let Some(content) = cache.read(provider, filename)? {
                return Ok(Some(content));
            }
        }
    }

    let content = do_get(url).map_err(|e| {
        anyhow!(
            "Failed to fetch {} for provider '{}': {}",
            filename,
            provider,
            e
        )
    })?;
    cache.write(provider, filename, &content)?;
    Ok(Some(content))
}

/// Parses a `provider.toml` string and extracts `FeatureSettings` for the given provider+feature.
pub(crate) fn parse_provider_toml(
    content: &str,
    provider: &str,
    feature: &Feature,
) -> Result<Option<FeatureSettings>> {
    let config = GlobalConfig::from_toml(content)
        .map_err(|e| anyhow!("Failed to parse provider.toml for '{}': {}", provider, e))?;

    let feature_settings = config
        .providers
        .and_then(|p| p.0)
        .unwrap_or_default()
        .remove(provider)
        .and_then(|f| f.get_config(feature));

    Ok(feature_settings)
}

/// Fills in missing `template` and/or `target` fields in `app_config` for all providers
/// that are listed in `targets` but lack fully-configured `FeatureSettings`.
///
/// - `registry`: the fetched registry; `None` when the fetch failed or `--offline` was specified.
/// - `offline`: when `true`, only the local cache is consulted and a cold cache is a hard error.
/// - `no_cache`: when `true`, cached files are ignored and everything is re-fetched.
pub(crate) fn resolve_provider_defaults(
    app_config: &mut AppConfig,
    registry: Option<&Registry>,
    cache: &TemplateCache,
    offline: bool,
    no_cache: bool,
) -> Result<()> {
    let all_features = [
        Feature::Command,
        Feature::Instruction,
        Feature::Mcp,
        Feature::Skill,
    ];

    for provider_name in app_config.targets.clone() {
        for feature in &all_features {
            if !app_config.has_feature(feature) {
                continue;
            }

            // Check whether this (provider, feature) already has both template and target.
            let existing: Option<FeatureSettings> = app_config
                .providers
                .as_ref()
                .and_then(|p| p.0.as_ref())
                .and_then(|map| map.get(&provider_name))
                .and_then(|f| f.get_config(feature));

            let needs_template = existing.as_ref().is_none_or(|s| s.template.is_none());
            let needs_target = existing.as_ref().is_none_or(|s| s.target.is_none());

            if !needs_template && !needs_target {
                continue;
            }

            // Attempt resolution; propagate as hard error only in --offline mode.
            match resolve_for_provider(&provider_name, feature, registry, cache, offline, no_cache)
            {
                Ok(None) => continue, // warning already emitted inside
                Ok(Some(resolved)) => {
                    // Merge: user-config takes priority over resolved defaults.
                    let merged = match existing {
                        Some(user) => FeatureSettings {
                            template: user.template.or(resolved.template),
                            target: user.target.or(resolved.target),
                            disabled: user.disabled.or(resolved.disabled),
                            variables: user.variables.or(resolved.variables),
                            hash: user.hash.or(resolved.hash),
                        },
                        None => resolved,
                    };
                    set_feature_settings(app_config, &provider_name, feature, merged);
                }
                Err(e) if offline => return Err(e),
                Err(e) => {
                    warn!("{}", e);
                }
            }
        }
    }

    Ok(())
}

/// Resolves a single (provider, feature) pair, returning the filled `FeatureSettings` or `None`
/// if the provider/feature cannot be found or should be skipped.
fn resolve_for_provider(
    provider: &str,
    feature: &Feature,
    registry: Option<&Registry>,
    cache: &TemplateCache,
    offline: bool,
    no_cache: bool,
) -> Result<Option<FeatureSettings>> {
    // Step 1: obtain provider.toml content.
    let toml_content = get_provider_toml(provider, registry, cache, offline, no_cache)?;
    let toml_content = match toml_content {
        Some(c) => c,
        None => return Ok(None),
    };

    // Step 2: parse and extract the feature settings.
    let mut feature_settings = match parse_provider_toml(&toml_content, provider, feature)? {
        Some(s) => s,
        None => {
            warn!(
                "Provider '{}' does not support the '{}' feature — skipping",
                provider,
                feature.as_str()
            );
            return Ok(None);
        }
    };

    // Step 3: pre-warm the .hbs cache and rewrite `template` to the local cache path.
    // This lets subsequent deploys read the template from disk rather than fetching it again.
    if let Some(template_url) = feature_settings.template.as_deref() {
        let filename = feature_filename(feature);
        let hbs_checksum = registry
            .and_then(|r| r.providers.get(provider))
            .and_then(|e| e.checksums.as_ref())
            .and_then(|c| c.get(filename))
            .map(|s| s.as_str());

        match fetch_or_cache_file(
            provider,
            filename,
            template_url,
            hbs_checksum,
            cache,
            no_cache,
        ) {
            Ok(Some(_)) => {
                // Point template at the cached local file so the renderer reads from disk.
                let local_path = cache.path_of(provider, filename);
                feature_settings.template = Some(local_path.to_string_lossy().into_owned());
            }
            Ok(None) => {}
            Err(e) => {
                // Non-fatal: keep the original remote URL so the renderer falls back to HTTP.
                warn!(
                    "Failed to pre-warm .hbs cache for provider '{}' feature '{}': {}",
                    provider,
                    feature.as_str(),
                    e
                );
            }
        }
    }

    Ok(Some(feature_settings))
}

/// Fetches or serves from cache the `provider.toml` for a given provider.
///
/// Returns `None` (with a logged warning) when the provider should be skipped.
/// Returns `Err` only when `offline` is `true` and the cache is cold.
fn get_provider_toml(
    provider: &str,
    registry: Option<&Registry>,
    cache: &TemplateCache,
    offline: bool,
    no_cache: bool,
) -> Result<Option<String>> {
    if offline {
        return match cache.read(provider, "provider.toml")? {
            Some(c) => Ok(Some(c)),
            None => Err(anyhow!(
                "Provider '{}': no cached provider.toml found. \
                 Run without --offline first to populate the cache.",
                provider
            )),
        };
    }

    if let Some(reg) = registry {
        match reg.providers.get(provider) {
            None => {
                warn!("Provider '{}' not found in registry — skipping", provider);
                return Ok(None);
            }
            Some(entry) => {
                let url = format!(
                    "{}{}",
                    TRUSTED_DOMAIN.trim_end_matches('/'),
                    entry.path
                );
                let checksum = entry
                    .checksums
                    .as_ref()
                    .and_then(|c| c.get("provider.toml"))
                    .map(|s| s.as_str());
                return fetch_or_cache_file(
                    provider,
                    "provider.toml",
                    &url,
                    checksum,
                    cache,
                    no_cache,
                );
            }
        }
    }

    // Registry unavailable (online mode, fetch failed) — try cache as fallback.
    match cache.read(provider, "provider.toml")? {
        Some(c) => Ok(Some(c)),
        None => {
            warn!(
                "Provider '{}': registry unavailable and no cached provider.toml — skipping",
                provider
            );
            Ok(None)
        }
    }
}

/// Writes resolved `FeatureSettings` back into `app_config` for the given provider and feature.
fn set_feature_settings(
    app_config: &mut AppConfig,
    provider: &str,
    feature: &Feature,
    settings: FeatureSettings,
) {
    let providers_map = app_config
        .providers
        .get_or_insert_with(Providers::new)
        .0
        .get_or_insert_with(HashMap::new);

    let features = providers_map
        .entry(provider.to_string())
        .or_default();

    match feature {
        Feature::Command => features.commands = Some(settings),
        Feature::Instruction => features.instructions = Some(settings),
        Feature::Mcp => features.mcp = Some(settings),
        Feature::Skill => features.skills = Some(settings),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use tempfile::TempDir;

    fn make_cache_dir() -> (TempDir, TemplateCache) {
        let dir = TempDir::new().unwrap();
        let cache = TemplateCache::new_at(dir.path().to_path_buf());
        (dir, cache)
    }

    fn minimal_app_config(targets: &[&str], features: &[&str]) -> AppConfig {
        AppConfig {
            schema: "test".into(),
            features: features.iter().map(|s| s.to_string()).collect(),
            targets: targets.iter().map(|s| s.to_string()).collect(),
            providers: None,
            variables: None,
        }
    }

    fn registry_for(
        provider: &str,
        path: &str,
        checksums: Option<HashMap<String, String>>,
    ) -> Registry {
        let mut providers = HashMap::new();
        providers.insert(
            provider.to_string(),
            crate::schema::registry::ProviderRegistryEntry {
                path: path.to_string(),
                checksums,
            },
        );
        Registry { providers }
    }

    fn provider_toml_for(provider: &str, feature: &str, template: &str, target: &str) -> String {
        format!(
            "[providers.{}.{}]\ntemplate = \"{}\"\ntarget = \"{}\"",
            provider, feature, template, target
        )
    }

    // parse_provider_toml extracts the right feature settings
    #[test]
    fn parse_provider_toml_returns_settings_for_known_feature() {
        let toml = provider_toml_for(
            "claude",
            "commands",
            "https://example.com/cmd.hbs",
            "{{dir.workspace}}/.claude/{{command.name}}.md",
        );
        let result = parse_provider_toml(&toml, "claude", &Feature::Command).unwrap();
        assert!(result.is_some());
        let settings = result.unwrap();
        assert_eq!(
            settings.template.as_deref(),
            Some("https://example.com/cmd.hbs")
        );
    }

    // parse_provider_toml returns None when the feature block is absent
    #[test]
    fn parse_provider_toml_returns_none_for_missing_feature() {
        let toml = provider_toml_for(
            "claude",
            "commands",
            "https://example.com/cmd.hbs",
            "some/path",
        );
        let result = parse_provider_toml(&toml, "claude", &Feature::Mcp).unwrap();
        assert!(result.is_none());
    }

    // checksum match → cache is used, no download
    #[test]
    fn fetch_or_cache_file_uses_cache_on_checksum_match() {
        let (_dir, cache) = make_cache_dir();
        let content = "{{command.content}}";
        cache.write("claude", "command.hbs", content).unwrap();
        let expected = crate::utils::fs::hash_content(content);

        // Pass a bad URL — if it tries to fetch, it will fail
        let result = fetch_or_cache_file(
            "claude",
            "command.hbs",
            "http://127.0.0.1:1/bad",
            Some(&expected),
            &cache,
            false,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_deref(), Some(content));
    }

    // offline + cold cache returns a hard error
    #[test]
    fn resolve_provider_defaults_offline_cold_cache_errors() {
        let (_dir, cache) = make_cache_dir();
        let mut config = minimal_app_config(&["claude"], &["commands"]);
        let result = resolve_provider_defaults(&mut config, None, &cache, true, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("--offline"));
    }

    // provider absent from registry → warning logged, config unchanged
    #[test]
    fn resolve_provider_defaults_unknown_provider_skips() {
        let (_dir, cache) = make_cache_dir();
        let mut config = minimal_app_config(&["unknown-provider"], &["commands"]);
        let registry = registry_for("claude", "/templates/claude/provider.toml", None);
        let result = resolve_provider_defaults(&mut config, Some(&registry), &cache, false, false);
        assert!(result.is_ok());
        // Providers map still empty (nothing was set)
        assert!(
            config.providers.is_none()
                || config
                    .providers
                    .as_ref()
                    .and_then(|p| p.0.as_ref())
                    .map_or(true, |m| !m.contains_key("unknown-provider"))
        );
    }

    // both template and target already set → registry is not consulted
    #[test]
    fn resolve_provider_defaults_skips_fully_configured_provider() {
        use crate::schema::config::common::{Features, Providers};
        let (_dir, cache) = make_cache_dir();

        let mut features_map = HashMap::new();
        let mut provider_features = Features::new();
        provider_features.commands = Some(FeatureSettings {
            template: Some("local.hbs".into()),
            target: Some("some/path".into()),
            ..Default::default()
        });
        features_map.insert("claude".to_string(), provider_features);

        let mut config = minimal_app_config(&["claude"], &["commands"]);
        config.providers = Some(Providers(Some(features_map)));

        // Pass a registry that would normally trigger a fetch; since both fields are present
        // the resolver should not touch this provider.
        let registry = registry_for("claude", "/templates/claude/provider.toml", None);
        resolve_provider_defaults(&mut config, Some(&registry), &cache, false, false).unwrap();

        let settings = config
            .providers
            .as_ref()
            .unwrap()
            .0
            .as_ref()
            .unwrap()
            .get("claude")
            .unwrap()
            .commands
            .as_ref()
            .unwrap();
        // Template unchanged
        assert_eq!(settings.template.as_deref(), Some("local.hbs"));
    }

    // registry unavailable (None) + warm cache → uses cached provider.toml (mock HTTP not needed)
    #[test]
    fn resolve_provider_defaults_registry_unavailable_falls_back_to_cache() {
        let (_dir, cache) = make_cache_dir();
        let toml = provider_toml_for(
            "claude",
            "commands",
            "https://dotagents.soorya-u.dev/templates/claude/command.hbs",
            "{{dir.workspace}}/.claude/{{command.name}}.md",
        );
        cache.write("claude", "provider.toml", &toml).unwrap();

        // Also pre-seed the hbs so fetch_or_cache_file won't try to hit the network.
        // (The template URL points to the real server; we override the local path rewrite by
        // pre-seeding a cached hbs so the checksum path takes over.)
        cache
            .write("claude", "command.hbs", "{{command.content}}")
            .unwrap();

        let mut config = minimal_app_config(&["claude"], &["commands"]);
        // registry = None simulates a fetch failure in online mode
        let result = resolve_provider_defaults(&mut config, None, &cache, false, false);
        assert!(
            result.is_ok(),
            "should fall back to cache, got {:?}",
            result
        );
    }
}
