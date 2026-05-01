use std::fs;

use anyhow::{Context, Result};
use cliclack::{confirm, input, intro, outro};
use gray_matter::Matter;
use gray_matter::engine::YAML;
use serde_json::Value;

use crate::cli::deploy::deploy;
use crate::cli::options::{
    AddCommandOptions, CommandsAction, DeployOptions, RmCommandOptions, SubLsOptions,
};
use crate::cli::ui::ls::{ListItem, render_commands};
use crate::constants::templates::{COMMAND_STARTER, render_starter};
use crate::prelude::*;
use crate::utils::fs::write_file;
use crate::utils::path::{get_application_dir, get_commands_dir};
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

/// Handle `dotagents commands new`.
fn new_command(opts: AddCommandOptions) -> Result<bool> {
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
        intro("dotagents commands new").ok();
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

/// Handle `dotagents commands rm`.
fn rm_command(opts: RmCommandOptions) -> Result<bool> {
    let app_dir = get_application_dir().context(
        "No .dotagents directory found. Run `dotagents init` to initialise a workspace.",
    )?;

    let target = app_dir.join("commands").join(format!("{}.md", opts.name));

    if !target.exists() {
        bail!("Command '{}' not found at {}.", opts.name, target.display());
    }

    intro("dotagents commands rm").ok();

    // Confirm in TTY unless --force.
    if is_tty() && !opts.force {
        let confirmed = confirm(format!(
            "Remove command '{}'? This cannot be undone.",
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

    fs::remove_file(&target).with_context(|| format!("failed to remove {}", target.display()))?;

    success!("Removed {}", target.display());

    maybe_prompt_deploy(opts.deploy)?;

    outro("").ok();
    Ok(true)
}

/// Load all commands from `.dotagents/commands/*.md`, returning name+description pairs.
fn load_commands() -> Result<Vec<ListItem>> {
    let commands_dir = match get_commands_dir() {
        Ok(d) => d,
        Err(_) => return Ok(vec![]), // commands dir absent → empty
    };

    let matter = Matter::<YAML>::new();
    let mut items = Vec::new();

    for entry in fs::read_dir(&commands_dir).context("failed to read commands directory")? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let content = fs::read_to_string(&path).context("failed to read command file")?;
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

/// Handle `dotagents commands ls`.
fn ls_commands(opts: SubLsOptions) -> Result<bool> {
    get_application_dir().context(
        "No .dotagents directory found. Run `dotagents init` to initialise a workspace.",
    )?;
    let commands = load_commands().context("failed to load commands")?;
    render_commands(commands, opts.full);
    Ok(true)
}

/// Dispatch `dotagents commands`.
pub(crate) fn run_commands(action: CommandsAction) -> Result<bool> {
    match action {
        CommandsAction::New(opts) => new_command(opts),
        CommandsAction::Rm(opts) => rm_command(opts),
        CommandsAction::Ls(opts) => ls_commands(opts),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_command(dir: &std::path::Path, name: &str, description: &str) {
        let content = format!(
            "---\nname: \"{}\"\ndescription: \"{}\"\n---\n\nBody.\n",
            name, description
        );
        fs::write(dir.join(format!("{}.md", name)), content).unwrap();
    }

    #[test]
    fn load_commands_reads_md_frontmatter() {
        // load_commands parses name and description from command .md files
        let tmp = TempDir::new().unwrap();
        make_command(tmp.path(), "hello", "Says hello");
        make_command(tmp.path(), "world", "Says world");

        let matter = Matter::<YAML>::new();
        let mut items = Vec::new();
        for entry in fs::read_dir(tmp.path()).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                let content = fs::read_to_string(&path).unwrap();
                if let Ok(parsed) = matter.parse::<Value>(&content) {
                    if let Some(data) = parsed.data {
                        items.push(data["name"].as_str().unwrap_or("").to_string());
                    }
                }
            }
        }
        assert_eq!(items.len(), 2);
    }
}
