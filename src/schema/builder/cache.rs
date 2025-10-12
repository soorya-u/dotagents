use std::collections::HashMap;

use crate::schema::cache::{CacheConfig, ProviderSettings};
use crate::schema::common::Target;

pub(crate) struct CacheConfigBuilder {
    pub ide: Option<HashMap<String, ProviderSettings>>,
    pub cli: Option<HashMap<String, ProviderSettings>>,
    pub custom: Option<HashMap<String, ProviderSettings>>,
}

impl CacheConfigBuilder {
    pub fn new() -> Self {
        Self {
            ide: Some(HashMap::new()),
            cli: Some(HashMap::new()),
            custom: Some(HashMap::new()),
        }
    }

    pub fn add_provider(
        mut self,
        provider_type: Target,
        provider_name: &str,
        provider_settings: ProviderSettings,
    ) -> Self {
        match provider_type {
            Target::CLI => self
                .cli
                .as_mut()
                .unwrap()
                .insert(provider_name.into(), provider_settings),
            Target::IDE => self
                .ide
                .as_mut()
                .unwrap()
                .insert(provider_name.into(), provider_settings),
            Target::Custom => self
                .custom
                .as_mut()
                .unwrap()
                .insert(provider_name.into(), provider_settings),
        };

        self
    }

    pub fn build(self) -> CacheConfig {
        CacheConfig {
            ide: self.ide,
            cli: self.cli,
            custom: self.custom,
            ..Default::default()
        }
    }
}
