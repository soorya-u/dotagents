#[cfg(debug_assertions)]
pub(crate) const ROOT_DIR: &str = ".dotagents-debug";
#[cfg(not(debug_assertions))]
pub(crate) const ROOT_DIR: &str = ".dotagents";

pub(crate) const COMMANDS_DIR: &str = "commands";
pub(crate) const CACHE_DIR: &str = "cache";
pub(crate) const TEMPLATE_DIR: &str = "templates";

pub(crate) const INSTRUCTIONS_FILE: &str = "INSTRUCTIONS.md";
pub(crate) const MCP_FILE: &str = "mcp.jsonc";
pub(crate) const GLOBAL_CONFIG_FILE: &str = "config.toml";
pub(crate) const LOCAL_CONFIG_FILE: &str = "local.config.toml";

pub(crate) const CONFIG_SCHEMA: &str = "https://dotagents.soorya-u.dev/schemas/config.schema.json";
pub(crate) const MCP_SCHEMA: &str = "https://dotagents.soorya-u.dev/json/schemas/mcp.schema.json";

pub(crate) const COMMANDS_FEATURE: &str = "commands";
pub(crate) const MCP_FEATURE: &str = "mcp";
pub(crate) const INSTRUCTION_FEATURE: &str = "instructions";
