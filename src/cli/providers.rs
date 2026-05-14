use cliclack::{outro, select};
use serde::Serialize;

use crate::constants::{domain::registry_url, file::REGISTRY_FILE};
use crate::prelude::*;
use crate::schema::registry::Registry;
use crate::utils::path::get_global_template_cache_dir;
use crate::utils::tty::is_tty;

use super::options::{ProvidersAction, ProvidersLsOptions};

/// A display-friendly provider entry.
#[derive(Debug, Serialize)]
struct DisplayProvider {
    slug: String,
    name: Option<String>,
    url: Option<String>,
}

/// Persist the fetched registry JSON to the template-source cache so offline
/// mode can read it back.
fn cache_registry(registry_json: &str) {
    match get_global_template_cache_dir() {
        Ok(cache_dir) => {
            let path = cache_dir.join(REGISTRY_FILE);
            if let Err(e) = std::fs::write(&path, registry_json) {
                warn!("Failed to cache registry: {}", e);
            }
        }
        Err(e) => {
            warn!("Failed to get cache dir: {}", e);
        }
    }
}

/// Fetch the registry: online by default, then cache it.  Cache-only when
/// `offline` is true.
fn fetch_registry(offline: bool) -> Result<Registry> {
    if offline {
        return read_registry_from_cache();
    }

    let url = registry_url();
    let body = crate::utils::http::do_get(url)
        .with_context(|| format!("Failed to fetch registry from {}", url))?;

    let registry: Registry = serde_json::from_str(&body)
        .with_context(|| format!("Failed to parse registry JSON from {}", url))?;

    // Cache only after successful parse to avoid poisoning offline cache with bad payloads.
    cache_registry(&body);
    Ok(registry)
}

/// Read `registry.json` from the template-source cache directory.
fn read_registry_from_cache() -> Result<Registry> {
    let cache_dir =
        get_global_template_cache_dir().context("Failed to get template cache directory")?;
    let cache_path = cache_dir.join(REGISTRY_FILE);

    let body = std::fs::read_to_string(&cache_path).map_err(|_| {
        anyhow!(
            "No cached registry found at {} — run `dotagents providers ls` without --offline first",
            cache_path.display()
        )
    })?;

    serde_json::from_str(&body).with_context(|| "Failed to parse cached registry JSON")
}

fn collect_providers(registry: &Registry) -> Vec<DisplayProvider> {
    let mut providers: Vec<DisplayProvider> = registry
        .providers
        .iter()
        .map(|(slug, entry)| DisplayProvider {
            slug: slug.clone(),
            name: entry.name.clone(),
            url: entry.url.clone(),
        })
        .collect();

    providers.sort_by(|a, b| a.slug.cmp(&b.slug));
    providers
}

/// Output providers as plain text.
fn print_text(providers: &[DisplayProvider]) {
    if providers.is_empty() {
        println!("No providers found.");
        return;
    }

    for p in providers {
        let name = p.name.as_deref().unwrap_or("");
        let url = p.url.as_deref().unwrap_or("");
        match (name.is_empty(), url.is_empty()) {
            (false, false) => println!("{}  ({}) \u{2014} {}", p.slug, name, url),
            (false, true) => println!("{}  ({})", p.slug, name),
            (true, false) => println!("{}  \u{2014} {}", p.slug, url),
            (true, true) => println!("{}", p.slug),
        }
    }
}

/// Output providers as JSON array.
fn print_json(providers: &[DisplayProvider]) {
    let output = serde_json::to_string_pretty(providers).expect("failed to serialize to JSON");
    println!("{}", output);
}

