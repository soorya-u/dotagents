#[cfg(debug_assertions)]
pub(crate) const ROOT_DIR: &str = ".dotagents-debug";
#[cfg(not(debug_assertions))]
pub(crate) const ROOT_DIR: &str = ".dotagents";
pub(crate) const COMMANDS_DIR: &str = "commands";
pub(crate) const SKILLS_DIR: &str = "skills";
pub(crate) const CACHE_DIR: &str = "cache";
pub(crate) const TEMPLATE_DIR: &str = "templates";

pub(crate) const MOCK_CUSTOM_AGENT_DIR: &str = "mycode";

/// Subdirectory inside the user-level cache dir that holds downloaded provider template files.
pub(crate) const TEMPLATE_CACHE_SUBDIR: &str = "templates";
