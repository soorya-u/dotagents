use crate::{
    constants::resources,
    schema::{
        common::Target,
        config::{ApplicationConfig, ConfigAgentAbilitySettings, Provider, Targets},
    },
};

#[derive(Clone)]
pub struct ApplicationConfigBuilder {
    pub schema: String,
    pub features: Vec<String>,
    pub targets: Option<Targets>,
    pub providers: Option<Provider>,
}

impl ApplicationConfigBuilder {
    pub fn new() -> Self {
        Self {
            schema: resources::CONFIG_SCHEMA.into(),
            features: vec![],
            targets: None,
            providers: None,
        }
    }

    pub fn add_features(mut self, is_commands: bool, is_instructions: bool, is_mcp: bool) -> Self {
        if is_commands {
            self.features.push(resources::COMMANDS_FEATURE.into());
        }
        if is_instructions {
            self.features.push(resources::INSTRUCTION_FEATURE.into());
        }
        if is_mcp {
            self.features.push(resources::MCP_FEATURE.into());
        }
        self
    }

    pub fn add_targets(mut self, ide: Vec<String>, cli: Vec<String>, custom: Vec<String>) -> Self {
        self.targets = Some(Targets { ide, cli, custom });
        self
    }

    pub fn add_target(mut self, target_name: Target, targets: Vec<String>) -> Self {
        match target_name {
            Target::CLI => self.targets.as_mut().unwrap().cli.extend(targets),
            Target::IDE => self.targets.as_mut().unwrap().ide.extend(targets),
            Target::Custom => self.targets.as_mut().unwrap().custom.extend(targets),
        };
        self
    }

    pub fn add_provider(
        mut self,
        target_name: Target,
        provider_name: &str,
        providers: ConfigAgentAbilitySettings,
    ) -> Self {
        let provider = self.providers.get_or_insert_with(Provider::default);

        match target_name {
            Target::CLI => {
                if let Some(ref mut cli) = provider.cli {
                    cli.insert(provider_name.into(), providers);
                }
            }
            Target::IDE => {
                if let Some(ref mut ide) = provider.ide {
                    ide.insert(provider_name.into(), providers);
                }
            }
            Target::Custom => {
                if let Some(ref mut custom) = provider.custom {
                    custom.insert(provider_name.into(), providers);
                }
            }
        };
        self
    }

    pub fn build(self) -> ApplicationConfig {
        ApplicationConfig {
            schema: self.schema,
            features: self.features,
            targets: self.targets.unwrap_or_default(),
            providers: self.providers,
        }
    }
}
