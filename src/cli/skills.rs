use anyhow::{Result, bail};
use std::io::ErrorKind;

use crate::schema::config::app::AppConfig;
use crate::schema::config::common::PackageRunner;
use crate::templates::get_templater;
use crate::utils::path::get_application_dir;

use super::options::SkillsAddOptions;

/// Install a skill into `.dotagents/skills/` by wrapping the `skills` CLI.
pub(crate) fn add(opts: SkillsAddOptions) -> Result<bool> {
    let templater = get_templater();
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

    let args = runner.args(&opts.name);
    let (program, rest) = args
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("runner produced empty args list"))?;

    let status = std::process::Command::new(program)
        .args(rest)
        .env("CLAUDE_CONFIG_DIR", claude_config_dir)
        .status()?;

    Ok(status.success())
}
