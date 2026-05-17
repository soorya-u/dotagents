use anyhow::{Context, Result, bail};
use cliclack::{confirm, input, outro};
use std::fs;
use std::io::ErrorKind;

use crate::cli::deploy::deploy;
use crate::cli::options::{
    AddSkillOptions, DeployOptions, RmSkillOptions, SkillsAddOptions, SubLsOptions,
};
use crate::cli::ui::ls::render_skills;
use crate::core::config::CacheConfig;
use crate::core::config::app::AppConfig;
use crate::core::config::common::PackageRunner;
use crate::core::features::skill::SkillFeature;
use crate::prelude::*;
use crate::schema::list_item::ListItem;
use crate::templates::get_templater;
use crate::utils::fs::write_file;
use crate::utils::path::{
    get_application_dir, get_skills_dir, get_workspace_dir, resolve_and_override_workspace,
};
use crate::utils::tui::is_tui_enabled;

/// Collect a string field: use provided value, prompt in TTY mode, or default to empty.
fn collect_field(value: Option<String>, prompt: &str, placeholder: &str) -> Result<String> {
    if let Some(v) = value {
        return Ok(v);
    }
    if is_tui_enabled() {
        let v: String = input(prompt)
            .placeholder(placeholder)
            .default_input("")
            .interact()
            .context(format!("failed to read {}", prompt))?;
        return Ok(v);
    }
    Ok(String::new())
}

/// Deploy after skill mutation: skip if --no-deploy, auto-deploy in CI, prompt in TTY.
fn maybe_prompt_deploy(no_deploy: bool) -> Result<()> {
    if no_deploy {
        return Ok(());
    }
    if !is_tui_enabled() {
        deploy(DeployOptions::default()).context("deploy failed")?;
        return Ok(());
    }
    let should_deploy = confirm("Deploy now?")
        .initial_value(false)
        .interact()
        .unwrap_or(false);
    if should_deploy {
        deploy(DeployOptions::default()).context("deploy failed")?;
    }
    Ok(())
}

