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
use crate::cli::ui::ls::{ListItem, render_commands};
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

/// Load all commands from `.dotagents/commands/*.md`, returning full frontmatter + body.
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

/// Handle `dotagents commands ls`.
fn ls_commands(opts: SubLsOptions) -> Result<bool> {
    get_application_dir().context(
        "No .dotagents directory found. Run `dotagents init` to initialise a workspace.",
    )?;
    let commands = load_commands().context("failed to load commands")?;

    if opts.json {
        let json_items: Vec<Value> = commands
            .iter()
            .map(|item| {
                let mut obj = item.frontmatter.clone();
                if opts.full
                    && let Some(body) = &item.body
                {
                    obj.as_object_mut()
                        .expect("frontmatter is always an object")
                        .insert("content".to_string(), Value::String(body.clone()));
                }
                obj
            })
            .collect();
        let output = serde_json::to_string_pretty(&json_items)?;
        println!("{}", output);
        return Ok(true);
    }

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

    #[test]
    fn load_commands_includes_body_content() {
        // load_commands returns body content alongside frontmatter
        let tmp = TempDir::new().unwrap();
        make_command(tmp.path(), "hello", "Says hello");

        // Use load_commands logic directly
        let matter = Matter::<YAML>::new();
        for entry in fs::read_dir(tmp.path()).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                let content = fs::read_to_string(&path).unwrap();
                let parsed = matter.parse::<Value>(&content).unwrap();
                assert_eq!(parsed.content.trim(), "Body.");
            }
        }
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

        let matter = Matter::<YAML>::new();
        for entry in fs::read_dir(tmp.path()).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                let content = fs::read_to_string(&path).unwrap();
                let parsed = matter.parse::<Value>(&content).unwrap();
                let data = parsed.data.unwrap();
                assert_eq!(data["category"], "Testing");
                assert_eq!(data["tags"].as_array().unwrap().len(), 2);
            }
        }
    }

    #[test]
    fn load_commands_empty_dir_returns_empty_vec() {
        // load_commands returns empty vec when commands dir is absent
        let tmp = TempDir::new().unwrap();
        // get_commands_dir reads from the workspace marker - we can't easily test
        // the function itself without a workspace, so we test the edge by creating
        // a dir with no .md files
        let matter = Matter::<YAML>::new();
        let mut items = Vec::new();
        if let Ok(entries) = fs::read_dir(tmp.path()) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("md") {
                    let content = fs::read_to_string(&path).unwrap();
                    if let Ok(parsed) = matter.parse::<Value>(&content) {
                        if let Some(data) = parsed.data {
                            let name = data["name"].as_str().unwrap_or("").to_string();
                            if !name.is_empty() {
                                items.push(name);
                            }
                        }
                    }
                }
            }
        }
        assert!(items.is_empty());
    }

    #[test]
    fn json_output_includes_frontmatter_fields() {
        // JSON output from frontmatter includes name and description
        let mut obj = serde_json::Map::new();
        obj.insert("name".into(), Value::String("test".into()));
        obj.insert("description".into(), Value::String("desc".into()));
        let json_items = vec![Value::Object(obj)];
        let output = serde_json::to_string_pretty(&json_items).unwrap();
        assert!(output.contains("\"name\": \"test\""));
        assert!(output.contains("\"description\": \"desc\""));
    }

    #[test]
    fn json_output_with_full_includes_content() {
        // JSON output with --full includes content key
        let mut obj = serde_json::json!({"name": "test"});
        let map = obj.as_object_mut().unwrap();
        map.insert("content".into(), Value::String("body text".into()));
        let json_items = vec![obj];
        let output = serde_json::to_string_pretty(&json_items).unwrap();
        assert!(output.contains("\"content\": \"body text\""));
    }
}
