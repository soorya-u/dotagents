use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde_json::{Value, json};
use std::env;
use std::sync::OnceLock;

use crate::templates::variables::get_env_variables;
use crate::utils::path::{get_application_dir, get_config_dir, get_workspace_dir};
use crate::{
    constants::helpers::JSON_HELPER,
    templates::{
        helpers::{IfEqHelper, JsonHelper},
        variables::{get_command_name_variable, get_dir_variables, get_skill_name_variable},
    },
};
use crate::{
    constants::{
        file::{GLOBAL_CONFIG_FILE, LOCAL_CONFIG_FILE},
        helpers::IF_EQ_HELPER,
    },
    utils::{merge_json, merge_many_json},
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
        let local_config_file = application_dir
            .join(LOCAL_CONFIG_FILE)
            .to_string_lossy()
            .to_string();

        self.register_template(GLOBAL_CONFIG_FILE, TemplateSource::File(global_config_file))?;
        self.register_template(LOCAL_CONFIG_FILE, TemplateSource::File(local_config_file))?;

        Ok(())
    }

    pub fn new() -> Result<Self> {
        let globals = Self::load_default_variables()?;
        let mut handlebar = Handlebars::new();
        handlebar.register_helper(IF_EQ_HELPER, Box::new(IfEqHelper));
        handlebar.register_helper(JSON_HELPER, Box::new(JsonHelper));
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
        .context("failed to render template")
    }
}
