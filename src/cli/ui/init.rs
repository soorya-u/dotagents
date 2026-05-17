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
        let overwrite = sel.interact().context("unable to get overwrite choice")?;
        if !overwrite {
            outro_cancel("Init cancelled.").ok();
            return Ok(false);
        }
        opts.force = true;
    }

    if opts.features.is_none() {
        let mut ms = multiselect("Which features do you want to enable?")
            .item(
                "commands",
                "Custom Commands",
                "Sync slash commands to your AI tools",
            )
            .item(
                "instructions",
                "INSTRUCTIONS.md",
                "Sync a global INSTRUCTIONS.md",
            )
            .item("mcp", "MCP Configuration", "Sync MCP server configuration")
            .item("skills", "Skills", "Sync skills")
            .initial_values(vec!["commands", "instructions", "mcp", "skills"])
            .required(false);
        let features = ms.interact().context("unable to get feature selection")?;

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
    }

    if opts.template.is_none() {
        let mut ts = select("Which starting template?")
            .item(InitTemplate::Starter, "Starter", "Core files only")
            .item(
                InitTemplate::WithCustomProvider,
                "With Custom Provider",
                "Adds an example of a custom provider",
            );
        let template = ts.interact().context("unable to get template choice")?;
        opts.template = Some(template);
    }

    if opts.targets.is_none() {
        opts.targets = prompt_targets(&[])?.or(Some(vec![]));
    }

    Ok(true)
}

/// Fetches the provider registry and prompts the user to select deployment targets.
/// Returns `Some(selected)` with the chosen provider names, or `None` if the registry
/// could not be reached (so callers can distinguish failure from an empty selection).
/// `initial` lists provider names that should be pre-checked in the multiselect.
pub(crate) fn prompt_targets(initial: &[String]) -> Result<Option<Vec<String>>> {
    let sp = spinner();
    sp.start("Fetching provider registry…");

    let registry = match Registry::fetch(registry_url()) {
        Ok(r) => {
            sp.clear();
            r
        }
        Err(e) => {
            sp.error(format!("Could not reach registry: {}", e));
            cliclack::log::warning(
                "Skipping provider selection — retry with a network connection to set targets.",
            )
            .ok();
            return Ok(None);
        }
    };

    let mut providers: Vec<(String, String)> = registry
        .providers
        .iter()
        .map(|(slug, entry)| {
            let label = match &entry.name {
                Some(name) => format!("{} [{}]", name, slug),
                None => slug.clone(),
            };
            (slug.clone(), label)
        })
        .collect();
    providers.sort_by(|a, b| a.0.cmp(&b.0));

    if providers.is_empty() {
        return Ok(Some(vec![]));
    }

    let initial_values: Vec<String> = initial
        .iter()
        .filter(|p| providers.iter().any(|(slug, _)| slug == *p))
        .cloned()
        .collect();

    let mut ms = multiselect::<String>("Which providers would you like to target?")
        .required(false)
        .max_rows(12)
        .initial_values(initial_values);
    for (slug, label) in &providers {
        ms = ms.item(slug.clone(), label.as_str(), "");
    }
    let selected = ms.interact().context("unable to get provider selection")?;

    Ok(Some(selected))
}

/// Shows the closing outro message after init completes successfully.
pub(crate) fn finish_init() {
    outro("Done! Run `dotagents deploy` to render your templates.").ok();
}
