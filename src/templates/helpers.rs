use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde_json::{Value, json};
use std::sync::OnceLock;

use crate::utils::path::{get_application_dir, get_config_dir, get_workspace_dir};
use crate::{
    constants::{
        file::{GLOBAL_CONFIG_FILE, LOCAL_CONFIG_FILE},
        variables::{APPLICATION_DIR, CONFIG_DIR, WORKSPACE_DIR},
    },
    utils::merge_json,
};

static TEMPLATER: OnceLock<Templater> = OnceLock::new();

pub fn get_templater() -> &'static Templater {
    TEMPLATER.get_or_init(|| Templater::new().expect("failed to create templater"))
}

pub enum TemplateSource {
    File(String),
    Text(String),
}

pub struct Templater {
    handlebar: Handlebars<'static>,
    globals: Value,
}

impl Templater {
    fn load_default_variables() -> Result<Value> {
        let config_dir = get_config_dir()?.to_string_lossy().to_string();
        let workspace_dir = get_workspace_dir()?.to_string_lossy().to_string();
        let application_dir = get_application_dir()?.to_string_lossy().to_string();

        Ok(json!({
            CONFIG_DIR: &config_dir,
            WORKSPACE_DIR: &workspace_dir,
            APPLICATION_DIR: &application_dir,
        }))
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
        let globals = Self::load_default_variables().expect("failed to load global variables");
        let mut templater = Self {
            handlebar: Handlebars::new(),
            globals,
        };
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

    pub fn render_template(&self, name: &str, data: Option<&Value>) -> Result<String> {
        let data = match data {
            Some(data) => &merge_json(data, &self.globals),
            None => &self.globals,
        };

        self.handlebar
            .render(name, data)
            .context("failed to render template")
    }
}
