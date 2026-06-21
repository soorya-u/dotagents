use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::path::Path;

use crate::cli::SkillsAddOptions;
use crate::core::config::app::AppConfig;
use crate::prelude::*;
use crate::templates::get_templater;
use crate::utils::path::{get_application_dir, resolve_and_override_workspace};
use crate::utils::tui::is_tui_enabled;

/// Package runner used to invoke the `skills` CLI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub(crate) enum PackageRunner {
    Npm,
    Pnpm,
    Yarn,
    Bun,
}

impl PackageRunner {
    /// Returns the executable name to check on PATH and use as the first argv element.
    pub(crate) fn binary(&self) -> &str {
        match self {
            PackageRunner::Npm => "npx",
            PackageRunner::Pnpm => "pnpm",
            PackageRunner::Yarn => "yarn",
            PackageRunner::Bun => "bunx",
        }
    }

    /// Returns the full argument list for `skills add <skill_name>`.
    /// Uses `--agent openclaw` (flat `skills/` dir matches dotagents layout) and `--copy` (real files).
    /// When `ci` is true, appends `--yes` to skip interactive confirmation prompts.
    pub(crate) fn args(&self, skill_name: &str, ci: bool) -> Vec<String> {
        let mut v = match self {
            PackageRunner::Npm => vec![
                "npx".into(),
                "skills".into(),
                "add".into(),
                skill_name.into(),
                "--agent".into(),
                "openclaw".into(),
                "--copy".into(),
            ],
            PackageRunner::Pnpm => vec![
                "pnpm".into(),
                "dlx".into(),
                "skills".into(),
                "add".into(),
                skill_name.into(),
                "--agent".into(),
                "openclaw".into(),
                "--copy".into(),
            ],
            PackageRunner::Yarn => vec![
                "yarn".into(),
                "dlx".into(),
                "skills".into(),
                "add".into(),
                skill_name.into(),
                "--agent".into(),
                "openclaw".into(),
                "--copy".into(),
            ],
            PackageRunner::Bun => vec![
                "bunx".into(),
                "skills".into(),
                "add".into(),
                skill_name.into(),
                "--agent".into(),
                "openclaw".into(),
                "--copy".into(),
            ],
        };
        if ci {
            v.push("--yes".into());
        }
        v
    }
}

/// Returns the full argument list for `skills remove <skill_name>`.
/// Uses `--agent openclaw` and `--yes` (always non-interactive for rm delegation).
fn remove_args(runner: &PackageRunner, skill_name: &str) -> Vec<String> {
    match runner {
        PackageRunner::Npm => vec![
            "npx".into(),
            "skills".into(),
            "remove".into(),
            skill_name.into(),
            "--agent".into(),
            "openclaw".into(),
            "--yes".into(),
        ],
        PackageRunner::Pnpm => vec![
            "pnpm".into(),
            "dlx".into(),
            "skills".into(),
            "remove".into(),
            skill_name.into(),
            "--agent".into(),
            "openclaw".into(),
            "--yes".into(),
        ],
        PackageRunner::Yarn => vec![
            "yarn".into(),
            "dlx".into(),
            "skills".into(),
            "remove".into(),
            skill_name.into(),
            "--agent".into(),
            "openclaw".into(),
            "--yes".into(),
        ],
        PackageRunner::Bun => vec![
            "bunx".into(),
            "skills".into(),
            "remove".into(),
            skill_name.into(),
            "--agent".into(),
            "openclaw".into(),
            "--yes".into(),
        ],
    }
}

/// Install a skill into `.dotagents/skills/` by wrapping the `skills` CLI.
pub(crate) fn add(opts: SkillsAddOptions) -> Result<bool> {
    resolve_and_override_workspace(opts.workspace.cwd)
        .context("unable to resolve workspace directory")?;

    let templater = get_templater()?;
    let app_config = AppConfig::from_application(templater)?;

    // Resolve runner: CLI flag > config > silent default (npm)
    let explicit_runner = opts.runner.or(app_config
        .integrations
        .as_ref()
        .and_then(|i| i.skills_sh.as_ref())
        .and_then(|s| s.package_runner.clone()));

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
                         Check the `[integrations.skills-sh]` package-runner setting in your config.toml.",
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

    let args = runner.args(&opts.name, !is_tui_enabled());
    let (program, rest) = args
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("runner produced empty args list"))?;

    let status = std::process::Command::new(program)
        .args(rest)
        .current_dir(&application_dir)
        .status()?;

    if status.success() {
        // Post-install assertion: openclaw coupling mitigation
        let skill_dir_name = opts.name.rsplit('/').next().unwrap_or(&opts.name);
        let skill_md = application_dir
            .join("skills")
            .join(skill_dir_name)
            .join("SKILL.md");
        if !skill_md.exists() {
            bail!(
                "skills CLI reported success but SKILL.md was not found at {}.\n\
                 The openclaw agent's skills directory may have changed upstream.",
                skill_md.display()
            );
        }
    }

    Ok(status.success())
}

