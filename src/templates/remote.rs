use std::collections::HashMap;

use crate::prelude::*;

use crate::constants::domain::TRUSTED_DOMAIN;
use crate::core::config::{AppConfig, FeatureSettings, GlobalConfig, Providers, TomlConfig};
use crate::core::features::Feature;
use crate::schema::registry::Registry;
use crate::templates::TemplateCache;
use crate::utils::hash::hash_content;
use crate::utils::http::fetch_template;

pub(crate) use crate::constants::domain::registry_url;

/// Fetches `url` or serves from cache; bypasses cache entirely when `no_cache` is `true`.
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

    let content = fetch_template(url).map_err(|e| {
        anyhow!(
            "Failed to fetch {} for provider '{}': {}",
            filename,
            provider,
            e
        )
    })?;
    if let Some(expected) = registry_checksum {
        let actual = hash_content(&content);
        if actual != expected {
            return Err(anyhow!(
                "Checksum mismatch for {} (provider '{}'): expected {}, got {}",
                filename,
                provider,
                expected,
                actual
            ));
        }
    }
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

/// Fills in missing `template`/`target` fields in `app_config` from the registry/cache, honouring `offline` and `no_cache` flags.
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
        Feature::AgentIgnore,
    ];

    for provider_name in app_config.targets.clone() {
        // If the registry is available and this provider is absent, emit a single warning
        // for all features rather than one per feature.  When the registry is unavailable
        // (None), fall through to the per-feature cache-fallback path.
        if let Some(reg) = registry
            && !reg.providers.contains_key(&provider_name)
        {
            // Only warn when at least one feature actually needs resolution; don't
            // emit spurious warnings for fully-configured custom providers.
            let needs_any = all_features.iter().any(|f| {
                if !app_config.has_feature(f) {
                    return false;
                }
                let s = app_config
                    .providers
                    .as_ref()
                    .and_then(|p| p.0.as_ref())
                    .and_then(|m| m.get(&provider_name))
                    .and_then(|fs| fs.get_config(f));
                s.is_none_or(|s| s.template.is_none() || s.target.is_none())
            });
            if needs_any {
                warn!(
                    "Provider '{}' not found in registry — skipping",
                    &provider_name
                );
            }
            continue;
        }

        // Skip the network fetch entirely when every active feature is already fully configured.
        let needs_any = all_features.iter().any(|f| {
            if !app_config.has_feature(f) {
                return false;
            }
            let s = app_config
                .providers
                .as_ref()
                .and_then(|p| p.0.as_ref())
                .and_then(|m| m.get(&provider_name))
                .and_then(|fs| fs.get_config(f));
            s.is_none_or(|s| s.template.is_none() || s.target.is_none())
        });
        if !needs_any {
            continue;
        }

        // Fetch provider.toml once for all features so the warning fires at most once per provider.
        let toml_content =
            match get_provider_toml(&provider_name, registry, cache, offline, no_cache) {
                Ok(Some(c)) => c,
                Ok(None) => continue,
                Err(e) if offline => return Err(e),
                Err(e) => {
                    warn!(
                        "Failed to fetch provider.toml for provider '{}': {}",
                        &provider_name, e
                    );
                    continue;
                }
            };

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
            match resolve_for_provider(
                &provider_name,
                feature,
                &toml_content,
                registry,
                cache,
                no_cache,
            ) {
                Ok(None) => continue,
                Ok(Some(resolved)) => {
                    // Merge: user-config takes priority over resolved defaults.
                    let merged = existing
                        .map(|user| resolved.merge(&user))
                        .unwrap_or(resolved);
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

/// Resolves a single (provider, feature) pair from pre-fetched `toml_content`, returning `None` when the feature should be skipped.
fn resolve_for_provider(
    provider: &str,
    feature: &Feature,
    toml_content: &str,
    registry: Option<&Registry>,
    cache: &TemplateCache,
    no_cache: bool,
) -> Result<Option<FeatureSettings>> {
    // Step 1: parse and extract the feature settings.
    let mut feature_settings = match parse_provider_toml(toml_content, provider, feature)? {
        Some(s) => s,
        None => {
            warn!(
                "Provider '{}' does not support the '{}' feature — skipping",
                provider,
                feature.as_ref()
            );
            return Ok(None);
        }
    };

    // Step 3: pre-warm the .hbs cache and rewrite `template` to the local cache path.
    // This lets subsequent deploys read the template from disk rather than fetching it again.
    if let Some(template_url) = feature_settings.template.as_deref() {
        let filename = feature.feature_filename();
        let hbs_checksum = registry
            .and_then(|r| r.providers.get(provider))
            .and_then(|e| e.checksums.as_ref())
            .and_then(|c| c.get(&filename))
            .map(|s| s.as_str());

        match fetch_or_cache_file(
            provider,
            &filename,
            template_url,
            hbs_checksum,
            cache,
            no_cache,
        ) {
            Ok(Some(_)) => {
                // Point template at the cached local file so the renderer reads from disk.
                let local_path = cache.path_of(provider, &filename);
                feature_settings.template = Some(local_path.to_string_lossy().into_owned());
            }
            Ok(None) => {}
            Err(e) => {
                // Non-fatal: keep the original remote URL so the renderer falls back to HTTP.
                warn!(
                    "Failed to pre-warm .hbs cache for provider '{}' feature '{}': {}",
                    provider,
                    feature.as_ref(),
                    e
                );
            }
        }
    }

    Ok(Some(feature_settings))
}

/// Fetches or reads from cache the `provider.toml` for a given provider; returns `None` when skipped and `Err` only in offline+cold-cache mode.
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
            None => {
                warn!(
                    "Provider '{}': no cached provider.toml found (run without --offline to populate the cache) — skipping",
                    provider
                );
                Ok(None)
            }
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
                    "{}/{}",
                    TRUSTED_DOMAIN.trim_end_matches('/'),
                    entry.path.trim_start_matches('/'),
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

    let features = providers_map.entry(provider.to_string()).or_default();

    match feature {
        Feature::Command => features.commands = Some(settings),
        Feature::Instruction => features.instructions = Some(settings),
        Feature::Mcp => features.mcp = Some(settings),
        Feature::Skill => features.skills = Some(settings),
        Feature::AgentIgnore => features.ignore = Some(settings),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
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
            #[cfg(feature = "skills-add")]
            package_runner: None,
            extra: std::collections::HashMap::new(),
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
                name: None,
                url: None,
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

    // checksum match -> cache is used, no download
    #[test]
    fn fetch_or_cache_file_uses_cache_on_checksum_match() {
        let (_dir, cache) = make_cache_dir();
        let content = "{{command.content}}";
        cache.write("claude", "command.hbs", content).unwrap();
        let expected = hash_content(content);

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

    // offline + cold cache skips the provider and succeeds (no hard error)
    #[test]
    fn resolve_provider_defaults_offline_cold_cache_errors() {
        let (_dir, cache) = make_cache_dir();
        let mut config = minimal_app_config(&["claude"], &["commands"]);
        let result = resolve_provider_defaults(&mut config, None, &cache, true, false);
        assert!(result.is_ok());
    }

    // provider absent from registry -> warning logged, config unchanged
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

    // both template and target already set -> registry is not consulted
    #[test]
    fn resolve_provider_defaults_skips_fully_configured_provider() {
        use crate::core::config::common::{Features, Providers};
        let (_dir, cache) = make_cache_dir();

        let mut features_map = HashMap::new();
        let mut provider_features = Features::default();
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

    // registry unavailable (None) + warm cache -> uses cached provider.toml (mock HTTP not needed)
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
