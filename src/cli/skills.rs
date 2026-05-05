use anyhow::{Context, Result, bail};
use cliclack::{confirm, input, outro};
use gray_matter::Matter;
use gray_matter::engine::YAML;
use serde_json::Value;
use serde_yaml::Value as YamlValue;
use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;

use crate::cli::deploy::deploy;
use crate::cli::options::{
    AddSkillOptions, DeployOptions, RmSkillOptions, SkillsAction, SkillsAddOptions, SubLsOptions,
};
use crate::cli::ui::ls::{ListItem, render_skills};
use crate::constants::templates::{SKILL_STARTER, render_starter};
use crate::prelude::*;
use crate::schema::config::CacheConfig;
use crate::schema::config::app::AppConfig;
use crate::schema::config::common::PackageRunner;
use crate::templates::get_templater;
use crate::utils::fs::write_file;
use crate::utils::path::{get_application_dir, get_skills_dir, get_workspace_dir};
use crate::utils::tty::is_tty;

/// Collect a string field: use provided value, prompt in TTY mode, or default to empty.
fn collect_field(value: Option<String>, prompt: &str, placeholder: &str) -> Result<String> {
    if let Some(v) = value {
        return Ok(v);
    }
    if is_tty() {
        let v: String = input(prompt)
            .placeholder(placeholder)
            .default_input("")
            .interact()
            .context(format!("failed to read {}", prompt))?;
        return Ok(v);
    }
    Ok(String::new())
}

/// Prompt the user to deploy now (TTY only, used when --deploy was not passed).
fn maybe_prompt_deploy(deploy_flag: bool) -> Result<()> {
    if deploy_flag {
        deploy(DeployOptions::default()).context("deploy failed")?;
        return Ok(());
    }
    if is_tty() {
        let should_deploy = confirm("Deploy now?")
            .initial_value(false)
            .interact()
            .unwrap_or(false);
        if should_deploy {
            deploy(DeployOptions::default()).context("deploy failed")?;
        }
    }
    Ok(())
}

/// Install a skill into `.dotagents/skills/` by wrapping the `skills` CLI.
fn add(opts: SkillsAddOptions) -> Result<bool> {
    let templater = get_templater()?;
    let app_config = AppConfig::from_application(templater)?;

    // Resolve runner: CLI flag > config > silent default (npm)
    let explicit_runner = opts.runner.or(app_config.package_runner);

    let runner = match &explicit_runner {
        Some(r) => {
            // Explicitly configured — validate binary is on PATH
            let binary = r.binary();
            let probe = std::process::Command::new(binary)
                .arg("--version")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .output();

            match probe {
                Err(e) if e.kind() == ErrorKind::NotFound => {
                    bail!(
                        "`{}` was not found on PATH.\n\
                         Check the `package-runner` setting in your config.toml.",
                        binary
                    );
                }
                Err(e) => {
                    bail!("Failed to probe `{}`: {}", binary, e);
                }
                Ok(_) => r,
            }
        }
        None => {
            // No runner configured — use npm silently, let OS surface any error
            &PackageRunner::Npm
        }
    };

    let application_dir = get_application_dir()?;
    let claude_config_dir = application_dir
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("application dir path is not valid UTF-8"))?;

    let args = runner.args(&opts.name);
    let (program, rest) = args
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("runner produced empty args list"))?;

    let status = std::process::Command::new(program)
        .args(rest)
        .env("CLAUDE_CONFIG_DIR", claude_config_dir)
        .status()?;

    Ok(status.success())
}

/// Handle `dotagents skills new`.
fn new_skill(opts: AddSkillOptions) -> Result<bool> {
    let app_dir = get_application_dir().context(
        "No .dotagents directory found. Run `dotagents init` to initialise a workspace.",
    )?;

    let skill_dir = app_dir.join("skills").join(&opts.name);
    let target = skill_dir.join("SKILL.md");

    if target.exists() && !opts.force {
        bail!(
            "Skill '{}' already exists at {}. Use --force to overwrite.",
            opts.name,
            target.display()
        );
    }

    let use_interactive = is_tty()
        && opts.description.is_none()
        && opts.license.is_none()
        && opts.compatibility.is_none();

    let description = collect_field(opts.description, "Description", "What does this skill do?")?;
    let license = collect_field(opts.license, "License", "e.g. MIT")?;
    let compatibility = collect_field(
        opts.compatibility,
        "Compatibility",
        "e.g. Requires openspec CLI.",
    )?;

    // Build frontmatter via serde_yaml to properly escape all values.
    let mut metadata: BTreeMap<&str, YamlValue> = BTreeMap::new();
    metadata.insert("version", YamlValue::String("1.0".to_string()));

    let mut fm: BTreeMap<&str, YamlValue> = BTreeMap::new();
    fm.insert("name", YamlValue::String(opts.name.clone()));
    fm.insert("description", YamlValue::String(description));
    fm.insert("license", YamlValue::String(license));
    fm.insert("compatibility", YamlValue::String(compatibility));
    fm.insert(
        "metadata",
        YamlValue::Mapping(
            metadata
                .into_iter()
                .map(|(k, v)| (YamlValue::String(k.to_string()), v))
                .collect(),
        ),
    );
    let frontmatter = serde_yaml::to_string(&fm).context("failed to serialize frontmatter")?;

    let body = render_starter(SKILL_STARTER, &opts.name);
    let content = format!("---\n{}---\n\n{}", frontmatter, body);

    write_file(&target, &content).context("failed to write SKILL.md")?;

    success!("Created {}", target.display());

    maybe_prompt_deploy(opts.deploy)?;

    if use_interactive {
        outro("").ok();
    }

    Ok(true)
}

