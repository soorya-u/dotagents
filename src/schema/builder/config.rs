use std::collections::HashMap;

use crate::{
    constants::schema::CONFIG_SCHEMA,
    schema::{
        common::Target,
        config::{
            ConfigAgentAbilitySettings, ConfigAgentSettings, GlobalConfig, LocalConfig, Providers,
            Targets,
        },
    },
};

#[derive(Clone)]
pub(crate) struct ApplicationConfigBuilder {
    schema: Option<String>,
    features: Option<Vec<String>>,
    targets: Option<Targets>,
    providers: Option<Providers>,
}

impl ApplicationConfigBuilder {
    pub fn new() -> Self {
        Self {
            schema: Some(CONFIG_SCHEMA.to_string()),
            features: None,
            targets: None,
            providers: None,
        }
    }

    pub fn add_features(mut self, commands: bool, instructions: bool, mcp: bool) -> Self {
        let mut features = Vec::new();
        if commands {
            features.push("commands".to_string());
        }
        if instructions {
            features.push("instructions".to_string());
        }
        if mcp {
            features.push("mcp".to_string());
        }
        self.features = Some(features);
        self
    }

    pub fn add_targets(mut self, cli: Vec<String>, ide: Vec<String>, custom: Vec<String>) -> Self {
        self.targets = Some(Targets {
            ide: if ide.is_empty() { None } else { Some(ide) },
            cli: if cli.is_empty() { None } else { Some(cli) },
            custom: if custom.is_empty() {
                None
            } else {
                Some(custom)
            },
        });
        self
    }

    pub fn add_target(mut self, target_type: Target, names: Vec<String>) -> Self {
        if self.targets.is_none() {
            self.targets = Some(Targets::new());
        }

        if let Some(ref mut targets) = self.targets {
            match target_type {
                Target::IDE => {
                    targets.ide = Some(names);
                }
                Target::CLI => {
                    targets.cli = Some(names);
                }
                Target::Custom => {
                    targets.custom = Some(names);
                }
            }
        }

        self
    }

    pub fn add_provider(
        mut self,
        target_type: Target,
        name: &str,
        settings: ConfigAgentAbilitySettings,
    ) -> Self {
        if self.providers.is_none() {
            self.providers = Some(Providers::new());
        }

        if let Some(ref mut providers) = self.providers {
            let provider_map = match target_type {
                Target::IDE => {
                    if providers.ide.is_none() {
                        providers.ide = Some(HashMap::new());
                    }
                    providers.ide.as_mut().unwrap()
                }
                Target::CLI => {
                    if providers.cli.is_none() {
                        providers.cli = Some(HashMap::new());
                    }
                    providers.cli.as_mut().unwrap()
                }
                Target::Custom => {
                    if providers.custom.is_none() {
                        providers.custom = Some(HashMap::new());
                    }
                    providers.custom.as_mut().unwrap()
                }
            };

            provider_map.insert(name.to_string(), settings);
        }

        self
    }

    pub fn build(self) -> GlobalConfig {
        GlobalConfig {
            schema: self.schema,
            features: self.features.unwrap_or_default(),
            targets: self.targets,
            providers: self.providers,
        }
    }

    pub fn build_local(self) -> LocalConfig {
        LocalConfig {
            schema: self.schema,
            features: self.features,
            targets: self.targets,
            providers: self.providers,
        }
    }
}

impl Default for ApplicationConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) struct ConfigAgentAbilitySettingsBuilder {
    mcp: Option<ConfigAgentSettings>,
    instructions: Option<ConfigAgentSettings>,
    commands: Option<ConfigAgentSettings>,
}

impl ConfigAgentAbilitySettingsBuilder {
    pub fn new() -> Self {
        Self {
            mcp: None,
            instructions: None,
            commands: None,
        }
    }

    pub fn mcp(mut self, settings: ConfigAgentSettings) -> Self {
        self.mcp = Some(settings);
        self
    }

    pub fn instructions(mut self, settings: ConfigAgentSettings) -> Self {
        self.instructions = Some(settings);
        self
    }

    pub fn commands(mut self, settings: ConfigAgentSettings) -> Self {
        self.commands = Some(settings);
        self
    }

    pub fn build(self) -> ConfigAgentAbilitySettings {
        ConfigAgentAbilitySettings {
            mcp: self.mcp,
            instructions: self.instructions,
            commands: self.commands,
        }
    }
}

impl Default for ConfigAgentAbilitySettingsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) struct ConfigAgentSettingsBuilder {
    template: Option<String>,
    target: Option<String>,
    disabled: Option<bool>,
    variables: Option<HashMap<String, String>>,
    hash: Option<String>,
}

impl ConfigAgentSettingsBuilder {
    pub fn new() -> Self {
        Self {
            template: None,
            target: None,
            disabled: None,
            variables: None,
            hash: None,
        }
    }

    pub fn template(mut self, template: &str) -> Self {
        self.template = Some(template.to_string());
        self
    }

    pub fn target(mut self, target: &str) -> Self {
        self.target = Some(target.to_string());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }

    pub fn variables(mut self, variables: HashMap<String, String>) -> Self {
        self.variables = Some(variables);
        self
    }

    pub fn hash(mut self, hash: &str) -> Self {
        self.hash = Some(hash.to_string());
        self
    }

    pub fn build(self) -> ConfigAgentSettings {
        ConfigAgentSettings {
            template: self.template,
            target: self.target,
            disabled: self.disabled,
            variables: self.variables,
            hash: self.hash,
        }
    }
}

impl Default for ConfigAgentSettingsBuilder {
    fn default() -> Self {
        Self::new()
    }
}
