use std::collections::HashMap;
use std::fs;
use std::io::IsTerminal;
use std::path::Path;

use anyhow::{Context, Result, bail};
use cliclack::{intro, multiselect, note, outro, spinner};
use serde::Serialize;

use crate::cli::options::ConfigTarget;
use crate::cli::ui::init::prompt_targets;
use crate::constants::dir::ROOT_DIR;
use crate::constants::file::{GLOBAL_CONFIG_FILE, LOCAL_CONFIG_FILE};
use crate::core::config::common::{FeatureSettings, Features, Providers};
use crate::core::config::local::LocalConfig;
use crate::core::config::{AppConfig, GlobalConfig, TomlConfig};
use crate::core::features::Feature;

use crate::templates::get_templater;
use crate::utils::fs::write_file;
use crate::utils::path::get_workspace_dir;

/// Validate `--edit` constraints: reject on `app` target and in non-TTY mode.
pub(crate) fn validate_edit(target: &ConfigTarget, edit: bool, is_tty: bool) -> Result<()> {
    if !edit {
        return Ok(());
    }
    if *target == ConfigTarget::App {
        bail!(
            "The `app` config is derived from global and local configs and cannot be edited directly. Use `dotagents config global --edit` or `dotagents config local --edit` instead."
        );
    }
    if !is_tty {
        bail!(
            "Interactive editing requires a terminal. Run this command in a TTY or edit the config file manually."
        );
    }
    Ok(())
}

/// Top-level handler for `dotagents config`.
pub(crate) fn handle(target: ConfigTarget, json: bool, edit: bool) -> Result<bool> {
    validate_edit(&target, edit, std::io::stdin().is_terminal())?;

    let workspace = get_workspace_dir().context(
        "No .dotagents directory found. Run `dotagents init` to initialise a workspace.",
    )?;
    let root_dir = workspace.join(ROOT_DIR);

    match target {
        ConfigTarget::App => {
            if json {
                handle_app_json()?;
            } else if std::io::stdin().is_terminal() && !edit {
                handle_app_tui()?;
            } else {
                handle_app_text()?;
            }
        }
        ConfigTarget::Global => {
            let path = root_dir.join(GLOBAL_CONFIG_FILE);
            if json {
                handle_global_json(&path)?;
            } else if edit {
                handle_global_edit(&path)?;
            } else if std::io::stdin().is_terminal() {
                handle_global_tui(&path)?;
            } else {
                handle_global_text(&path)?;
            }
        }
        ConfigTarget::Local => {
            let path = root_dir.join(LOCAL_CONFIG_FILE);
            if json {
                handle_local_json(&path)?;
            } else if edit {
                handle_local_edit(&path)?;
            } else if std::io::stdin().is_terminal() {
                handle_local_tui(&path)?;
            } else {
                handle_local_text(&path)?;
            }
        }
    }

    Ok(true)
}

// ── App config handlers ──────────────────────────────────────────────

fn handle_app_json() -> Result<()> {
    let config = load_app_config()?;
    let display = AppDisplay::from_app_config(&config);
    let json =
        serde_json::to_string_pretty(&display).context("Failed to serialize config to JSON")?;
    println!("{json}");
    Ok(())
}

fn handle_app_text() -> Result<()> {
    let config = load_app_config()?;
    print_app_config(&config);
    Ok(())
}

fn handle_app_tui() -> Result<()> {
    let config = load_app_config()?;
    display_tui_config(&config)?;
    Ok(())
}

// ── Global config handlers ──────────────────────────────────────────

fn handle_global_json(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("Global config not found at {}", path.display());
    }
    let content = fs::read_to_string(path).context("Failed to read global config")?;
    let config: GlobalConfig = GlobalConfig::from_toml(&content)?;
    let json = serde_json::to_string_pretty(&config)
        .context("Failed to serialize global config to JSON")?;
    println!("{json}");
    Ok(())
}

fn handle_global_text(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("Global config not found at {}", path.display());
    }
    let content = fs::read_to_string(path).context("Failed to read global config")?;
    println!("{}", content.trim());
    Ok(())
}

fn handle_global_tui(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("Global config not found at {}", path.display());
    }
    let content = fs::read_to_string(path).context("Failed to read global config")?;
    let config: GlobalConfig = GlobalConfig::from_toml(&content)?;
    display_tui_global(&config)?;
    Ok(())
}

