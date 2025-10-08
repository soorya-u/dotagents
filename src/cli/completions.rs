use super::options::Options;
use anyhow::{Context, Result};
use clap::CommandFactory;
use clap_complete::{Shell, generate, generate_to};
use std::{io, path::PathBuf};

pub(super) fn generate_cli_completions(shell: Shell, to: Option<PathBuf>) -> Result<()> {
    let cmd = &mut Options::command();
    let bin_name = "dotagents";
    if let Some(to) = to {
        generate_to(shell, cmd, bin_name, to).context("write completion to a file")?;
    } else {
        generate(shell, cmd, bin_name, &mut io::stdout());
    }

    Ok(())
}
