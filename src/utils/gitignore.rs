use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use crate::utils::fs::read_file;
use crate::utils::fs::write_file;
use crate::utils::path::make_workspace_relative;

const FENCE_START: &str = "# BEGIN dotagents managed - do not edit manually";
const FENCE_END: &str = "# END dotagents managed";

/// Read .gitignore content, returning empty string if file doesn't exist.
pub(crate) fn read_gitignore(path: &PathBuf) -> Result<String> {
    if path.exists() {
        read_file(path)
    } else {
        Ok(String::new())
    }
}

/// Extract paths currently inside the dotagents fenced section.
pub(crate) fn parse_fenced_section(content: &str) -> HashSet<String> {
    let mut paths = HashSet::new();
    let mut in_fence = false;

    for line in content.lines() {
        if line == FENCE_START {
            in_fence = true;
            continue;
        }
        if line == FENCE_END {
            in_fence = false;
            continue;
        }
        if in_fence && !line.is_empty() && !line.starts_with('#') {
            paths.insert(line.to_string());
        }
    }

    paths
}

/// Update .gitignore content with new paths, creating/updating the fenced section.
pub(crate) fn update_gitignore(content: &str, new_paths: &[String]) -> String {
    let existing_paths = parse_fenced_section(content);
    let mut to_add: Vec<String> = new_paths
        .iter()
        .filter(|p| !existing_paths.contains(*p))
        .cloned()
        .collect();

    if to_add.is_empty() {
        return content.to_string();
    }

    to_add.sort();

    let fence_start_idx = content.find(FENCE_START);
    let fence_end_idx = content.find(FENCE_END);

    match (fence_start_idx, fence_end_idx) {
        (Some(start), Some(end)) => {
            let before = &content[..start];
            let after_end = end + FENCE_END.len();
            let after = &content[after_end..];

            let mut new_fence = String::from(FENCE_START);
            new_fence.push('\n');

            for path in existing_paths.iter() {
                new_fence.push_str(path);
                new_fence.push('\n');
            }
            for path in to_add {
                new_fence.push_str(&path);
                new_fence.push('\n');
            }

            new_fence.push_str(FENCE_END);

            format!("{}{}{}", before.trim_end(), "\n", new_fence) + after
        }
        _ => {
            let mut result = content.to_string();
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
            result.push('\n');
            result.push_str(FENCE_START);
            result.push('\n');

            for path in existing_paths.iter() {
                result.push_str(path);
                result.push('\n');
            }
            for path in to_add {
                result.push_str(&path);
                result.push('\n');
            }

            result.push_str(FENCE_END);
            result.push('\n');
            result
        }
    }
}

/// Orchestrate read → update → write; skips write if nothing changed.
pub(crate) fn write_gitignore(workspace_root: &Path, new_paths: &[PathBuf]) -> Result<()> {
    let gitignore_path = workspace_root.join(".gitignore");
    let relative_paths: Vec<String> = new_paths
        .iter()
        .filter_map(|p| make_workspace_relative(p, workspace_root))
        .collect();

    if relative_paths.is_empty() {
        return Ok(());
    }

    let current_content = read_gitignore(&gitignore_path)?;
    let new_content = update_gitignore(&current_content, &relative_paths);

    if current_content != new_content {
        write_file(&gitignore_path, &new_content)?;
    }

    Ok(())
}

/// Detect whether stdin is an interactive terminal.
pub(crate) fn is_tty() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal()
}

/// Prompt the user to confirm adding deployed paths to .gitignore.
///
/// Returns `false` immediately in non-TTY environments.
pub(crate) fn prompt_gitignore_update(new_path_count: usize) -> bool {
    if !is_tty() {
        return false;
    }

    print!(
        "Add {} deployed path(s) to .gitignore? [y/N]: ",
        new_path_count
    );
    std::io::stdout().flush().ok();

    if enable_raw_mode().is_err() {
        return false;
    }

    let result = loop {
        match event::read() {
            Ok(Event::Key(key)) => {
                break matches!(key.code, KeyCode::Char('y') | KeyCode::Char('Y'));
            }
            Ok(_) => continue,
            Err(_) => break false,
        }
    };

    disable_raw_mode().ok();
    println!();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fenced_section_empty() {
        let content = "";
        let paths = parse_fenced_section(content);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_parse_fenced_section_with_fence() {
        let content = "# BEGIN dotagents managed - do not edit manually\n.claude/commands/hello.md\nAGENTS.md\n# END dotagents managed";
        let paths = parse_fenced_section(content);
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(".claude/commands/hello.md"));
        assert!(paths.contains("AGENTS.md"));
    }

    #[test]
    fn test_parse_fenced_section_no_fence() {
        let content = "node_modules/\n.env";
        let paths = parse_fenced_section(content);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_update_gitignore_new_file() {
        let content = "";
        let new_paths = vec![
            ".claude/commands/hello.md".to_string(),
            "CLAUDE.md".to_string(),
        ];
        let result = update_gitignore(content, &new_paths);

        assert!(result.contains(FENCE_START));
        assert!(result.contains(FENCE_END));
        assert!(result.contains(".claude/commands/hello.md"));
        assert!(result.contains("CLAUDE.md"));
    }

    #[test]
    fn test_update_gitignore_existing_no_fence() {
        let content = "node_modules/\n.env";
        let new_paths = vec!["CLAUDE.md".to_string()];
        let result = update_gitignore(content, &new_paths);

        assert!(result.contains("node_modules/"));
        assert!(result.contains(".env"));
        assert!(result.contains(FENCE_START));
        assert!(result.contains("CLAUDE.md"));
    }

    #[test]
    fn test_update_gitignore_existing_with_fence() {
        let content = "node_modules/\n# BEGIN dotagents managed - do not edit manually\n.claude/commands/hello.md\n# END dotagents managed";
        let new_paths = vec!["CLAUDE.md".to_string()];
        let result = update_gitignore(content, &new_paths);

        assert!(result.contains("node_modules/"));
        assert!(result.contains(".claude/commands/hello.md"));
        assert!(result.contains("CLAUDE.md"));
    }

    #[test]
    fn test_update_gitignore_no_duplicates() {
        let content =
            "# BEGIN dotagents managed - do not edit manually\nCLAUDE.md\n# END dotagents managed";
        let new_paths = vec!["CLAUDE.md".to_string()];
        let result = update_gitignore(content, &new_paths);

        let fence_count = result.matches("CLAUDE.md").count();
        assert_eq!(fence_count, 1, "CLAUDE.md should appear only once");
    }

    #[test]
    fn test_update_gitignore_user_content_preserved() {
        // user content before and after fence is preserved verbatim
        let content = "*.log\n\n# BEGIN dotagents managed - do not edit manually\nCLAUDE.md\n# END dotagents managed\n\n.DS_Store\n";
        let new_paths = vec!["AGENTS.md".to_string()];
        let result = update_gitignore(content, &new_paths);

        assert!(result.contains("*.log"));
        assert!(result.contains(".DS_Store"));
        assert!(result.contains("CLAUDE.md"));
        assert!(result.contains("AGENTS.md"));
    }

    #[test]
    fn test_is_tty_returns_bool() {
        // helper returns a bool without panicking
        let _result: bool = is_tty();
    }
}