fn handle_global_edit(path: &Path) -> Result<()> {
    intro("Edit Global Configuration")?;

    let spin = spinner();
    spin.start("Reading global config...");
    let existing_content = if path.exists() {
        fs::read_to_string(path).context("Failed to read global config")?
    } else {
        String::new()
    };
    let mut config: GlobalConfig = if existing_content.is_empty() {
        GlobalConfig::new()
    } else {
        GlobalConfig::from_toml(&existing_content)?
    };
    spin.clear();

    edit_global_config(&mut config)?;

    let spin = spinner();
    spin.start("Writing global config...");
    let content = config.to_toml()?;
    write_file(&path.to_path_buf(), &content).context("Failed to write global config")?;
    spin.stop("Global config updated.");

    outro("Done.")?;
    Ok(())
}

// ── Local config handlers ───────────────────────────────────────────

fn handle_local_json(path: &Path) -> Result<()> {
    if !path.exists() {
        println!("{{}}");
        return Ok(());
    }
    let content = fs::read_to_string(path).context("Failed to read local config")?;
    let config: LocalConfig = LocalConfig::from_toml(&content)?;
    let json = serde_json::to_string_pretty(&config)
        .context("Failed to serialize local config to JSON")?;
    println!("{json}");
    Ok(())
}

fn handle_local_text(path: &Path) -> Result<()> {
    if !path.exists() {
        println!("No local config found at {}", path.display());
        return Ok(());
    }
    let content = fs::read_to_string(path).context("Failed to read local config")?;
    println!("{}", content.trim());
    Ok(())
}

fn handle_local_tui(path: &Path) -> Result<()> {
    if !path.exists() {
        println!("No local config found at {}", path.display());
        return Ok(());
    }
    let content = fs::read_to_string(path).context("Failed to read local config")?;
    let config: LocalConfig = LocalConfig::from_toml(&content)?;
    display_tui_local(&config)?;
    Ok(())
}

fn handle_local_edit(path: &Path) -> Result<()> {
    intro("Edit Local Configuration")?;

    let spin = spinner();
    spin.start("Reading local config...");
    let existing_content = if path.exists() {
        fs::read_to_string(path).context("Failed to read local config")?
    } else {
        String::new()
    };
    let mut config: LocalConfig = if existing_content.is_empty() {
        LocalConfig::new()
    } else {
        LocalConfig::from_toml(&existing_content)?
    };
    spin.clear();

    edit_local_config(&mut config)?;

    let spin = spinner();
    spin.start("Writing local config...");
    let content = config.to_toml()?;
    write_file(&path.to_path_buf(), &content).context("Failed to write local config")?;
    spin.stop("Local config updated.");

    outro("Done.")?;
    Ok(())
}

// ── App display helpers ─────────────────────────────────────────────

fn load_app_config() -> Result<AppConfig> {
    let templater = get_templater().context("Failed to initialise templater")?;
    let config =
        AppConfig::from_application(templater).context("Failed to load application config")?;
    Ok(config)
}

fn print_app_config(config: &AppConfig) {
    println!("=== Merged App Config ===");
    println!("Schema: {}", config.schema);

    println!("\nFeatures:");
    if config.features.is_empty() {
        println!("  (none configured)");
    } else {
        let mut features: Vec<&String> = config.features.iter().collect();
        features.sort();
        for f in features {
            println!("  - {f}");
        }
    }

    println!("\nTargets:");
    if config.targets.is_empty() {
        println!("  (none configured)");
    } else {
        let mut targets: Vec<&String> = config.targets.iter().collect();
        targets.sort();
        for t in targets {
            println!("  - {t}");
        }
    }

    if let Some(vars) = &config.variables
        && !vars.is_empty()
    {
        println!("\nVariables:");
        let mut keys: Vec<&String> = vars.keys().collect();
        keys.sort();
        for k in keys {
            println!("  {k} = {}", vars[k]);
        }
    }

    if let Some(runner) = &config.package_runner {
        println!(
            "\nPackage runner: {}",
            serde_json::to_string(runner).unwrap_or_default()
        );
    }

    if let Some(providers) = &config.providers
        && let Some(map) = &providers.0
    {
        println!("\nProviders:");
        let mut names: Vec<&String> = map.keys().collect();
        names.sort();
        for name in names {
            let feats = &map[name];
            println!("  {name}:");
            print_feature_settings("commands", &feats.commands);
            print_feature_settings("instructions", &feats.instructions);
            print_feature_settings("mcp", &feats.mcp);
            print_feature_settings("skills", &feats.skills);
        }
    }
    println!();
}