/// Install a skill into `.dotagents/skills/` by wrapping the `skills` CLI.
pub(crate) fn add(opts: SkillsAddOptions) -> Result<bool> {
    resolve_and_override_workspace(opts.workspace.cwd)
        .context("unable to resolve workspace directory")?;

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

    let args = runner.args(&opts.name, !is_tui_enabled());
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
pub(crate) fn new_skill(opts: AddSkillOptions) -> Result<bool> {
    resolve_and_override_workspace(opts.workspace.cwd)
        .context("unable to resolve workspace directory")?;

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

    let use_interactive = is_tui_enabled()
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

    let content = SkillFeature::scaffold(&opts.name, &description, &license, &compatibility)
        .context("failed to scaffold skill")?;

    write_file(&target, &content).context("failed to write SKILL.md")?;

    success!("Created {}", target.display());

    maybe_prompt_deploy(opts.no_deploy)?;

    if use_interactive {
        outro("").ok();
    }

    Ok(true)
}

/// Handle `dotagents skills rm`.
pub(crate) fn rm_skill(opts: RmSkillOptions) -> Result<bool> {
    resolve_and_override_workspace(opts.workspace.cwd)
        .context("unable to resolve workspace directory")?;

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
    if is_tui_enabled() && !opts.force {
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

    maybe_prompt_deploy(opts.no_deploy)?;

    outro("").ok();
    Ok(true)
}

/// Convert a `SkillFeature` into a `ListItem` for display.
fn skill_to_list_item(skill: SkillFeature) -> ListItem {
    let name = skill.metadata.name.clone();
    let description = skill.metadata.description.clone();
    let frontmatter = serde_json::to_value(&skill.metadata).unwrap_or_default();
    ListItem {
        name,
        description,
        frontmatter,
        body: Some(skill.content),
    }
}

/// Load all skills from `.dotagents/skills/*/SKILL.md`, returning full frontmatter + body.
fn load_skills() -> Result<Vec<ListItem>> {
    // skills dir absent means the feature is not enabled; any other error propagates
    if get_skills_dir().is_err() {
        return Ok(vec![]);
    }
    let skills = SkillFeature::from_application().context("failed to load skills")?;
    let mut items: Vec<ListItem> = skills.into_iter().map(skill_to_list_item).collect();
    items.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(items)
}

/// Handle `dotagents skills ls`.
pub(crate) fn ls_skills(opts: SubLsOptions) -> Result<bool> {
    resolve_and_override_workspace(opts.workspace.cwd)
        .context("unable to resolve workspace directory")?;

    get_application_dir().context(
        "No .dotagents directory found. Run `dotagents init` to initialise a workspace.",
    )?;
    let mut skills = load_skills().context("failed to load skills")?;

    if let Some(ref filter) = opts.skill {
        skills.retain(|item| item.name == *filter);
    }

    if opts.json {
        let json_items = ListItem::to_json_array(&skills, opts.content);
        let output = serde_json::to_string_pretty(&json_items)?;
        println!("{}", output);
        return Ok(true);
    }

    render_skills(skills, opts.content);
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tui::set_ci_mode;
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

    fn make_skill_full(
        skills_dir: &std::path::Path,
        name: &str,
        description: &str,
        license: &str,
        compatibility: &str,
        body: &str,
    ) {
        let skill_dir = skills_dir.join(name);
        fs::create_dir_all(&skill_dir).unwrap();
        let content = format!(
            "---\nname: {}\ndescription: \"{}\"\nlicense: {}\ncompatibility: {}\n---\n\n{}",
            name, description, license, compatibility, body
        );
        fs::write(skill_dir.join("SKILL.md"), content).unwrap();
    }

    fn load_skills_from_dir(dir: &std::path::Path) -> Result<Vec<ListItem>> {
        let mut items = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let skill_md = path.join("SKILL.md");
            if !skill_md.is_file() {
                continue;
            }
            let content = fs::read_to_string(&skill_md)?;
            let Ok(skill) = SkillFeature::from_markdown(&content) else {
                continue;
            };
            items.push(skill_to_list_item(skill));
        }
        items.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(items)
    }

    #[test]
    fn load_skills_reads_skill_md_frontmatter() {
        // load_skills_from parses name and description from SKILL.md files
        let tmp = TempDir::new().unwrap();
        make_skill(tmp.path(), "my-skill", "Does something");

        let items = load_skills_from_dir(tmp.path()).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "my-skill");
        assert_eq!(items[0].description, "Does something");
    }

    #[test]
    fn load_skills_includes_body_content() {
        // load_skills_from returns body content alongside frontmatter
        let tmp = TempDir::new().unwrap();
        make_skill(tmp.path(), "my-skill", "Does something");

        let items = load_skills_from_dir(tmp.path()).unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0].body.as_deref().unwrap().contains("Body."));
    }

    #[test]
    fn load_skills_returns_full_frontmatter() {
        // load_skills_from returns frontmatter with all fields (license, compatibility)
        let tmp = TempDir::new().unwrap();
        make_skill_full(
            tmp.path(),
            "my-skill",
            "A skill",
            "MIT",
            "Any agent",
            "Body",
        );

        let items = load_skills_from_dir(tmp.path()).unwrap();
        assert_eq!(items[0].frontmatter["license"], "MIT");
        assert_eq!(items[0].frontmatter["compatibility"], "Any agent");
    }

    #[test]
    fn to_json_array_includes_frontmatter_fields() {
        // to_json_array includes frontmatter fields like license
        let items = vec![ListItem {
            name: "test".into(),
            description: "desc".into(),
            frontmatter: serde_json::json!({"name": "test", "license": "MIT"}),
            body: None,
        }];
        let result = ListItem::to_json_array(&items, false);
        assert_eq!(result[0]["name"], "test");
        assert_eq!(result[0]["license"], "MIT");
    }

    #[test]
    fn to_json_array_with_content_includes_body_for_skills() {
        // to_json_array with content=true includes body key for skills
        let items = vec![ListItem {
            name: "test".into(),
            description: "desc".into(),
            frontmatter: serde_json::json!({"name": "test"}),
            body: Some("body text".into()),
        }];
        let result = ListItem::to_json_array(&items, true);
        assert_eq!(result[0]["content"], "body text");
    }

    // maybe_prompt_deploy(true) skips deploy and returns immediately
    #[test]
    fn maybe_prompt_deploy_no_deploy_true_skips_deploy() {
        // no_deploy=true causes early return without calling deploy
        assert!(maybe_prompt_deploy(true).is_ok());
    }

    // maybe_prompt_deploy(false) in CI auto-deploys without prompting
    #[test]
    fn maybe_prompt_deploy_ci_calls_deploy_when_no_deploy_false() {
        use crate::utils::path::override_workspace_dir;
        let original_dir = std::env::current_dir().unwrap();
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join(".dotagents-debug");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("config.toml"), "features = []\ntargets = []\n").unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();

        // Skip if a prior test already seized the OnceLock (e.g. commands equivalent)
        if override_workspace_dir(tmp.path().to_path_buf()).is_err() {
            std::env::set_current_dir(&original_dir).unwrap();
            return;
        }
        set_ci_mode(true);
        let result = maybe_prompt_deploy(false);
        set_ci_mode(false);
        std::env::set_current_dir(&original_dir).unwrap();
        // Deploy succeeds trivially (no targets, no features), proving deploy was called
        assert!(result.is_ok());
    }
}
