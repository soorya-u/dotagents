use std::fs;

use anyhow::{Context, Result, bail};
use cliclack::{confirm, intro, log, outro};

use crate::cli::deploy::deploy;
use crate::cli::options::{DeployOptions, RmAction, RmCommandOptions, RmSkillOptions};
use crate::utils::path::get_application_dir;
use crate::utils::tty::is_tty;

/// Prompt to deploy after a removal (same dual-mode pattern as add).
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

/// Handle `dotagents rm command`.
fn rm_command(opts: RmCommandOptions) -> Result<bool> {
    let app_dir = get_application_dir().context(
        "No .dotagents directory found. Run `dotagents init` to initialise a workspace.",
    )?;

    let target = app_dir.join("commands").join(format!("{}.md", opts.name));

    if !target.exists() {
        bail!("Command '{}' not found at {}.", opts.name, target.display());
    }

    intro("dotagents rm command").ok();

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

    log::success(format!("Removed {}", target.display())).ok();

    maybe_prompt_deploy(opts.deploy)?;

    outro("").ok();
    Ok(true)
}

/// Handle `dotagents rm skill`.
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

    intro("dotagents rm skill").ok();

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

    fs::remove_dir_all(&skill_dir)
        .with_context(|| format!("failed to remove {}", skill_dir.display()))?;

    log::success(format!("Removed {}", skill_dir.display())).ok();

    maybe_prompt_deploy(opts.deploy)?;

    outro("").ok();
    Ok(true)
}

/// Dispatch `dotagents rm`.
pub(crate) fn run_rm(action: RmAction) -> Result<bool> {
    match action {
        RmAction::Command(opts) => rm_command(opts),
        RmAction::Skill(opts) => rm_skill(opts),
    }
}
