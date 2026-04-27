use std::collections::HashSet;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::utils::fs::read_file;
use crate::utils::fs::write_file;
use crate::utils::path::make_workspace_relative;

/// Controls how a feature's deployed paths are represented in .gitignore.
pub(crate) enum GitignoreScope {
    /// Write the exact deployed file path into .gitignore.
    File,
    /// Write the parent directory as a glob pattern (`dir/*`) into .gitignore.
    Directory,
}

/// Represents how a deployed path should appear in .gitignore.
pub(crate) enum GitignorePath {
    /// Write the exact file path (e.g. `.kilo/mcp.json`).
    File(PathBuf),
    /// Write a glob covering the whole directory (e.g. `.kilo/commands/*`).
    Directory(PathBuf),
}

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

/// Convert a `GitignorePath` to its workspace-relative gitignore pattern string.
pub(crate) fn gitignore_path_to_pattern(
    entry: &GitignorePath,
    workspace_root: &Path,
) -> Option<String> {
    match entry {
        GitignorePath::File(p) => make_workspace_relative(p, workspace_root),
        GitignorePath::Directory(p) => {
            make_workspace_relative(p, workspace_root).map(|s| format!("{s}/*"))
        }
    }
}

/// Orchestrate read → update → write; skips write if nothing changed.
pub(crate) fn write_gitignore(workspace_root: &Path, new_paths: &[GitignorePath]) -> Result<()> {
    let gitignore_path = workspace_root.join(".gitignore");
    let relative_paths: Vec<String> = new_paths
        .iter()
        .filter_map(|entry| gitignore_path_to_pattern(entry, workspace_root))
        .collect::<HashSet<_>>()
        .into_iter()
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

/// Detects whether both stdin and stdout are interactive terminals.
pub(crate) fn is_tty() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
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

    #[test]
    fn test_gitignore_path_to_pattern_file() {
        // File variant produces exact workspace-relative path
        let root = PathBuf::from("/workspace");
        let entry = GitignorePath::File(PathBuf::from("/workspace/.kilo/mcp.json"));
        let pattern = gitignore_path_to_pattern(&entry, &root).unwrap();
        assert_eq!(pattern, ".kilo/mcp.json");
    }

    #[test]
    fn test_gitignore_path_to_pattern_directory() {
        // Directory variant appends /* to the workspace-relative path
        let root = PathBuf::from("/workspace");
        let entry = GitignorePath::Directory(PathBuf::from("/workspace/.kilo/commands"));
        let pattern = gitignore_path_to_pattern(&entry, &root).unwrap();
        assert_eq!(pattern, ".kilo/commands/*");
    }

    #[test]
    fn test_gitignore_path_to_pattern_out_of_workspace() {
        // path outside workspace root returns None
        let root = PathBuf::from("/workspace");
        let entry = GitignorePath::File(PathBuf::from("/other/.kilo/mcp.json"));
        let pattern = gitignore_path_to_pattern(&entry, &root);
        assert!(pattern.is_none());
    }

    #[test]
    fn test_update_gitignore_with_directory_glob() {
        // directory glob patterns are written with /* suffix
        let content = "";
        let new_paths = vec![".kilo/commands/*".to_string(), ".kilo/mcp.json".to_string()];
        let result = update_gitignore(content, &new_paths);
        assert!(result.contains(".kilo/commands/*"));
        assert!(result.contains(".kilo/mcp.json"));
    }

    #[test]
    fn test_update_gitignore_directory_no_duplicates() {
        // same directory glob added twice appears only once
        let content = "# BEGIN dotagents managed - do not edit manually\n.kilo/commands/*\n# END dotagents managed";
        let new_paths = vec![".kilo/commands/*".to_string()];
        let result = update_gitignore(content, &new_paths);
        assert_eq!(result.matches(".kilo/commands/*").count(), 1);
    }

    #[test]
    fn test_update_gitignore_file_and_directory_coexist() {
        // File and Directory patterns can coexist in the fenced section
        let content = "";
        let new_paths = vec![
            ".kilo/commands/*".to_string(),
            ".kilo/mcp.json".to_string(),
            ".windsurf/workflows/*".to_string(),
        ];
        let result = update_gitignore(content, &new_paths);
        assert!(result.contains(".kilo/commands/*"));
        assert!(result.contains(".kilo/mcp.json"));
        assert!(result.contains(".windsurf/workflows/*"));
    }
}