/// Handle `dotagents skills rm`.
fn rm_skill(opts: RmSkillOptions) -> Result<bool> {
    let app_dir = get_application_dir().context(
        "No .dotagents directory found. Run `dotagents init` to initialise a workspace.",
    )?;

    let skill_dir = app_dir.join("skills").join(&opts.name);

    if !skill_dir.exists() {
        bail!(
            "Skill '{}' not found at {}.",
            opts.name,
            skill_dir.display()
        );
    }

    // Confirm in TTY unless --force.
    if is_tty() && !opts.force {
        let confirmed = confirm(format!(
            "Remove skill '{}'? This cannot be undone.",
            opts.name
        ))
        .initial_value(false)
        .interact()
        .unwrap_or(false);

        if !confirmed {
            outro("Cancelled.").ok();
            return Ok(true);
        }
    }

    // Load cache before removing the source — cleanup runs unconditionally after removal.
    let cache_opt: Option<CacheConfig> = match CacheConfig::load() {
        Ok(c) => Some(c),
        Err(e) => {
            warn!(
                "Failed to load cache, deployed files will not be cleaned up: {}",
                e
            );
            None
        }
    };

    fs::remove_dir_all(&skill_dir)
        .with_context(|| format!("failed to remove {}", skill_dir.display()))?;

    success!("Removed {}", skill_dir.display());

    // Clean up deployed output across all providers.
    if let Some(mut cache) = cache_opt {
        match get_workspace_dir() {
            Ok(workspace_dir) => {
                if let Err(e) =
                    super::undeploy::undeploy_item("skills", &opts.name, &mut cache, &workspace_dir)
                {
                    warn!("Failed to clean up deployed files: {}", e);
                }
                if let Err(e) = cache.save() {
                    warn!("Failed to save cache after cleanup: {}", e);
                }
            }
            Err(e) => {
                warn!("Failed to get workspace directory for cleanup: {}", e);
            }
        }
    }

    maybe_prompt_deploy(opts.deploy)?;

    outro("").ok();
    Ok(true)
}

/// Load all skills from `.dotagents/skills/*/SKILL.md`, returning name+description pairs.
fn load_skills() -> Result<Vec<ListItem>> {
    let skills_dir = match get_skills_dir() {
        Ok(d) => d,
        Err(_) => return Ok(vec![]), // skills dir absent → empty
    };

    let matter = Matter::<YAML>::new();
    let mut items = Vec::new();

    for entry in fs::read_dir(&skills_dir).context("failed to read skills directory")? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let skill_md = path.join("SKILL.md");
        if !skill_md.is_file() {
            continue;
        }
        let content = fs::read_to_string(&skill_md).context("failed to read SKILL.md")?;
        let Ok(parsed) = matter.parse::<Value>(&content) else {
            continue;
        };
        if let Some(data) = parsed.data {
            let name: String = data["name"].as_str().unwrap_or("").to_string();
            let description: String = data["description"].as_str().unwrap_or("").to_string();
            if !name.is_empty() {
                items.push(ListItem { name, description });
            }
        }
    }

    items.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(items)
}

/// Handle `dotagents skills ls`.
fn ls_skills(opts: SubLsOptions) -> Result<bool> {
    get_application_dir().context(
        "No .dotagents directory found. Run `dotagents init` to initialise a workspace.",
    )?;
    let skills = load_skills().context("failed to load skills")?;
    render_skills(skills, opts.full);
    Ok(true)
}

/// Dispatch `dotagents skills`.
pub(crate) fn run_skills(action: SkillsAction) -> Result<bool> {
    match action {
        SkillsAction::Add(opts) => add(opts),
        SkillsAction::New(opts) => new_skill(opts),
        SkillsAction::Rm(opts) => rm_skill(opts),
        SkillsAction::Ls(opts) => ls_skills(opts),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_skill(skills_dir: &std::path::Path, name: &str, description: &str) {
        let skill_dir = skills_dir.join(name);
        fs::create_dir_all(&skill_dir).unwrap();
        let content = format!(
            "---\nname: {}\ndescription: \"{}\"\n---\n\nBody.\n",
            name, description
        );
        fs::write(skill_dir.join("SKILL.md"), content).unwrap();
    }

    #[test]
    fn load_skills_reads_skill_md_frontmatter() {
        // load_skills parses name and description from SKILL.md files
        let tmp = TempDir::new().unwrap();
        make_skill(tmp.path(), "my-skill", "Does something");

        let matter = Matter::<YAML>::new();
        let skill_md = tmp.path().join("my-skill").join("SKILL.md");
        let content = fs::read_to_string(&skill_md).unwrap();
        let parsed = matter.parse::<Value>(&content).unwrap();
        let data = parsed.data.unwrap();
        assert_eq!(data["name"].as_str().unwrap(), "my-skill");
        assert_eq!(data["description"].as_str().unwrap(), "Does something");
    }
}
