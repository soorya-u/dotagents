use anyhow::{Context, Result};
use cliclack::{multiselect, outro, outro_cancel, select, spinner};

use crate::cli::options::{Feature, InitOptions, InitTemplate};
use crate::schema::registry::Registry;
use crate::templates::registry_url;

/// Runs the interactive init wizard, populating `opts` from user input.
/// Returns true if init should proceed, false if the user chose to cancel.
pub(crate) fn run_init_wizard(opts: &mut InitOptions, dir_exists: bool) -> Result<bool> {
    // Overwrite confirmation — only shown when the directory exists and --force was not passed.
    if dir_exists && !opts.force {
        let mut sel = select("A .dotagents directory already exists. Overwrite?")
            .item(false, "No, cancel", "")
            .item(true, "Yes, overwrite", "existing files will be deleted");
        let overwrite = sel.interact().context("Failed to get overwrite choice")?;
        if !overwrite {
            outro_cancel("Init cancelled.").ok();
            return Ok(false);
        }
        opts.force = true;
    }

    // Feature multiselect — all four features pre-checked by default.
    let mut ms = multiselect("Which features do you want to enable?")
        .item(
            "commands",
            "Custom Commands",
            "Sync slash commands to your AI tools",
        )
        .item("instructions", "AGENTS.md", "Sync a global AGENTS.md")
        .item("mcp", "MCP Configuration", "Sync MCP server configuration")
        .item("skills", "Skills", "Sync skills")
        .initial_values(vec!["commands", "instructions", "mcp", "skills"])
        .required(false);
    let features = ms.interact().context("Failed to get feature selection")?;

    // Map the string selections back to Feature enum values and store in opts.features.
    let feature_list: Vec<Feature> = features
        .iter()
        .filter_map(|&f| match f {
            "commands" => Some(Feature::Commands),
            "instructions" => Some(Feature::Instructions),
            "mcp" => Some(Feature::Mcp),
            "skills" => Some(Feature::Skills),
            _ => None,
        })
        .collect();
    opts.features = Some(feature_list);

    // Template select — Starter is the first item (default).
    let mut ts = select("Which starting template?")
        .item(InitTemplate::Starter, "Starter", "Core files only")
        .item(
            InitTemplate::WithCustomProvider,
            "With Custom Provider",
            "Adds an example of a custom provider",
        );
    let template = ts.interact().context("Failed to get template choice")?;
    opts.template = Some(template);

    // Provider selection — runs before files are written so targets are known upfront.
    opts.targets = prompt_targets()?;

    Ok(true)
}

/// Fetches the provider registry and prompts the user to select deployment targets.
/// Returns the selected provider names sorted alphabetically.
/// On registry fetch failure, warns and returns an empty vec.
pub(crate) fn prompt_targets() -> Result<Vec<String>> {
    let mut sp = spinner();
    sp.start("Fetching provider registry…");

    let registry = match Registry::fetch(registry_url()) {
        Ok(r) => {
            sp.clear();
            r
        }
        Err(e) => {
            sp.error(format!("Could not reach registry: {}", e));
            cliclack::log::warning(
                "Skipping provider selection — run `dotagents init` again with a network connection to set targets.",
            )
            .ok();
            return Ok(vec![]);
        }
    };

    let mut providers: Vec<String> = registry.providers.keys().cloned().collect();
    providers.sort();

    if providers.is_empty() {
        return Ok(vec![]);
    }

    let mut ms = multiselect::<String>("Which providers would you like to target?")
        .required(false)
        .max_rows(12);
    for provider in &providers {
        ms = ms.item(provider.clone(), provider.as_str(), "");
    }
    let selected = ms.interact().context("Failed to get provider selection")?;

    Ok(selected)
}

/// Shows the closing outro message after init completes successfully.
pub(crate) fn finish_init() {
    outro("Done! Run `dotagents deploy` to render your templates.").ok();
}
