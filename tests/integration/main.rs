//! Integration tests for the dotagents CLI.
//!
//! These are coarser-grained "smoke" tests that run the compiled binary and
//! check exit codes and basic file existence.  For detailed content and
//! variable-interpolation tests see the `e2e` suite.
//!
//! Suite layout
//! ────────────
//!  init         – `init` command creates expected scaffolding
//!  deploy       – `deploy` command produces output after init
//!  completions  – `gen-completions` for all supported shells
//!  flags        – global --verbose / --quiet flags
//!
//! Run the whole suite:   cargo test --test integration
//! Run one test:          cargo test --test integration <test_name>

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

mod completions;
mod deploy;
mod flags;
mod init;
mod skills;

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
