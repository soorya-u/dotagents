use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};

#[cfg(feature = "skills-add")]
use super::common::PackageRunner;
use super::common::Providers;
use super::global::GlobalConfig;
use super::local::LocalConfig;
use crate::constants::file::{GLOBAL_CONFIG_FILE, LOCAL_CONFIG_FILE};
use crate::constants::schema::CONFIG_SCHEMA;
use crate::core::config::{FeatureSettings, TomlConfig};
use crate::core::features::Feature;
use crate::templates::{RenderType, Templater};
use crate::utils::merge::merge_optional;
use crate::utils::path::get_application_dir;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    pub schema: String,
    pub features: HashSet<String>,
    pub targets: HashSet<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<Providers>,
    pub variables: Option<HashMap<String, String>>,
    #[cfg(feature = "skills-add")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_runner: Option<PackageRunner>,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            schema: CONFIG_SCHEMA.into(),
            features: HashSet::new(),
            targets: HashSet::new(),
            providers: None,
            variables: None,
            #[cfg(feature = "skills-add")]
            package_runner: None,
        }
    }

    pub fn has_feature(&self, feature: &Feature) -> bool {
        self.features.contains(feature.as_str())
    }

    pub fn get_provider_feature_settings(
        &self,
        feature: &Feature,
    ) -> HashMap<String, FeatureSettings> {
        let Some(providers) = &self.providers else {
            return HashMap::new();
        };

        let Some(provider_map) = &providers.0 else {
            return HashMap::new();
        };

        let has_feature = self.has_feature(feature);

        provider_map
            .iter()
            .filter_map(|(name, settings)| {
                let config = settings.get_config(feature)?;
                let is_disabled = config.disabled.unwrap_or(false);

                if has_feature && !is_disabled {
                    Some((name.clone(), config.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn from_application(templater: &Templater) -> Result<Self> {
        let global_config_content = templater
            .render_template(RenderType::Name(GLOBAL_CONFIG_FILE.into()), None)
            .context("unable to render global config")?;

        let local_config = {
            let local_config_path = get_application_dir()?.join(LOCAL_CONFIG_FILE);
            if local_config_path.exists() {
                let content = templater
                    .render_template(RenderType::Name(LOCAL_CONFIG_FILE.into()), None)
                    .context("unable to render local config")?;
                let config = LocalConfig::from_toml(&content)?;
                config.validate().context("invalid local config")?;
                config
            } else {
                LocalConfig::default()
            }
        };

        let global_config = GlobalConfig::from_toml(&global_config_content)?;
        global_config.validate().context("invalid global config")?;

        let app_config = AppConfig::from((&global_config, &local_config));

        Ok(app_config)
    }
}

impl From<(&GlobalConfig, &LocalConfig)> for AppConfig {
    fn from((global, local): (&GlobalConfig, &LocalConfig)) -> Self {
        let schema = local
            .schema
            .clone()
            .or_else(|| global.schema.clone())
            .unwrap_or_else(|| CONFIG_SCHEMA.into());

        let features = local
            .features
            .clone()
            .unwrap_or_else(|| global.features.clone());

        let targets = local
            .targets
            .clone()
            .or(global.targets.clone())
            .unwrap_or_default();

        let providers = merge_optional(
            global.providers.as_ref(),
            local.providers.as_ref(),
            |g, l| g.merge(l),
        );

        let variables = merge_optional(
            global.variables.as_ref(),
            local.variables.as_ref(),
            |g, l| {
                let mut merged = g.clone();
                merged.extend(l.clone());
                merged
            },
        );

        #[cfg(feature = "skills-add")]
        let package_runner = local
            .package_runner
            .clone()
            .or_else(|| global.package_runner.clone());

        Self {
            schema,
            features,
            targets,
            providers,
            variables,
            #[cfg(feature = "skills-add")]
            package_runner,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(debug_assertions)]
impl TomlConfig for AppConfig {}

#[cfg(all(test, feature = "skills-add"))]
mod tests {
    use super::*;
    use crate::core::config::common::PackageRunner;

    fn make_global(runner: Option<PackageRunner>) -> GlobalConfig {
        GlobalConfig {
            schema: None,
            features: std::collections::HashSet::new(),
            targets: None,
            providers: None,
            variables: None,
            package_runner: runner,
        }
    }

    fn make_local(runner: Option<PackageRunner>) -> LocalConfig {
        LocalConfig {
            schema: None,
            features: None,
            targets: None,
            providers: None,
            variables: None,
            package_runner: runner,
        }
    }

    #[test]
    fn local_runner_wins_over_global() {
        let global = make_global(Some(PackageRunner::Npm));
        let local = make_local(Some(PackageRunner::Pnpm));
        let app = AppConfig::from((&global, &local));
        assert_eq!(app.package_runner, Some(PackageRunner::Pnpm));
    }

    #[test]
    fn global_runner_used_when_local_absent() {
        let global = make_global(Some(PackageRunner::Yarn));
        let local = make_local(None);
        let app = AppConfig::from((&global, &local));
        assert_eq!(app.package_runner, Some(PackageRunner::Yarn));
    }

    #[test]
    fn both_absent_yields_none() {
        let global = make_global(None);
        let local = make_local(None);
        let app = AppConfig::from((&global, &local));
        assert_eq!(app.package_runner, None);
    }

    fn make_global_with_targets(targets: Option<HashSet<String>>) -> GlobalConfig {
        GlobalConfig {
            schema: None,
            features: HashSet::new(),
            targets,
            providers: None,
            variables: None,
            package_runner: None,
        }
    }

    fn make_local_with_targets(targets: Option<HashSet<String>>) -> LocalConfig {
        LocalConfig {
            schema: None,
            features: None,
            targets,
            providers: None,
            variables: None,
            package_runner: None,
        }
    }

    #[test]
    fn local_targets_win_over_global() {
        let global = make_global_with_targets(Some(["codex".into()].into_iter().collect()));
        let local = make_local_with_targets(Some(["claude".into()].into_iter().collect()));
        let app = AppConfig::from((&global, &local));
        assert_eq!(app.targets, ["claude".to_string()].into_iter().collect());
    }

    #[test]
    fn global_targets_used_when_local_absent() {
        let global = make_global_with_targets(Some(["codex".into()].into_iter().collect()));
        let local = make_local_with_targets(None);
        let app = AppConfig::from((&global, &local));
        assert_eq!(app.targets, ["codex".to_string()].into_iter().collect());
    }

    #[test]
    fn both_targets_absent_yields_empty() {
        let global = make_global_with_targets(None);
        let local = make_local_with_targets(None);
        let app = AppConfig::from((&global, &local));
        assert!(app.targets.is_empty());
    }
}