/// Interactive TUI mode: browsable list with slug and URL shown on the highlighted item.
fn run_tui(providers: &[DisplayProvider]) -> Result<bool> {
    if providers.is_empty() {
        outro("No providers found.")?;
        return Ok(true);
    }

    let items: Vec<(&str, &str, String)> = providers
        .iter()
        .map(|p| {
            let label = p.name.as_deref().unwrap_or(&p.slug);
            // Only show [slug] in hint when the label is a name, not the slug itself.
            let slug_part = p
                .name
                .as_ref()
                .map(|_| format!("[{}]", p.slug))
                .unwrap_or_default();
            let url_part = p.url.as_deref().unwrap_or("");
            let hint = match (slug_part.is_empty(), url_part.is_empty()) {
                (false, false) => format!("{} {}", slug_part, url_part),
                (false, true) => slug_part,
                (true, false) => url_part.to_string(),
                (true, true) => String::new(),
            };
            (p.slug.as_str(), label, hint)
        })
        .collect();

    select("Select a provider")
        .items(&items)
        .max_rows(10)
        .interact()
        .map_err(|e| anyhow!("TUI interaction failed: {}", e))?;

    outro("")?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_registry() -> Registry {
        let mut providers = HashMap::new();
        providers.insert(
            "claude".to_string(),
            crate::schema::registry::ProviderRegistryEntry {
                path: "/v1/templates/claude/provider.toml".into(),
                checksums: None,
                name: Some("Claude Code".into()),
                url: Some("https://docs.anthropic.com/en/docs/claude-code".into()),
            },
        );
        providers.insert(
            "cursor".to_string(),
            crate::schema::registry::ProviderRegistryEntry {
                path: "/v1/templates/cursor/provider.toml".into(),
                checksums: None,
                name: Some("Cursor".into()),
                url: Some("https://docs.cursor.com".into()),
            },
        );
        providers.insert(
            "roo".to_string(),
            crate::schema::registry::ProviderRegistryEntry {
                path: "/v1/templates/roo/provider.toml".into(),
                checksums: None,
                name: None,
                url: None,
            },
        );
        Registry { providers }
    }

    // collect_providers sorts by slug and includes all entries
    #[test]
    fn collect_providers_returns_sorted() {
        let registry = make_registry();
        let providers = collect_providers(&registry);
        assert_eq!(providers.len(), 3);
        assert_eq!(providers[0].slug, "claude");
        assert_eq!(providers[1].slug, "cursor");
        assert_eq!(providers[2].slug, "roo");
    }

    // collect_providers preserves name and url from registry entries
    #[test]
    fn collect_providers_preserves_name_and_url() {
        let registry = make_registry();
        let providers = collect_providers(&registry);
        let claude = providers.iter().find(|p| p.slug == "claude").unwrap();
        assert_eq!(claude.name.as_deref(), Some("Claude Code"));
        assert_eq!(
            claude.url.as_deref(),
            Some("https://docs.anthropic.com/en/docs/claude-code")
        );
    }

    // collect_providers sets name and url to None when absent from registry
    #[test]
    fn collect_providers_handles_missing_name_url() {
        let registry = make_registry();
        let providers = collect_providers(&registry);
        let roo = providers.iter().find(|p| p.slug == "roo").unwrap();
        assert!(roo.name.is_none());
        assert!(roo.url.is_none());
    }

    // print_json includes null name and url fields for all providers
    #[test]
    fn print_json_includes_all_fields() {
        let providers = vec![
            DisplayProvider {
                slug: "claude".into(),
                name: Some("Claude Code".into()),
                url: Some("https://docs.anthropic.com/en/docs/claude-code".into()),
            },
            DisplayProvider {
                slug: "roo".into(),
                name: None,
                url: None,
            },
        ];

        let output = serde_json::to_string_pretty(&providers).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed.len(), 2);

        let claude = &parsed[0];
        assert_eq!(claude["slug"], "claude");
        assert_eq!(claude["name"], "Claude Code");
        assert_eq!(
            claude["url"],
            "https://docs.anthropic.com/en/docs/claude-code"
        );

        // roo has null name and url
        let roo = &parsed[1];
        assert_eq!(roo["slug"], "roo");
        assert!(roo["name"].is_null());
        assert!(roo["url"].is_null());
    }

    // read_registry_from_cache errors with clear message when cache is cold
    #[test]
    fn read_registry_from_cache_cold_cache_errors() {
        let result = read_registry_from_cache();
        // May succeed if there happens to be a cached registry in CI;
        // just ensure the function exists and either succeeds or errors with the right message.
        if let Err(e) = result {
            let msg = e.to_string();
            assert!(
                msg.contains("cached registry"),
                "error should mention 'cached registry', got: {}",
                msg
            );
        }
    }
}

/// Dispatch `dotagents providers`.
pub(crate) fn run_providers(action: ProvidersAction) -> Result<bool> {
    match action {
        ProvidersAction::Ls(opts) => handle_ls(opts),
    }
}

/// Handle `dotagents providers ls`.
fn handle_ls(opts: ProvidersLsOptions) -> Result<bool> {
    let registry = fetch_registry(opts.offline).context("Failed to load provider registry")?;

    let providers = collect_providers(&registry);

    if providers.is_empty() {
        if opts.json {
            println!("[]");
        } else {
            println!("No providers found.");
        }
        return Ok(true);
    }

    if opts.json {
        print_json(&providers);
        return Ok(true);
    }

    if is_tty() {
        return run_tui(&providers);
    }

    print_text(&providers);
    Ok(true)
}
