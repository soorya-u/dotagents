use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::utils::fs::{read_file, write_file};
use crate::utils::path::make_workspace_relative;
use crate::utils::tty::is_tty;

/// Controls how a feature's deployed paths are represented in .gitignore.
pub(crate) enum GitignoreScope {
    /// Write the exact deployed file path into .gitignore.
    File,
}

/// Represents how a deployed path should appear in .gitignore.
#[derive(Debug)]
pub(crate) enum GitignorePath {
    /// Write the exact file path (e.g. `.kilo/mcp.json`).
    File(PathBuf),
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

/// Remove the entire dotagents-managed fenced section from .gitignore.
pub(crate) fn clear_gitignore_fence(workspace_root: &Path) -> Result<()> {
    let gitignore_path = workspace_root.join(".gitignore");
    let content = read_gitignore(&gitignore_path)?;
    if content.is_empty() || !content.contains(FENCE_START) {
        return Ok(());
    }
    let new_content = remove_fence(&content);
    if content != new_content {
        write_file(&gitignore_path, &new_content)?;
    }
    Ok(())
}

/// Strip the fenced section and the blank line that preceded it from raw content.
fn remove_fence(content: &str) -> String {
    if !content.contains(FENCE_START) {
        return content.to_string();
    }

    let mut result: Vec<&str> = Vec::new();
    let mut in_fence = false;

    for line in content.lines() {
        if line == FENCE_START {
            in_fence = true;
            // Remove blank line that update_gitignore inserts before the fence
            if result.last() == Some(&"") {
                result.pop();
            }
            continue;
        }
        if in_fence {
            if line == FENCE_END {
                in_fence = false;
            }
            continue;
        }
        result.push(line);
    }

    // Trim trailing blank lines left behind by fence removal
    while result.last() == Some(&"") {
        result.pop();
    }

    if result.is_empty() {
        return String::new();
    }

    let mut out = result.join("\n");
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fenced_section_empty() {
        // empty content yields no paths
        let content = "";
        let paths = parse_fenced_section(content);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_parse_fenced_section_with_fence() {
        // paths inside the fence are extracted
        let content = "# BEGIN dotagents managed - do not edit manually\n.claude/commands/hello.md\nAGENTS.md\n# END dotagents managed";
        let paths = parse_fenced_section(content);
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(".claude/commands/hello.md"));
        assert!(paths.contains("AGENTS.md"));
    }

    #[test]
    fn test_parse_fenced_section_no_fence() {
        // content without fence yields no paths
        let content = "node_modules/\n.env";
        let paths = parse_fenced_section(content);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_update_gitignore_new_file() {
        // creates fenced section when none exists
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
        // appends fence when user content exists without one
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
        // adds new path to existing fence without touching others
        let content = "node_modules/\n# BEGIN dotagents managed - do not edit manually\n.claude/commands/hello.md\n# END dotagents managed";
        let new_paths = vec!["CLAUDE.md".to_string()];
        let result = update_gitignore(content, &new_paths);

        assert!(result.contains("node_modules/"));
        assert!(result.contains(".claude/commands/hello.md"));
        assert!(result.contains("CLAUDE.md"));
    }

    #[test]
    fn test_update_gitignore_no_duplicates() {
        // already-present path is not written twice
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
    fn test_gitignore_path_to_pattern_out_of_workspace() {
        // path outside workspace root returns None
        let root = PathBuf::from("/workspace");
        let entry = GitignorePath::File(PathBuf::from("/other/.kilo/mcp.json"));
        let pattern = gitignore_path_to_pattern(&entry, &root);
        assert!(pattern.is_none());
    }

    #[test]
    fn test_remove_fence_only_fence() {
        // content consisting only of a fence returns empty string
        let content = "# BEGIN dotagents managed - do not edit manually\n.claude/mcp.json\n# END dotagents managed\n";
        let result = remove_fence(content);
        assert_eq!(result, "");
    }

    #[test]
    fn test_remove_fence_with_preceding_content() {
        // blank line before fence is removed along with the fence
        let content = "node_modules/\n.env\n\n# BEGIN dotagents managed - do not edit manually\n.claude/mcp.json\n# END dotagents managed\n";
        let result = remove_fence(content);
        assert_eq!(result, "node_modules/\n.env\n");
    }

    #[test]
    fn test_remove_fence_no_fence_unchanged() {
        // content without a fence is returned unchanged
        let content = "node_modules/\n.env\n";
        let result = remove_fence(content);
        assert_eq!(result, content);
    }

    #[test]
    fn test_remove_fence_preserves_content_after_fence() {
        // user content after the fence is preserved
        let content = "# BEGIN dotagents managed - do not edit manually\n.claude/mcp.json\n# END dotagents managed\n.DS_Store\n";
        let result = remove_fence(content);
        assert_eq!(result, ".DS_Store\n");
    }

    #[test]
    fn test_clear_gitignore_fence_no_fence() {
        // no-op when .gitignore has no fence
        use std::fs;
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let gi = tmp.path().join(".gitignore");
        fs::write(&gi, "node_modules/\n").unwrap();
        clear_gitignore_fence(tmp.path()).unwrap();
        assert_eq!(fs::read_to_string(&gi).unwrap(), "node_modules/\n");
    }

    #[test]
    fn test_clear_gitignore_fence_removes_fence() {
        // removes fenced section and writes back
        use std::fs;
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let gi = tmp.path().join(".gitignore");
        fs::write(
            &gi,
            "node_modules/\n\n# BEGIN dotagents managed - do not edit manually\n.claude/mcp.json\n# END dotagents managed\n",
        )
        .unwrap();
        clear_gitignore_fence(tmp.path()).unwrap();
        let after = fs::read_to_string(&gi).unwrap();
        assert!(!after.contains(FENCE_START));
        assert!(!after.contains(".claude/mcp.json"));
        assert!(after.contains("node_modules/"));
    }

    #[test]
    fn test_clear_gitignore_fence_missing_file_is_noop() {
        // missing .gitignore does not produce an error
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        assert!(clear_gitignore_fence(tmp.path()).is_ok());
    }

    #[test]
    fn test_clear_gitignore_fence_only_fence_writes_empty() {
        // when entire file was the fence, result is empty string
        use std::fs;
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let gi = tmp.path().join(".gitignore");
        fs::write(
            &gi,
            "# BEGIN dotagents managed - do not edit manually\n.claude/mcp.json\n# END dotagents managed\n",
        )
        .unwrap();
        clear_gitignore_fence(tmp.path()).unwrap();
        let after = fs::read_to_string(&gi).unwrap();
        assert_eq!(after, "");
    }
}