/// Remove an externally-installed skill by wrapping the `skills` CLI.
pub(crate) fn remove(
    skill_name: &str,
    application_dir: &Path,
    runner: &PackageRunner,
) -> Result<bool> {
    let args = remove_args(runner, skill_name);
    let (program, rest) = args
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("runner produced empty args list"))?;

    let status = std::process::Command::new(program)
        .args(rest)
        .current_dir(application_dir)
        .status()?;

    Ok(status.success())
}

/// Parsed entry from `skills-lock.json`.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LockfileEntry {
    #[serde(default)]
    source: Option<String>,
}

/// Parsed `skills-lock.json` structure.
#[derive(Debug, Deserialize)]
struct Lockfile {
    #[serde(default)]
    skills: std::collections::HashMap<String, LockfileEntry>,
}

/// Read `<application_dir>/skills-lock.json` and check if `skill_name` is present.
/// Returns false on missing/malformed file (treat as locally authored).
pub(crate) fn is_external_skill(skill_name: &str, application_dir: &Path) -> bool {
    let lockfile_path = application_dir.join("skills-lock.json");
    let content = match std::fs::read_to_string(&lockfile_path) {
        Ok(c) => c,
        Err(_) => {
            debug!(
                "skills-lock.json not found at {}, treating as local",
                lockfile_path.display()
            );
            return false;
        }
    };
    let lockfile: Lockfile = match serde_json::from_str(&content) {
        Ok(l) => l,
        Err(e) => {
            warn!("Failed to parse skills-lock.json, treating as local: {}", e);
            return false;
        }
    };
    lockfile.skills.contains_key(skill_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn package_runner_args_npm() {
        // npm runner produces npx invocation with openclaw and copy
        let args = PackageRunner::Npm.args("vercel-labs/agent-skills", false);
        assert_eq!(
            args,
            vec![
                "npx",
                "skills",
                "add",
                "vercel-labs/agent-skills",
                "--agent",
                "openclaw",
                "--copy"
            ]
        );
    }

    #[test]
    fn package_runner_args_npm_ci() {
        // ci mode appends --yes
        let args = PackageRunner::Npm.args("vercel-labs/agent-skills", true);
        assert_eq!(
            args,
            vec![
                "npx",
                "skills",
                "add",
                "vercel-labs/agent-skills",
                "--agent",
                "openclaw",
                "--copy",
                "--yes"
            ]
        );
    }

    #[test]
    fn package_runner_args_pnpm() {
        // pnpm runner produces pnpm dlx invocation
        let args = PackageRunner::Pnpm.args("my-skill", false);
        assert_eq!(
            args,
            vec![
                "pnpm", "dlx", "skills", "add", "my-skill", "--agent", "openclaw", "--copy"
            ]
        );
    }

    #[test]
    fn package_runner_args_pnpm_ci() {
        // pnpm ci mode appends --yes
        let args = PackageRunner::Pnpm.args("my-skill", true);
        assert_eq!(
            args,
            vec![
                "pnpm", "dlx", "skills", "add", "my-skill", "--agent", "openclaw", "--copy",
                "--yes"
            ]
        );
    }

    #[test]
    fn package_runner_args_yarn() {
        // yarn runner produces yarn dlx invocation
        let args = PackageRunner::Yarn.args("my-skill", false);
        assert_eq!(
            args,
            vec![
                "yarn", "dlx", "skills", "add", "my-skill", "--agent", "openclaw", "--copy"
            ]
        );
    }

    #[test]
    fn package_runner_args_yarn_ci() {
        // yarn ci mode appends --yes
        let args = PackageRunner::Yarn.args("my-skill", true);
        assert_eq!(
            args,
            vec![
                "yarn", "dlx", "skills", "add", "my-skill", "--agent", "openclaw", "--copy",
                "--yes"
            ]
        );
    }

    #[test]
    fn package_runner_args_bun() {
        // bun runner produces bunx invocation
        let args = PackageRunner::Bun.args("my-skill", false);
        assert_eq!(
            args,
            vec![
                "bunx", "skills", "add", "my-skill", "--agent", "openclaw", "--copy"
            ]
        );
    }

    #[test]
    fn package_runner_args_bun_ci() {
        // bun ci mode appends --yes
        let args = PackageRunner::Bun.args("my-skill", true);
        assert_eq!(
            args,
            vec![
                "bunx", "skills", "add", "my-skill", "--agent", "openclaw", "--copy", "--yes"
            ]
        );
    }

    #[test]
    fn package_runner_serialises_to_lowercase() {
        // PackageRunner serialises to lowercase strings
        #[derive(Serialize, Deserialize)]
        struct W {
            r: PackageRunner,
        }
        for (variant, expected) in [
            (PackageRunner::Npm, "npm"),
            (PackageRunner::Pnpm, "pnpm"),
            (PackageRunner::Yarn, "yarn"),
            (PackageRunner::Bun, "bun"),
        ] {
            let s = toml::to_string(&W { r: variant }).unwrap();
            assert!(
                s.contains(&format!("\"{expected}\"")),
                "expected \"{expected}\" in: {s}"
            );
        }
    }

    #[test]
    fn package_runner_deserialises_from_lowercase() {
        // PackageRunner deserialises from lowercase strings
        #[derive(Serialize, Deserialize)]
        struct W {
            r: PackageRunner,
        }
        for (toml_val, expected) in [
            ("npm", PackageRunner::Npm),
            ("pnpm", PackageRunner::Pnpm),
            ("yarn", PackageRunner::Yarn),
            ("bun", PackageRunner::Bun),
        ] {
            let w: W = toml::from_str(&format!("r = \"{toml_val}\"\n")).unwrap();
            assert_eq!(w.r, expected);
        }
    }

    #[test]
    fn is_external_skill_returns_true_when_present_in_lockfile() {
        // lockfile entry exists for the skill
        let tmp = TempDir::new().unwrap();
        let lockfile = r#"{"version":1,"skills":{"find-skills":{"source":"vercel-labs/skills","sourceType":"github","skillPath":"skills/find-skills/SKILL.md","computedHash":"abc"}}}"#;
        fs::write(tmp.path().join("skills-lock.json"), lockfile).unwrap();
        assert!(is_external_skill("find-skills", tmp.path()));
    }

    #[test]
    fn is_external_skill_returns_false_when_absent_from_lockfile() {
        // lockfile exists but skill is not in it
        let tmp = TempDir::new().unwrap();
        let lockfile = r#"{"version":1,"skills":{"find-skills":{"source":"vercel-labs/skills"}}}"#;
        fs::write(tmp.path().join("skills-lock.json"), lockfile).unwrap();
        assert!(!is_external_skill("my-local-skill", tmp.path()));
    }

    #[test]
    fn is_external_skill_returns_false_when_lockfile_missing() {
        // no lockfile means locally authored
        let tmp = TempDir::new().unwrap();
        assert!(!is_external_skill("any-skill", tmp.path()));
    }

    #[test]
    fn is_external_skill_returns_false_when_lockfile_malformed() {
        // malformed lockfile is treated as local
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("skills-lock.json"), "not valid json {").unwrap();
        assert!(!is_external_skill("any-skill", tmp.path()));
    }

    #[test]
    fn remove_args_npm() {
        // remove args use openclaw and --yes
        let args = remove_args(&PackageRunner::Npm, "my-skill");
        assert_eq!(
            args,
            vec![
                "npx", "skills", "remove", "my-skill", "--agent", "openclaw", "--yes"
            ]
        );
    }

    #[test]
    fn remove_args_bun() {
        // remove args with bun runner
        let args = remove_args(&PackageRunner::Bun, "my-skill");
        assert_eq!(
            args,
            vec![
                "bunx", "skills", "remove", "my-skill", "--agent", "openclaw", "--yes"
            ]
        );
    }
}
