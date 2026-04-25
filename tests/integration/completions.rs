//! Smoke tests for the `gen-completions` command.

use super::TestWorkspace;
use std::fs;

fn completions_dir(ws: &TestWorkspace) -> std::path::PathBuf {
    let dir = ws.root().join("completions");
    fs::create_dir_all(&dir).expect("failed to create completions dir");
    dir
}

#[test]
fn gen_completions_bash() {
    let ws = TestWorkspace::new();
    let dir = completions_dir(&ws);
    ws.run_command(&[
        "gen-completions",
        "--shell",
        "bash",
        "--to",
        dir.to_str().unwrap(),
    ])
    .assert_success();
    let f = dir.join("dotagents.bash");
    assert!(f.exists(), "bash completion file should be generated");
    assert!(!fs::read_to_string(f).unwrap().is_empty());
}

#[test]
fn gen_completions_zsh() {
    let ws = TestWorkspace::new();
    let dir = completions_dir(&ws);
    ws.run_command(&[
        "gen-completions",
        "--shell",
        "zsh",
        "--to",
        dir.to_str().unwrap(),
    ])
    .assert_success();
    let f = dir.join("_dotagents");
    assert!(f.exists(), "zsh completion file should be generated");
    assert!(!fs::read_to_string(f).unwrap().is_empty());
}

#[test]
fn gen_completions_fish() {
    let ws = TestWorkspace::new();
    let dir = completions_dir(&ws);
    ws.run_command(&[
        "gen-completions",
        "--shell",
        "fish",
        "--to",
        dir.to_str().unwrap(),
    ])
    .assert_success();
    let f = dir.join("dotagents.fish");
    assert!(f.exists(), "fish completion file should be generated");
    assert!(!fs::read_to_string(f).unwrap().is_empty());
}

#[test]
fn gen_completions_powershell() {
    let ws = TestWorkspace::new();
    let dir = completions_dir(&ws);
    ws.run_command(&[
        "gen-completions",
        "--shell",
        "powershell",
        "--to",
        dir.to_str().unwrap(),
    ])
    .assert_success();
    let f = dir.join("_dotagents.ps1");
    assert!(f.exists(), "powershell completion file should be generated");
    assert!(!fs::read_to_string(f).unwrap().is_empty());
}

#[test]
fn gen_completions_elvish() {
    let ws = TestWorkspace::new();
    let dir = completions_dir(&ws);
    ws.run_command(&[
        "gen-completions",
        "--shell",
        "elvish",
        "--to",
        dir.to_str().unwrap(),
    ])
    .assert_success();
    let f = dir.join("dotagents.elv");
    assert!(f.exists(), "elvish completion file should be generated");
    assert!(!fs::read_to_string(f).unwrap().is_empty());
}
