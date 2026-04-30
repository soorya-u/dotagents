use cliclack::{confirm, input, intro, outro};

use crate::cli::deploy::deploy;
use crate::cli::options::{AddAction, AddCommandOptions, AddSkillOptions, DeployOptions};
use crate::constants::templates::{COMMAND_STARTER, SKILL_STARTER, render_starter};
use crate::prelude::*;
use crate::utils::fs::write_file;
use crate::utils::path::get_application_dir;
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

/// Handle `dotagents add command`.
fn add_command(opts: AddCommandOptions) -> Result<bool> {
    let app_dir = get_application_dir().context(
        "No .dotagents directory found. Run `dotagents init` to initialise a workspace.",
    )?;

    let target = app_dir.join("commands").join(format!("{}.md", opts.name));

    if target.exists() && !opts.force {
        bail!(
            "Command '{}' already exists at {}. Use --force to overwrite.",
            opts.name,
            target.display()
        );
    }

    let use_interactive =
        is_tty() && opts.description.is_none() && opts.category.is_none() && opts.tags.is_none();

    if use_interactive {
        intro("dotagents add command").ok();
    }

    let description = collect_field(
        opts.description,
        "Description",
        "What does this command do?",
    )?;
    let category = collect_field(opts.category, "Category", "e.g. Workflow")?;
    let tags_raw = collect_field(opts.tags, "Tags (comma-separated)", "e.g. workflow,explore")?;

    // Build frontmatter manually to preserve optional fields cleanly.
    let mut frontmatter = format!("name: \"{}\"\n", opts.name);
    frontmatter.push_str(&format!("description: \"{}\"\n", description));
    if !category.is_empty() {
        frontmatter.push_str(&format!("category: {}\n", category));
    }
    if !tags_raw.is_empty() {
        let tag_list: Vec<&str> = tags_raw.split(',').map(str::trim).collect();
        frontmatter.push_str(&format!("tags: [{}]\n", tag_list.join(", ")));
    }

    let body = render_starter(COMMAND_STARTER, &opts.name);
    let content = format!("---\n{}---\n\n{}", frontmatter, body);

    write_file(&target, &content).context("failed to write command file")?;

    success!("Created {}", target.display());

    maybe_prompt_deploy(opts.deploy)?;

    if use_interactive {
        outro("").ok();
    }

    Ok(true)
}

/// Handle `dotagents add skill`.
fn add_skill(opts: AddSkillOptions) -> Result<bool> {
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

    if use_interactive {
        intro("dotagents add skill").ok();
    }

    let description = collect_field(opts.description, "Description", "What does this skill do?")?;
    let license = collect_field(opts.license, "License", "e.g. MIT")?;
    let compatibility = collect_field(
        opts.compatibility,
        "Compatibility",
        "e.g. Requires openspec CLI.",
    )?;

    // Build frontmatter.
    let mut frontmatter = format!("name: {}\n", opts.name);
    frontmatter.push_str(&format!("description: \"{}\"\n", description));
    if !license.is_empty() {
        frontmatter.push_str(&format!("license: {}\n", license));
    }
    if !compatibility.is_empty() {
        frontmatter.push_str(&format!("compatibility: \"{}\"\n", compatibility));
    }
    frontmatter.push_str("metadata:\n  version: \"1.0\"\n");

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

/// Dispatch `dotagents add`.
pub(crate) fn run_add(action: AddAction) -> Result<bool> {
    match action {
        AddAction::Command(opts) => add_command(opts),
        AddAction::Skill(opts) => add_skill(opts),
    }
}
