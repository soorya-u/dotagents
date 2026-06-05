use std::collections::HashMap;

use anyhow::{Context, Result};
use cliclack::{intro, multiselect, note, outro};
use strum::VariantNames;

use crate::cli::ui::init::prompt_targets;
use crate::core::config::common::{Features, Providers};
use crate::core::config::local::LocalConfig;
use crate::core::config::{AppConfig, GlobalConfig};
use crate::core::features::Feature;

/// Displays the effective merged app configuration in the TUI.
pub(crate) fn display_tui_config(config: &AppConfig) -> Result<()> {
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

    if let Some(vars) = &config.variables
        && !vars.is_empty()
    {
        let mut keys: Vec<&String> = vars.keys().collect();
        keys.sort();
        note(
            "Variables",
            keys.iter()
                .map(|k| format!("{k} = {}", vars[*k]))
                .collect::<Vec<_>>()
                .join("\n"),
        )?;
    }

    outro("Done.")?;
    Ok(())
}

/// Displays the global configuration in the TUI.
pub(crate) fn display_tui_global(config: &GlobalConfig) -> Result<()> {
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

    if let Some(vars) = &config.variables
        && !vars.is_empty()
    {
        let mut keys: Vec<&String> = vars.keys().collect();
        keys.sort();
        note(
            "Variables",
            keys.iter()
                .map(|k| format!("{k} = {}", vars[*k]))
                .collect::<Vec<_>>()
                .join("\n"),
        )?;
    }

    outro("Done.")?;
    Ok(())
}

/// Displays the local configuration in the TUI.
pub(crate) fn display_tui_local(config: &LocalConfig) -> Result<()> {
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

    if let Some(vars) = &config.variables
        && !vars.is_empty()
    {
        let mut keys: Vec<&String> = vars.keys().collect();
        keys.sort();
        note(
            "Override Variables",
            keys.iter()
                .map(|k| format!("{k} = {}", vars[*k]))
                .collect::<Vec<_>>()
                .join("\n"),
        )?;
    }

    outro("Done.")?;
    Ok(())
}

/// Edits the global configuration through interactive prompts.
pub(crate) fn edit_global_config(config: &mut GlobalConfig) -> Result<()> {
    let all_features = Feature::VARIANTS;

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
        .context("unable to select features")?;

    config.features = selected.into_iter().map(|s| s.to_string()).collect();

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

            let retained: HashMap<String, Features> = config
                .providers
                .as_ref()
                .and_then(|p| p.0.as_ref())
                .map(|map| {
                    map.iter()
                        .filter(|(name, feats)| {
                            selected_providers.contains(*name) && feats.has_configured_overrides()
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

/// Edits the local configuration through interactive prompts.
pub(crate) fn edit_local_config(config: &mut LocalConfig) -> Result<()> {
    let all_features = Feature::VARIANTS;

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
        .context("unable to select override features")?;

    if selected.is_empty() {
        config.features = None;
    } else {
        config.features = Some(selected.into_iter().map(|s| s.to_string()).collect());
    }

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

            let retained: HashMap<String, Features> = config
                .providers
                .as_ref()
                .and_then(|p| p.0.as_ref())
                .map(|map| {
                    map.iter()
                        .filter(|(name, feats)| {
                            selected_providers.contains(*name) && feats.has_configured_overrides()
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
