use std::collections::HashMap;

use crate::{
    constants::resources,
    schema::mcp::{CommonConfig, McpConfig, ServerConfig},
};

pub(crate) struct McpConfigBuilder {
    schema: String,
    servers: HashMap<String, ServerConfig>,
}

impl McpConfigBuilder {
    pub fn new() -> Self {
        Self {
            schema: resources::MCP_SCHEMA.into(),
            servers: HashMap::new(),
        }
    }

    pub fn add_http_server(
        mut self,
        name: &str,
        url: &str,
        headers: Option<HashMap<String, String>>,
        common: Option<CommonConfig>,
    ) -> Self {
        self.servers.insert(
            name.to_string(),
            ServerConfig::Http {
                url: url.into(),
                headers,
                common,
            },
        );
        self
    }

    pub fn add_stdio_server(
        mut self,
        name: &str,
        command: &str,
        args: Vec<String>,
        cwd: Option<&str>,
        common: Option<CommonConfig>,
    ) -> Self {
        self.servers.insert(
            name.to_string(),
            ServerConfig::Stdio {
                command: command.into(),
                args,
                cwd: cwd.map(|s| s.into()),
                env: Some(HashMap::new()),
                env_file: None,
                common,
            },
        );
        self
    }

    pub fn build(self) -> McpConfig {
        McpConfig {
            schema: self.schema,
            servers: self.servers,
        }
    }
}
