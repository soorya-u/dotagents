//! Integration tests for the dotagents CLI.
//!
//! These tests exercise specific behavioral scenarios by spawning the compiled
//! binary with crafted config files, then inspecting output files and exit codes.
//! They complement the unit tests (colocated in `src/`) and the e2e suite (`tests/e2e/`).
//!
//! Suite layout
//! ────────────
//!  config      – AppConfig merge scenarios (feature override, provider disable, variable deep-merge)
//!  render      – Render pipeline (variable/env interpolation, frontmatter stripping)
//!  features    – Feature source-file format and deploy output
//!  cache       – Deploy idempotency and --no-cache flag behaviour
//!  gitignore   – .gitignore fence management (--gitignore / --no-gitignore flags)
//!
//! Run the whole suite:   cargo test --test integration
//! Run one module:        cargo test --test integration config

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

mod cache;
mod config;
mod dedup;
mod features;
mod gitignore;
mod merge;
mod render;
mod undeploy;

// ─────────────────────────────────────────────────────────────────────────────
// Shared test harness
// ─────────────────────────────────────────────────────────────────────────────

pub struct TestWorkspace {
    _temp_dir: TempDir,
    workspace_root: PathBuf,
}

impl TestWorkspace {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("failed to create temp directory");
        let workspace_root = temp_dir.path().to_path_buf();
        TestWorkspace {
            _temp_dir: temp_dir,
            workspace_root,
        }
    }

    pub fn root(&self) -> &Path {
        &self.workspace_root
    }

    /// `.dotagents` (release) or `.dotagents-debug` (debug).
    pub fn active_root_dir(&self) -> PathBuf {
        self.root().join(self.root_dir_name())
    }

    pub fn root_dir_name(&self) -> &'static str {
        #[cfg(debug_assertions)]
        {
            ".dotagents-debug"
        }
        #[cfg(not(debug_assertions))]
        {
            ".dotagents"
        }
    }

    pub fn run_command(&self, args: &[&str]) -> CmdResult {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_dotagents"));
        cmd.current_dir(self.root());
        // Null stdin so the binary sees a non-TTY and never shows interactive prompts.
        cmd.stdin(std::process::Stdio::null());
        for arg in args {
            cmd.arg(arg);
        }
        let output = cmd.output().expect("failed to execute command");
        CmdResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            status: output.status,
        }
    }

    pub fn file_exists(&self, path: impl AsRef<Path>) -> bool {
        self.root().join(path).is_file()
    }

    pub fn dir_exists(&self, path: impl AsRef<Path>) -> bool {
        self.root().join(path).is_dir()
    }

    pub fn read_file(&self, path: impl AsRef<Path>) -> String {
        let full = self.root().join(&path);
        fs::read_to_string(&full)
            .unwrap_or_else(|_| panic!("failed to read file: {}", full.display()))
    }

    pub fn write_file(&self, path: impl AsRef<Path>, content: &str) {
        let full = self.root().join(&path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).expect("failed to create parent dirs");
        }
        fs::write(&full, content)
            .unwrap_or_else(|_| panic!("failed to write file: {}", full.display()));
    }

    pub fn list_files(&self, dir: impl AsRef<Path>) -> Vec<String> {
        let full = self.root().join(dir);
        fs::read_dir(&full)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok().map(|d| d.file_name().to_string_lossy().to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

pub struct CmdResult {
    pub stdout: String,
    pub stderr: String,
    pub status: std::process::ExitStatus,
}

impl CmdResult {
    pub fn is_success(&self) -> bool {
        self.status.success()
    }

    #[track_caller]
    pub fn assert_success(&self) {
        assert!(
            self.is_success(),
            "command failed with status {}\nstdout:\n{}\nstderr:\n{}",
            self.status,
            self.stdout,
            self.stderr
        );
    }

    #[track_caller]
    pub fn assert_failure(&self) {
        assert!(
            !self.is_success(),
            "command succeeded when it should have failed\nstdout:\n{}",
            self.stdout
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers for test modules
// ─────────────────────────────────────────────────────────────────────────────

/// Initialises an `advanced` workspace and strips the remote
/// `gemini` target so that deploy can run fully offline in tests.
pub fn init_with_mycode_provider(ws: &TestWorkspace) {
    ws.run_command(&[
        "init",
        "--template",
        "advanced",
        "--features",
        "command,instruction,mcp,skill",
    ])
    .assert_success();
    let config_path = ws.active_root_dir().join("local.config.toml");
    let content = fs::read_to_string(&config_path).expect("failed to read local.config.toml");
    // Remove the remote gemini target; mycode inline provider remains.
    let patched = content.replace(r#"targets = ["gemini"]"#, "targets = []");
    fs::write(&config_path, patched).expect("failed to patch local.config.toml");
}
