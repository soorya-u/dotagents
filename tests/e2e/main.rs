//! End-to-end integration tests for the dotagents CLI.
//!
//! Every test spawns the compiled binary in its own isolated `/tmp` directory,
//! so tests are completely independent and can run in parallel.  The
//! `WORKSPACE_DIR` `OnceLock` is per-process, so each `Command::new(…)` call
//! starts with a clean slate.
//!
//! Suite layout
//! ────────────
//!  init          – command file-tree, content, and behavioural flags
//!  deploy        – output structure, rendered content, variable interpolation,
//!                  custom configs, and error handling
//!  completions   – gen-completions for all supported shells
//!  no_subcommand – binary invoked with no arguments
//!  idempotency   – repeated deploys and source-file changes
//!  flags         – --verbose / --quiet flag behaviour
//!  workflow      – full end-to-end scenarios
//!
//! Run the whole suite:   cargo test --test e2e
//! Run one test:          cargo test --test e2e <test_name>

#![allow(dead_code)] // helpers used selectively across submodules

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

mod completions;
mod deploy;
mod flags;
mod idempotency;
mod init;
mod no_subcommand;
mod workflow;

// ─────────────────────────────────────────────────────────────────────────────
// Shared test workspace
// ─────────────────────────────────────────────────────────────────────────────

