use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde_json::Value;
use std::sync::OnceLock;

use crate::templates::variables::get_env_variables;
use crate::{
    constants::file::{GLOBAL_CONFIG_FILE, LOCAL_CONFIG_FILE},
    constants::helpers::{
        IF_DEFINED_HELPER, IF_EQ_HELPER, JSON_HELPER, SNAKE_CASE_HELPER, TOML_HELPER,
        TOML_INLINE_HELPER, YAML_HELPER,
    },
    templates::{
        helpers::{
            IfDefinedHelper, IfEqHelper, JsonHelper, SnakeCaseHelper, TomlHelper, TomlInlineHelper,
            YamlHelper,
        },
        variables::{get_command_name_variable, get_dir_variables, get_skill_name_variable},
    },
    utils::json::{merge_json, merge_many_json},
    utils::path::get_application_dir,
};

static TEMPLATER: OnceLock<Templater> = OnceLock::new();

pub fn get_templater() -> Result<&'static Templater> {
    if let Some(t) = TEMPLATER.get() {
        return Ok(t);
    }
    let templater = Templater::new()?;
    let _ = TEMPLATER.set(templater);
    TEMPLATER
        .get()
        .ok_or_else(|| anyhow::anyhow!("templater unexpectedly not initialised"))
}

pub enum TemplateSource {
    File(String),
    Text(String),
}

pub enum RenderType {
    Name(String),
    Content(String),
}

pub struct Templater {
    handlebar: Handlebars<'static>,
    globals: Value,
}

impl Templater {
    fn load_default_variables() -> Result<Value> {
        let dir_variables = get_dir_variables()?;
        let env_variables = get_env_variables()?;
        let command_variables = get_command_name_variable("{{ command.name }}")?;
        let skill_variables = get_skill_name_variable("{{ skill.name }}")?;
        Ok(merge_many_json(&[
            dir_variables,
            env_variables,
            command_variables,
            skill_variables,
        ]))
    }

    fn register_default_templates(&mut self) -> Result<()> {
        let application_dir = get_application_dir()?;
        let global_config_file = application_dir
            .join(GLOBAL_CONFIG_FILE)
            .to_string_lossy()
            .to_string();
        let local_config_path = application_dir.join(LOCAL_CONFIG_FILE);

        self.register_template(GLOBAL_CONFIG_FILE, TemplateSource::File(global_config_file))?;

        if local_config_path.exists() {
            self.register_template(
                LOCAL_CONFIG_FILE,
                TemplateSource::File(local_config_path.to_string_lossy().to_string()),
            )?;
        } else {
            self.register_template(LOCAL_CONFIG_FILE, TemplateSource::Text(String::new()))?;
        }

        Ok(())
    }

    pub fn new() -> Result<Self> {
        let globals = Self::load_default_variables()?;
        let mut handlebar = Handlebars::new();
        handlebar.register_helper(IF_EQ_HELPER, Box::new(IfEqHelper));
        handlebar.register_helper(IF_DEFINED_HELPER, Box::new(IfDefinedHelper));
        handlebar.register_helper(JSON_HELPER, Box::new(JsonHelper));
        handlebar.register_helper(TOML_HELPER, Box::new(TomlHelper));
        handlebar.register_helper(TOML_INLINE_HELPER, Box::new(TomlInlineHelper));
        handlebar.register_helper(YAML_HELPER, Box::new(YamlHelper));
        handlebar.register_helper(SNAKE_CASE_HELPER, Box::new(SnakeCaseHelper));
        let mut templater = Self { handlebar, globals };
        templater.register_default_templates()?;
        Ok(templater)
    }

    pub fn register_template(&mut self, name: &str, source: TemplateSource) -> Result<()> {
        match source {
            TemplateSource::File(path) => self.handlebar.register_template_file(name, path),
            TemplateSource::Text(str) => self.handlebar.register_template_string(name, str),
        }
        .context("failed to register template. check for syntax errors")
    }

    pub fn render_template(&self, name: RenderType, data: Option<&Value>) -> Result<String> {
        let data = merge_json(Some(&self.globals), data);

        match name {
            RenderType::Name(path) => self.handlebar.render(&path, &data),
            RenderType::Content(str) => self.handlebar.render_template(&str, &data),
        }
        .map_err(|e| anyhow::anyhow!("{}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    use crate::constants::dir::ROOT_DIR;
    use crate::constants::file::{GLOBAL_CONFIG_FILE, LOCAL_CONFIG_FILE};
    use crate::constants::mocks::default_config;
    use crate::utils::path::override_workspace_dir;

    fn setup_test_workspace() -> Result<TempDir> {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join(ROOT_DIR);
        fs::create_dir_all(&root)?;
        fs::write(
            root.join(GLOBAL_CONFIG_FILE),
            default_config(&["commands", "instructions", "mcp", "skills"], &["claude"]),
        )?;
        fs::write(root.join(LOCAL_CONFIG_FILE), "")?;
        override_workspace_dir(tmp.path().to_path_buf())?;
        Ok(tmp)
    }

    // render_template with a broken template does not emit "failed to render template"
    #[test]
    fn render_template_error_does_not_contain_generic_message() {
        let Ok(_tmp) = setup_test_workspace() else {
            return; // WORKSPACE_DIR OnceLock already set by a prior test; skip
        };
        let templater = Templater::new().unwrap();
        let result = templater.render_template(RenderType::Content("{{".to_string()), None);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            !err.contains("failed to render template"),
            "error chain should not contain generic 'failed to render template', got: {err}"
        );
    }
}
