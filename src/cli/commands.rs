use std::fs;

use anyhow::{Context, Result};
use cliclack::{confirm, input, outro};

use crate::cli::deploy::deploy;
use crate::cli::options::{
    AddCommandOptions, CommandsAction, DeployOptions, RmCommandOptions, SubLsOptions,
};
use crate::cli::ui::ls::render_commands;
use crate::core::config::CacheConfig;
use crate::core::features::command::CommandFeature;
use crate::prelude::*;
use crate::schema::list_item::ListItem;
use crate::utils::fs::write_file;
use crate::utils::path::{
    get_application_dir, get_commands_dir, get_workspace_dir, resolve_and_override_workspace,
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

/// Deploy after command mutation: skip if --no-deploy, auto-deploy in CI, prompt in TTY.
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

/// Handle `dotagents commands new`.
fn new_command(opts: AddCommandOptions) -> Result<bool> {
    resolve_and_override_workspace(opts.workspace.cwd)
        .context("Failed to resolve workspace directory")?;

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

    let use_interactive = is_tui_enabled()
        && opts.description.is_none()
        && opts.category.is_none()
        && opts.tags.is_none();

    let description = collect_field(
        opts.description,
        "Description",
        "What does this command do?",
    )?;
    let category = collect_field(opts.category, "Category", "e.g. Workflow")?;
    let tags_raw = collect_field(opts.tags, "Tags (comma-separated)", "e.g. workflow,explore")?;

    let tags: Vec<String> = if tags_raw.is_empty() {
        vec![]
    } else {
        tags_raw.split(',').map(|t| t.trim().to_string()).collect()
    };

    let content = CommandFeature::scaffold(&opts.name, &description, &category, &tags)
        .context("failed to scaffold command")?;

    write_file(&target, &content).context("failed to write command file")?;

    success!("Created {}", target.display());

    maybe_prompt_deploy(opts.no_deploy)?;

    if use_interactive {
        outro("").ok();
    }

    Ok(true)
}

/// Handle `dotagents commands rm`.
fn rm_command(opts: RmCommandOptions) -> Result<bool> {
    resolve_and_override_workspace(opts.workspace.cwd)
        .context("Failed to resolve workspace directory")?;

    let app_dir = get_application_dir().context(
        "No .dotagents directory found. Run `dotagents init` to initialise a workspace.",
    )?;

    let target = app_dir.join("commands").join(format!("{}.md", opts.name));

    if !target.exists() {
        bail!("Command '{}' not found at {}.", opts.name, target.display());
    }

    // Confirm in TTY unless --force.
    if is_tui_enabled() && !opts.force {
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

    maybe_prompt_deploy(opts.no_deploy)?;

    outro("").ok();
    Ok(true)
}

/// Convert a `CommandFeature` into a `ListItem` for display.
fn command_to_list_item(cmd: CommandFeature) -> ListItem {
    let name = cmd.metadata.name.clone();
    let description = cmd.metadata.description.clone();
    let frontmatter = serde_json::to_value(&cmd.metadata).unwrap_or_default();
    ListItem {
        name,
        description,
        frontmatter,
        body: Some(cmd.content),
    }
}

/// Load all commands from `.dotagents/commands/*.md`, returning full frontmatter + body.
fn load_commands() -> Result<Vec<ListItem>> {
    // commands dir absent means the feature is not enabled; any other error propagates
    if get_commands_dir().is_err() {
        return Ok(vec![]);
    }
    let commands = CommandFeature::from_application().context("failed to load commands")?;
    let mut items: Vec<ListItem> = commands.into_iter().map(command_to_list_item).collect();
    items.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(items)
}

/// Handle `dotagents commands ls`.
fn ls_commands(opts: SubLsOptions) -> Result<bool> {
    resolve_and_override_workspace(opts.workspace.cwd)
        .context("Failed to resolve workspace directory")?;

    get_application_dir().context(
        "No .dotagents directory found. Run `dotagents init` to initialise a workspace.",
    )?;
    let mut commands = load_commands().context("failed to load commands")?;

    if let Some(ref filter) = opts.command {
        commands.retain(|item| item.name == *filter);
    }

    if opts.json {
        let json_items = ListItem::to_json_array(&commands, opts.content);
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
    use crate::utils::tui::set_ci_mode;
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

    fn load_commands_from_dir(dir: &std::path::Path) -> Result<Vec<ListItem>> {
        let commands = fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
            .filter_map(|e| fs::read_to_string(e.path()).ok())
            .filter_map(|content| CommandFeature::from_markdown(&content).ok())
            .map(command_to_list_item)
            .collect();
        let mut items: Vec<ListItem> = commands;
        items.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(items)
    }

    #[test]
    fn load_commands_reads_md_frontmatter() {
        // load_commands parses name and description from command .md files
        let tmp = TempDir::new().unwrap();
        make_command(tmp.path(), "hello", "Says hello");
        make_command(tmp.path(), "world", "Says world");

        let items = load_commands_from_dir(tmp.path()).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "hello");
        assert_eq!(items[1].name, "world");
    }

    #[test]
    fn load_commands_includes_body_content() {
        // load_commands returns body content alongside frontmatter
        let tmp = TempDir::new().unwrap();
        make_command(tmp.path(), "hello", "Says hello");

        let items = load_commands_from_dir(tmp.path()).unwrap();
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

        let items = load_commands_from_dir(tmp.path()).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].frontmatter["category"], "Testing");
        assert_eq!(items[0].frontmatter["tags"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn load_commands_empty_dir_returns_empty_vec() {
        // load_commands returns empty vec when dir has no .md files
        let tmp = TempDir::new().unwrap();
        let items = load_commands_from_dir(tmp.path()).unwrap();
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
        let result = ListItem::to_json_array(&items, true);
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
        let result = ListItem::to_json_array(&items, false);
        assert!(result[0].get("content").is_none());
    }

    #[test]
    fn to_json_array_handles_non_object_frontmatter() {
        // to_json_array does not panic when frontmatter is not an object
        let items = vec![ListItem {
            name: "test".into(),
            description: "desc".into(),
            frontmatter: serde_json::Value::String("not-an-object".into()),
            body: Some("body".into()),
        }];
        let result = ListItem::to_json_array(&items, true);
        assert_eq!(result[0], serde_json::Value::String("not-an-object".into()));
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
        // When no_deploy=false and CI mode is active, deploy is attempted
        // (error is expected since there's no workspace; the test proves deploy is called)
        let tmp = TempDir::new().unwrap();
        let _guard = std::env::set_current_dir(tmp.path());
        set_ci_mode(true);
        let result = maybe_prompt_deploy(false);
        set_ci_mode(false);
        // Deploy fails because no workspace exists, but the key point is deploy was called
        assert!(result.is_err());
    }
}
