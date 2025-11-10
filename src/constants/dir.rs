#[cfg(debug_assertions)]
pub(crate) const ROOT_DIR: &str = ".dotagents-debug";
#[cfg(not(debug_assertions))]
pub(crate) const ROOT_DIR: &str = ".dotagents";
pub(crate) const COMMANDS_DIR: &str = "commands";
pub(crate) const CACHE_DIR: &str = "cache";
pub(crate) const TEMPLATE_DIR: &str = "templates";