/// An isolated workspace backed by a temporary directory.
///
/// `_temp_dir` owns the directory; it is deleted automatically on drop.
/// Each test creates its own `TestWorkspace` so tests never share state.
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

    // ── path helpers ──────────────────────────────────────────────────────────

    /// Workspace root (the temp dir itself).
    pub fn root(&self) -> &Path {
        &self.workspace_root
    }

    /// `.dotagents` (release) or `.dotagents-debug` (debug) inside the
    /// workspace.
    pub fn root_dir(&self) -> PathBuf {
        self.root().join(self.root_dir_name())
    }

    /// Bare name of the dotagents config directory for the current build
    /// profile.
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

    /// Absolute path of `rel` relative to the workspace root.
    pub fn path(&self, rel: impl AsRef<Path>) -> PathBuf {
        self.root().join(rel)
    }

    // ── filesystem helpers ────────────────────────────────────────────────────

    pub fn file_exists(&self, rel: impl AsRef<Path>) -> bool {
        self.path(rel).is_file()
    }

    pub fn dir_exists(&self, rel: impl AsRef<Path>) -> bool {
        self.path(rel).is_dir()
    }

    /// Read a file relative to the workspace root; panics on error.
    pub fn read(&self, rel: impl AsRef<Path>) -> String {
        let p = self.path(&rel);
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("failed to read {}: {e}", p.display()))
    }

    /// Names (not full paths) of direct children inside `rel`.
    /// Returns an empty `Vec` when the directory does not exist.
    pub fn dir_entries(&self, rel: impl AsRef<Path>) -> Vec<String> {
        let p = self.path(rel);
        fs::read_dir(&p)
            .map(|rd| {
                rd.filter_map(|e| e.ok().map(|d| d.file_name().to_string_lossy().into_owned()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Write `content` to `rel` (workspace-relative), creating parent dirs.
    pub fn write(&self, rel: impl AsRef<Path>, content: &str) {
        let p = self.path(&rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)
                .unwrap_or_else(|e| panic!("create_dir_all for {}: {e}", p.display()));
        }
        fs::write(&p, content).unwrap_or_else(|e| panic!("write {}: {e}", p.display()));
    }

    /// Write `content` to `rel` inside the dotagents root dir, creating
    /// parent dirs as needed.
    pub fn write_in_root_dir(&self, rel: impl AsRef<Path>, content: &str) {
        let full = self.root_dir().join(&rel);
        self.write(full, content);
    }

    // ── command runner ────────────────────────────────────────────────────────

    /// Spawn the compiled binary with `args` from the workspace root.
    pub fn run(&self, args: &[&str]) -> CmdOutput {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_dotagents"));
        cmd.current_dir(self.root());
        for arg in args {
            cmd.arg(arg);
        }
        let out = cmd.output().expect("failed to execute dotagents binary");
        CmdOutput {
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            status: out.status,
        }
    }

    // ── convenience ───────────────────────────────────────────────────────────

    /// Run `init` then `deploy`, panicking if either fails.  Returns the
    /// deploy output.
    pub fn init_and_deploy(&self) -> CmdOutput {
        self.run(&["init"]).assert_success();
        let out = self.run(&["deploy"]);
        out.assert_success();
        out
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Command-output wrapper
// ─────────────────────────────────────────────────────────────────────────────

pub struct CmdOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: std::process::ExitStatus,
}

impl CmdOutput {
    pub fn success(&self) -> bool {
        self.status.success()
    }

    /// Assert the command exited successfully; panic with diagnostics otherwise.
    #[track_caller]
    pub fn assert_success(&self) -> &Self {
        assert!(
            self.success(),
            "expected success, got {}\nstdout:\n{}\nstderr:\n{}",
            self.status,
            self.stdout,
            self.stderr
        );
        self
    }

    /// Assert the command failed; panic with diagnostics otherwise.
    #[track_caller]
    pub fn assert_failure(&self) -> &Self {
        assert!(
            !self.success(),
            "expected failure, but command succeeded\nstdout:\n{}\nstderr:\n{}",
            self.stdout,
            self.stderr
        );
        self
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Config fixtures shared across deploy / workflow tests
// ─────────────────────────────────────────────────────────────────────────────

/// A `local.config.toml` with a skills *provider* section for `mycode` but
/// without "skills" in the `features` array.
///
/// The config validator only allows `commands`, `instructions`, and `mcp` as
/// feature names; listing "skills" would fail validation.  This fixture is
/// used to verify that an inert provider section does not break deployment
/// and that no skill output files are written.
pub const LOCAL_CONFIG_WITH_SKILL_PROVIDER_ONLY: &str = r#"schema = "https://dotagents.soorya-u.dev/schemas/config.schema.json"
features = [
    "commands",
    "instructions",
    "mcp",
]

[targets]
cli = []
ide = []
custom = ["mycode"]

[providers.custom.mycode.mcp]
template = "{{ dir.application }}/templates/mycode/mcp.hbs"
target = "{{ dir.workspace }}/.mycode/mcp.json"

[providers.custom.mycode.instructions]
template = "{{ dir.application }}/templates/mycode/instructions.hbs"
target = "{{ dir.workspace }}/.mycode/instructions.md"
variables = {agent_name = "Mycode"}

[providers.custom.mycode.commands]
template = "{{ dir.application }}/templates/mycode/command.hbs"
target = "{{ dir.workspace }}/.mycode/commands/{{ command.name }}.md"

[providers.custom.mycode.skills]
template = "{{ dir.application }}/templates/mycode/skill.hbs"
target = "{{ dir.workspace }}/.mycode/skills/{{ skill.name }}.md"
"#;

/// A `local.config.toml` where the `commands` provider feature is explicitly
/// disabled via `disabled = true`.  Instructions and MCP are still active.
pub const LOCAL_CONFIG_COMMANDS_DISABLED: &str = r#"schema = "https://dotagents.soorya-u.dev/schemas/config.schema.json"
features = [
    "commands",
    "instructions",
    "mcp",
]

[targets]
cli = []
ide = []
custom = ["mycode"]

[providers.custom.mycode.mcp]
template = "{{ dir.application }}/templates/mycode/mcp.hbs"
target = "{{ dir.workspace }}/.mycode/mcp.json"

[providers.custom.mycode.instructions]
template = "{{ dir.application }}/templates/mycode/instructions.hbs"
target = "{{ dir.workspace }}/.mycode/instructions.md"
variables = {agent_name = "Mycode"}

[providers.custom.mycode.commands]
template = "{{ dir.application }}/templates/mycode/command.hbs"
target = "{{ dir.workspace }}/.mycode/commands/{{ command.name }}.md"
disabled = true
"#;