fn print_feature_settings(label: &str, settings: &Option<FeatureSettings>) {
    if let Some(s) = settings {
        println!("    {label}:");
        if let Some(tmpl) = &s.template {
            println!("      template: {tmpl}");
        }
        if let Some(tgt) = &s.target {
            println!("      target: {tgt}");
        }
        if let Some(d) = s.disabled {
            println!("      disabled: {d}");
        }
    }
}

// ── TUI display ─────────────────────────────────────────────────────

fn display_tui_config(config: &AppConfig) -> Result<()> {
    intro("Effective Configuration")?;

    let mut sorted_features: Vec<&String> = config.features.iter().collect();
    sorted_features.sort();

    if !sorted_features.is_empty() {
        note(
            "Active Features",
            sorted_features
                .iter()
                .map(|f| f.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
        )?;
    }

    {
        let mut sorted_targets: Vec<&String> = config.targets.iter().collect();
        sorted_targets.sort();
        let targets_body = if sorted_targets.is_empty() {
            "(none configured)".to_string()
        } else {
            sorted_targets
                .iter()
                .map(|t| t.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        };
        note("Targets", &targets_body)?;
    }

    if let Some(providers) = &config.providers
        && let Some(map) = &providers.0
    {
        let mut sorted_providers: Vec<&String> = map.keys().collect();
        sorted_providers.sort();
        if !sorted_providers.is_empty() {
            note(
                "Providers",
                sorted_providers
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join("\n"),
            )?;
        }
    }

    outro("Done.")?;
    Ok(())
}

fn display_tui_global(config: &GlobalConfig) -> Result<()> {
    intro("Global Configuration")?;

    let mut features: Vec<&String> = config.features.iter().collect();
    features.sort();
    if !features.is_empty() {
        note(
            "Features",
            features
                .iter()
                .map(|f| f.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
        )?;
    }

    {
        let targets = config.targets.as_ref();
        let mut sorted: Vec<&String> = targets.iter().flat_map(|s| s.iter()).collect();
        sorted.sort();
        let targets_body = if sorted.is_empty() {
            "(none configured)".to_string()
        } else {
            sorted
                .iter()
                .map(|t| t.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        };
        note("Targets", &targets_body)?;
    }

    if let Some(providers) = &config.providers
        && let Some(map) = &providers.0
    {
        let mut sorted: Vec<&String> = map.keys().collect();
        sorted.sort();
        if !sorted.is_empty() {
            note(
                "Providers",
                sorted
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join("\n"),
            )?;
        }
    }

    outro("Done.")?;
    Ok(())
}

fn display_tui_local(config: &LocalConfig) -> Result<()> {
    intro("Local Configuration")?;

    if let Some(features) = &config.features {
        let mut sorted: Vec<&String> = features.iter().collect();
        sorted.sort();
        if !sorted.is_empty() {
            note(
                "Override Features",
                sorted
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join("\n"),
            )?;
        }
    }

    {
        let targets = config.targets.as_ref();
        let mut sorted: Vec<&String> = targets.iter().flat_map(|s| s.iter()).collect();
        sorted.sort();
        let targets_body = if sorted.is_empty() {
            "(none configured)".to_string()
        } else {
            sorted
                .iter()
                .map(|t| t.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        };
        note("Override Targets", &targets_body)?;
    }

    if let Some(providers) = &config.providers
        && let Some(map) = &providers.0
    {
        let mut sorted: Vec<&String> = map.keys().collect();
        sorted.sort();
        if !sorted.is_empty() {
            note(
                "Override Providers",
                sorted
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join("\n"),
            )?;
        }
    }

    outro("Done.")?;
    Ok(())
}

// ── TUI editor ──────────────────────────────────────────────────────

/// Returns true when a provider has at least one feature override worth preserving.
fn features_has_overrides(feats: &Features) -> bool {
    feats.commands.is_some()
        || feats.instructions.is_some()
        || feats.mcp.is_some()
        || feats.skills.is_some()
}

fn edit_global_config(config: &mut GlobalConfig) -> Result<()> {
    let all_features = Feature::all_names();

    let current_features: Vec<&str> = all_features
        .iter()
        .copied()
        .filter(|f| config.features.contains(*f))
        .collect();

    let selected = multiselect("Select active features")
        .items(
            &all_features
                .iter()
                .map(|f| (*f, *f, ""))
                .collect::<Vec<_>>(),
        )
        .initial_values(current_features)
        .required(false)
        .interact()
        .context("Failed to select features")?;

    config.features = selected.into_iter().map(|s| s.to_string()).collect();

    // Pre-select from targets (the canonical "who to deploy to" list).
    let existing_provider_names: Vec<String> = config
        .targets
        .as_ref()
        .map(|set| set.iter().cloned().collect())
        .unwrap_or_default();

    if let Some(selected_providers) = prompt_targets(&existing_provider_names)? {
        if selected_providers.is_empty() {
            config.targets = None;
            config.providers = None;
        } else {
            config.targets = Some(selected_providers.iter().cloned().collect());

            // Retain only overrides for selected providers that have real config.
            // New registry-backed providers need no [providers.X] entry.
            let retained: HashMap<String, Features> = config
                .providers
                .as_ref()
                .and_then(|p| p.0.as_ref())
                .map(|map| {
                    map.iter()
                        .filter(|(name, feats)| {
                            selected_providers.contains(*name) && features_has_overrides(feats)
                        })
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect()
                })
                .unwrap_or_default();

            config.providers = if retained.is_empty() {
                None
            } else {
                Some(Providers(Some(retained)))
            };
        }
    }

    Ok(())
}

fn edit_local_config(config: &mut LocalConfig) -> Result<()> {
    let all_features = Feature::all_names();

    let current_features: Vec<&str> = all_features
        .iter()
        .copied()
        .filter(|f| config.features.as_ref().is_some_and(|set| set.contains(*f)))
        .collect();

    let selected = multiselect("Select override features")
        .items(
            &all_features
                .iter()
                .map(|f| (*f, *f, ""))
                .collect::<Vec<_>>(),
        )
        .initial_values(current_features)
        .required(false)
        .interact()
        .context("Failed to select features")?;

    if selected.is_empty() {
        config.features = None;
    } else {
        config.features = Some(selected.into_iter().map(|s| s.to_string()).collect());
    }

    // Pre-select from targets (the canonical "who to deploy to" list).
    let existing_provider_names: Vec<String> = config
        .targets
        .as_ref()
        .map(|set| set.iter().cloned().collect())
        .unwrap_or_default();

    if let Some(selected_providers) = prompt_targets(&existing_provider_names)? {
        if selected_providers.is_empty() {
            config.targets = None;
            config.providers = None;
        } else {
            config.targets = Some(selected_providers.iter().cloned().collect());

            // Retain only overrides for selected providers that have real config.
            // New registry-backed providers need no [providers.X] entry.
            let retained: HashMap<String, Features> = config
                .providers
                .as_ref()
                .and_then(|p| p.0.as_ref())
                .map(|map| {
                    map.iter()
                        .filter(|(name, feats)| {
                            selected_providers.contains(*name) && features_has_overrides(feats)
                        })
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect()
                })
                .unwrap_or_default();

            config.providers = if retained.is_empty() {
                None
            } else {
                Some(Providers(Some(retained)))
            };
        }
    }

    Ok(())
}

// ── Display-friendly JSON structures ─────────────────────────────────

#[derive(Debug, Serialize)]
struct AppDisplay {
    features: Vec<String>,
    targets: Vec<String>,
    providers: Vec<ProviderDisplay>,
}

#[derive(Debug, Serialize)]
struct ProviderDisplay {
    name: String,
    commands: Option<FeatureSettings>,
    instructions: Option<FeatureSettings>,
    mcp: Option<FeatureSettings>,
    skills: Option<FeatureSettings>,
}

impl AppDisplay {
    fn from_app_config(config: &AppConfig) -> Self {
        let mut features: Vec<String> = config.features.iter().cloned().collect();
        features.sort();

        let providers = match &config.providers {
            Some(p) => match &p.0 {
                Some(map) => {
                    let mut list: Vec<ProviderDisplay> = map
                        .iter()
                        .map(|(name, feats)| ProviderDisplay {
                            name: name.clone(),
                            commands: feats.commands.clone(),
                            instructions: feats.instructions.clone(),
                            mcp: feats.mcp.clone(),
                            skills: feats.skills.clone(),
                        })
                        .collect();
                    list.sort_by(|a, b| a.name.cmp(&b.name));
                    list
                }
                None => vec![],
            },
            None => vec![],
        };

        let mut targets: Vec<String> = config.targets.iter().cloned().collect();
        targets.sort();

        AppDisplay {
            features,
            targets,
            providers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── validate_edit tests ─────────────────────────────────────────

    #[test]
    fn validate_edit_ok_when_not_editing() {
        // No error when edit is false regardless of target or TTY
        assert!(validate_edit(&ConfigTarget::App, false, true).is_ok());
        assert!(validate_edit(&ConfigTarget::Global, false, false).is_ok());
    }

    #[test]
    fn validate_edit_rejects_app_target() {
        // --edit on app target should fail
        let err = validate_edit(&ConfigTarget::App, true, true).unwrap_err();
        assert!(err.to_string().contains("cannot be edited directly"));
    }

    #[test]
    fn validate_edit_rejects_non_tty() {
        // --edit in non-TTY mode should fail
        let err = validate_edit(&ConfigTarget::Global, true, false).unwrap_err();
        assert!(err.to_string().contains("requires a terminal"));
    }

    #[test]
    fn validate_edit_accepts_global_tty() {
        // --edit on global target in TTY should pass
        assert!(validate_edit(&ConfigTarget::Global, true, true).is_ok());
    }

    #[test]
    fn validate_edit_accepts_local_tty() {
        // --edit on local target in TTY should pass
        assert!(validate_edit(&ConfigTarget::Local, true, true).is_ok());
    }

    // ── AppDisplay tests ────────────────────────────────────────────

    #[test]
    fn app_display_from_empty_config() {
        // Empty AppConfig produces empty display
        let config = AppConfig::new();
        let display = AppDisplay::from_app_config(&config);
        assert!(display.features.is_empty());
        assert!(display.targets.is_empty());
        assert!(display.providers.is_empty());
    }

    #[test]
    fn app_display_features_sorted() {
        // Features appear sorted in the display
        let mut config = AppConfig::new();
        config.features = ["mcp", "commands", "skills"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let display = AppDisplay::from_app_config(&config);
        assert_eq!(display.features, vec!["commands", "mcp", "skills"]);
    }

    #[test]
    fn app_display_providers_sorted() {
        // Providers appear sorted by name in the display
        let mut config = AppConfig::new();
        config.features = ["commands"].iter().map(|s| s.to_string()).collect();
        let mut map = HashMap::new();
        let mut feats_z = Features::default();
        feats_z.commands = Some(FeatureSettings {
            template: Some("z.tmpl".into()),
            ..Default::default()
        });
        let mut feats_a = Features::default();
        feats_a.commands = Some(FeatureSettings {
            template: Some("a.tmpl".into()),
            ..Default::default()
        });
        map.insert("zebra".into(), feats_z);
        map.insert("alpha".into(), feats_a);
        config.providers = Some(Providers(Some(map)));

        let display = AppDisplay::from_app_config(&config);
        assert_eq!(display.providers.len(), 2);
        assert_eq!(display.providers[0].name, "alpha");
        assert_eq!(display.providers[1].name, "zebra");
    }

    #[test]
    fn app_display_json_serializes() {
        // AppDisplay round-trips through JSON without error
        let mut config = AppConfig::new();
        config.features = ["commands"].iter().map(|s| s.to_string()).collect();
        let display = AppDisplay::from_app_config(&config);
        let json = serde_json::to_string(&display).unwrap();
        assert!(json.contains("commands"));
        assert!(json.contains("providers"));
        assert!(json.contains("targets"));
    }

    #[test]
    fn app_display_provider_detail() {
        // Provider detail fields (template, target) appear in output
        let mut config = AppConfig::new();
        config.features = ["commands"].iter().map(|s| s.to_string()).collect();
        let mut map = HashMap::new();
        let mut feats = Features::default();
        feats.commands = Some(FeatureSettings {
            template: Some("test.tmpl".into()),
            target: Some("output.md".into()),
            disabled: Some(false),
            ..Default::default()
        });
        map.insert("claude".into(), feats);
        config.providers = Some(Providers(Some(map)));

        let display = AppDisplay::from_app_config(&config);
        assert_eq!(display.providers.len(), 1);
        let p = &display.providers[0];
        assert_eq!(p.name, "claude");
        let cmd = p.commands.as_ref().unwrap();
        assert_eq!(cmd.template.as_deref(), Some("test.tmpl"));
        assert_eq!(cmd.target.as_deref(), Some("output.md"));
        assert_eq!(cmd.disabled, Some(false));
    }
}
