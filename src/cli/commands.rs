use std::collections::BTreeMap;
use std::fs;

use anyhow::{Context, Result};
use cliclack::{confirm, input, outro};
use gray_matter::Matter;
use gray_matter::engine::YAML;
use serde_json::Value;
use serde_yaml::Value as YamlValue;

use crate::cli::deploy::deploy;
use crate::cli::options::{
    AddCommandOptions, CommandsAction, DeployOptions, RmCommandOptions, SubLsOptions,
};
use crate::cli::ui::ls::{ListItem, render_commands, to_json_array};
use crate::constants::templates::{COMMAND_STARTER, render_starter};
use crate::core::config::CacheConfig;
use crate::prelude::*;
use crate::utils::fs::write_file;
use crate::utils::path::{get_application_dir, get_commands_dir, get_workspace_dir};
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

    let description = collect_field(
        opts.description,
        "Description",
        "What does this command do?",
    )?;
    let category = collect_field(opts.category, "Category", "e.g. Workflow")?;
    let tags_raw = collect_field(opts.tags, "Tags (comma-separated)", "e.g. workflow,explore")?;

    // Build frontmatter via serde_yaml to properly escape all values.
    let tags: Vec<String> = if tags_raw.is_empty() {
        vec![]
    } else {
        tags_raw.split(',').map(|t| t.trim().to_string()).collect()
    };
    let mut fm: BTreeMap<&str, YamlValue> = BTreeMap::new();
    fm.insert("name", YamlValue::String(opts.name.clone()));
    fm.insert("description", YamlValue::String(description));
    fm.insert("category", YamlValue::String(category));
    fm.insert(
        "tags",
        YamlValue::Sequence(tags.into_iter().map(YamlValue::String).collect()),
    );
    let frontmatter = serde_yaml::to_string(&fm).context("failed to serialize frontmatter")?;

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

    fs::remove_file(&target).with_context(|| format!("failed to remove {}", target.display()))?;

    success!("Removed {}", target.display());

    // Clean up deployed output across all providers.
    if let Some(mut cache) = cache_opt {
        match get_workspace_dir() {
            Ok(workspace_dir) => {
                if let Err(e) = super::undeploy::undeploy_item(
                    "commands",
                    &opts.name,
                    &mut cache,
                    &workspace_dir,
                ) {
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

/// Load all commands from a given directory, returning full frontmatter + body.
fn load_commands_from(dir: &std::path::Path) -> Result<Vec<ListItem>> {
    let matter = Matter::<YAML>::new();
    let mut items = Vec::new();

    for entry in fs::read_dir(dir).context("failed to read commands directory")? {
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
        if let Some(ref data) = parsed.data {
            let name: String = data["name"].as_str().unwrap_or("").to_string();
            let description: String = data["description"].as_str().unwrap_or("").to_string();
            if !name.is_empty() {
                items.push(ListItem {
                    name,
                    description,
                    frontmatter: data.clone(),
                    body: Some(parsed.content),
                });
            }
        }
    }

    items.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(items)
}

/// Load all commands from `.dotagents/commands/*.md`, returning full frontmatter + body.
fn load_commands() -> Result<Vec<ListItem>> {
    let commands_dir = match get_commands_dir() {
        Ok(d) => d,
        Err(_) => return Ok(vec![]), // commands dir absent → empty
    };
    load_commands_from(&commands_dir)
}

/// Handle `dotagents commands ls`.
fn ls_commands(opts: SubLsOptions) -> Result<bool> {
    get_application_dir().context(
        "No .dotagents directory found. Run `dotagents init` to initialise a workspace.",
    )?;
    let mut commands = load_commands().context("failed to load commands")?;

    if let Some(ref filter) = opts.command {
        commands.retain(|item| item.name == *filter);
    }

    if opts.json {
        let json_items = to_json_array(&commands, opts.content);
        let output = serde_json::to_string_pretty(&json_items)?;
        println!("{}", output);
        return Ok(true);
    }

    render_commands(commands, opts.content);
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

    fn make_command_full(
        dir: &std::path::Path,
        name: &str,
        description: &str,
        category: &str,
        tags: &[&str],
        body: &str,
    ) {
        let tags_yaml: String = tags
            .iter()
            .map(|t| format!("  - {}", t))
            .collect::<Vec<_>>()
            .join("\n");
        let content = format!(
            "---\nname: \"{}\"\ndescription: \"{}\"\ncategory: \"{}\"\ntags:\n{}\n---\n\n{}",
            name, description, category, tags_yaml, body
        );
        fs::write(dir.join(format!("{}.md", name)), content).unwrap();
    }

    #[test]
    fn load_commands_reads_md_frontmatter() {
        // load_commands parses name and description from command .md files
        let tmp = TempDir::new().unwrap();
        make_command(tmp.path(), "hello", "Says hello");
        make_command(tmp.path(), "world", "Says world");

        let items = load_commands_from(tmp.path()).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "hello");
        assert_eq!(items[1].name, "world");
    }

    #[test]
    fn load_commands_includes_body_content() {
        // load_commands returns body content alongside frontmatter
        let tmp = TempDir::new().unwrap();
        make_command(tmp.path(), "hello", "Says hello");

        let items = load_commands_from(tmp.path()).unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0].body.as_deref().unwrap().contains("Body."));
    }

    #[test]
    fn load_commands_returns_full_frontmatter() {
        // load_commands returns frontmatter with all fields (category, tags)
        let tmp = TempDir::new().unwrap();
        make_command_full(
            tmp.path(),
            "test-cmd",
            "Test desc",
            "Testing",
            &["a", "b"],
            "Body text",
        );

        let items = load_commands_from(tmp.path()).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].frontmatter["category"], "Testing");
        assert_eq!(items[0].frontmatter["tags"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn load_commands_empty_dir_returns_empty_vec() {
        // load_commands returns empty vec when dir has no .md files
        let tmp = TempDir::new().unwrap();
        let items = load_commands_from(tmp.path()).unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn to_json_array_with_content_includes_content_key() {
        // to_json_array with content=true includes the content key
        let items = vec![ListItem {
            name: "test".into(),
            description: "desc".into(),
            frontmatter: serde_json::json!({"name": "test"}),
            body: Some("body text".into()),
        }];
        let result = to_json_array(&items, true);
        assert_eq!(result[0]["content"], "body text");
    }

    #[test]
    fn to_json_array_without_content_omits_content_key() {
        // to_json_array with content=false does not include content key
        let items = vec![ListItem {
            name: "test".into(),
            description: "desc".into(),
            frontmatter: serde_json::json!({"name": "test"}),
            body: Some("body text".into()),
        }];
        let result = to_json_array(&items, false);
        assert!(result[0].get("content").is_none());
    }

    #[test]
    fn to_json_array_handles_non_object_frontmatter() {
        // to_json_array does not panic when frontmatter is not an object
        let items = vec![ListItem {
            name: "test".into(),
            description: "desc".into(),
            frontmatter: Value::String("not-an-object".into()),
            body: Some("body".into()),
        }];
        let result = to_json_array(&items, true);
        assert_eq!(result[0], Value::String("not-an-object".into()));
    }
}
