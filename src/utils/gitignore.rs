use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::constants::file::{FENCE_END, FENCE_START};
use crate::utils::fs::{read_file, write_file};
use crate::utils::path::make_workspace_relative;

fn read_gitignore(path: &PathBuf) -> Result<String> {
    if path.exists() {
        read_file(path)
    } else {
        Ok(String::new())
    }
}

/// Collapse generated paths into directory patterns where possible.
pub(crate) fn collapse_paths(paths: &[String], workspace_root: &Path) -> Vec<String> {
    if paths.is_empty() {
        return vec![];
    }

    let generated: HashSet<PathBuf> = paths.iter().map(PathBuf::from).collect();

    let mut root_files: Vec<String> = Vec::new();
    let mut dir_files: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

    for p in &generated {
        if let Some(parent) = p.parent() {
            if parent.as_os_str().is_empty() {
                root_files.push(p.to_string_lossy().to_string());
            } else {
                dir_files
                    .entry(parent.to_path_buf())
                    .or_default()
                    .push(p.clone());
            }
        } else {
            root_files.push(p.to_string_lossy().to_string());
        }
    }

    let mut collapsible: HashMap<PathBuf, bool> = HashMap::new();
    let mut collapsed_dirs: HashSet<PathBuf> = HashSet::new();

    is_collapsible_dir_cached(workspace_root, &generated, &mut collapsible, &dir_files);

    for dir in dir_files.keys() {
        find_highest_collapsible(
            dir,
            workspace_root,
            &generated,
            &mut collapsible,
            &dir_files,
            &mut collapsed_dirs,
        );
    }

    let mut result: Vec<String> = root_files;

    let covered: HashSet<&PathBuf> = collapsed_dirs.iter().collect();
    for dir in &collapsed_dirs {
        let mut dominated = false;
        let mut ancestor = dir.parent();
        while let Some(a) = ancestor {
            if a.as_os_str().is_empty() {
                break;
            }
            if covered.contains(&a.to_path_buf()) {
                dominated = true;
                break;
            }
            ancestor = a.parent();
        }
        if !dominated {
            let mut s = dir.to_string_lossy().to_string();
            if !s.ends_with('/') {
                s.push('/');
            }
            result.push(s);
        }
    }

    for p in &generated {
        if let Some(parent) = p.parent() {
            if parent.as_os_str().is_empty() {
                continue;
            }
            let mut is_covered = false;
            let mut ancestor = Some(parent.to_path_buf());
            while let Some(a) = ancestor {
                if a.as_os_str().is_empty() {
                    break;
                }
                if collapsed_dirs.contains(&a) {
                    is_covered = true;
                    break;
                }
                ancestor = a.parent().map(|p| p.to_path_buf());
            }
            if !is_covered {
                result.push(p.to_string_lossy().to_string());
            }
        }
    }

    result.sort();
    result
}

fn is_collapsible_dir_cached(
    workspace_root: &Path,
    generated: &HashSet<PathBuf>,
    cache: &mut HashMap<PathBuf, bool>,
    dir_files: &HashMap<PathBuf, Vec<PathBuf>>,
) {
    for dir in dir_files.keys() {
        check_collapsible(dir, workspace_root, generated, cache, dir_files);
    }
}

fn check_collapsible(
    rel_dir: &Path,
    workspace_root: &Path,
    generated: &HashSet<PathBuf>,
    cache: &mut HashMap<PathBuf, bool>,
    dir_files: &HashMap<PathBuf, Vec<PathBuf>>,
) -> bool {
    if let Some(&result) = cache.get(rel_dir) {
        return result;
    }

    let abs_dir = workspace_root.join(rel_dir);
    let entries = match std::fs::read_dir(&abs_dir) {
        Ok(e) => e,
        Err(_) => {
            cache.insert(rel_dir.to_path_buf(), false);
            return false;
        }
    };

    let mut collapsible = true;
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => {
                collapsible = false;
                break;
            }
        };
        let name = entry.file_name();
        let child_rel = rel_dir.join(&name);

        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_dir {
            if dir_files.contains_key(&child_rel)
                || has_generated_descendants(&child_rel, generated)
            {
                if !check_collapsible(&child_rel, workspace_root, generated, cache, dir_files) {
                    collapsible = false;
                    break;
                }
            } else {
                collapsible = false;
                break;
            }
        } else if !generated.contains(&child_rel) {
            collapsible = false;
            break;
        }
    }

    cache.insert(rel_dir.to_path_buf(), collapsible);
    collapsible
}

fn has_generated_descendants(dir: &Path, generated: &HashSet<PathBuf>) -> bool {
    generated.iter().any(|p| p.starts_with(dir))
}

fn find_highest_collapsible(
    dir: &Path,
    workspace_root: &Path,
    generated: &HashSet<PathBuf>,
    cache: &mut HashMap<PathBuf, bool>,
    dir_files: &HashMap<PathBuf, Vec<PathBuf>>,
    collapsed: &mut HashSet<PathBuf>,
) {
    if !cache.get(dir).copied().unwrap_or(false) {
        return;
    }

    let mut highest = dir.to_path_buf();
    let mut ancestor = dir.parent();
    while let Some(a) = ancestor {
        if a.as_os_str().is_empty() {
            break;
        }
        if check_collapsible(a, workspace_root, generated, cache, dir_files) {
            highest = a.to_path_buf();
        } else {
            break;
        }
        ancestor = a.parent();
    }

    collapsed.insert(highest);
}

/// Rewrite the fenced section with the given patterns, replacing any existing fence content.
fn rewrite_fence(content: &str, patterns: &[String]) -> String {
    let mut sorted = patterns.to_vec();
    sorted.sort();

    let fence_start_idx = content.find(FENCE_START);
    let fence_end_idx = content.find(FENCE_END);

    let mut fence_body = String::from(FENCE_START);
    fence_body.push('\n');
    for p in &sorted {
        fence_body.push_str(p);
        fence_body.push('\n');
    }
    fence_body.push_str(FENCE_END);

    match (fence_start_idx, fence_end_idx) {
        (Some(start), Some(end)) => {
            let before = &content[..start];
            let before = before
                .strip_suffix("\n\n")
                .or_else(|| before.strip_suffix('\n'))
                .unwrap_or(before);
            let after_end = end + FENCE_END.len();
            let after = &content[after_end..];
            format!("{before}\n{fence_body}{after}")
        }
        _ => {
            let mut result = content.to_string();
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
            result.push('\n');
            result.push_str(&fence_body);
            result.push('\n');
            result
        }
    }
}

/// Rebuild the `.gitignore` fence from all cached target paths, using the collapse algorithm.
pub(crate) fn rebuild_fence_from_cache(
    cache_targets: &[PathBuf],
    workspace_root: &Path,
) -> Result<()> {
    let relative_paths: Vec<String> = cache_targets
        .iter()
        .filter_map(|p| make_workspace_relative(p, workspace_root))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    if relative_paths.is_empty() {
        return clear_gitignore_fence(workspace_root);
    }

    let patterns = collapse_paths(&relative_paths, workspace_root);

    let gitignore_path = workspace_root.join(".gitignore");
    let current_content = read_gitignore(&gitignore_path)?;
    let new_content = rewrite_fence(&current_content, &patterns);

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
    use crate::constants::file::{FENCE_END, FENCE_START};

    #[test]
    fn rewrite_fence_creates_fence_in_empty_content() {
        let patterns = vec!["CLAUDE.md".to_string(), ".claude/commands/".to_string()];
        let result = rewrite_fence("", &patterns);
        assert!(result.contains(FENCE_START));
        assert!(result.contains(FENCE_END));
        assert!(result.contains(".claude/commands/"));
        assert!(result.contains("CLAUDE.md"));
    }

    #[test]
    fn rewrite_fence_appends_to_existing_content() {
        let result = rewrite_fence("node_modules/\n.env", &["CLAUDE.md".to_string()]);
        assert!(result.contains("node_modules/"));
        assert!(result.contains(".env"));
        assert!(result.contains(FENCE_START));
        assert!(result.contains("CLAUDE.md"));
    }

    #[test]
    fn rewrite_fence_replaces_existing_fence() {
        let content =
            "node_modules/\n#region dotagents\n.claude/commands/hello.md\n#endregion dotagents";
        let result = rewrite_fence(content, &["CLAUDE.md".to_string()]);
        assert!(result.contains("node_modules/"));
        assert!(result.contains("CLAUDE.md"));
        assert!(!result.contains(".claude/commands/hello.md"));
    }

    #[test]
    fn rewrite_fence_preserves_user_content_around_fence() {
        let content = "*.log\n\n#region dotagents\nCLAUDE.md\n#endregion dotagents\n\n.DS_Store\n";
        let result = rewrite_fence(content, &["AGENTS.md".to_string()]);
        assert!(result.contains("*.log"));
        assert!(result.contains(".DS_Store"));
        assert!(result.contains("AGENTS.md"));
        assert!(!result.contains("CLAUDE.md"));
    }

    #[test]
    fn rebuild_fence_creates_fence_for_new_gitignore() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let cmd_dir = tmp.path().join(".claude").join("commands");
        std::fs::create_dir_all(&cmd_dir).unwrap();
        std::fs::write(cmd_dir.join("a.md"), "").unwrap();

        let targets = vec![tmp.path().join(".claude/commands/a.md")];
        rebuild_fence_from_cache(&targets, tmp.path()).unwrap();

        let content = std::fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
        assert!(content.contains(FENCE_START));
        assert!(content.contains(FENCE_END));
        assert!(content.contains(".claude/"));
    }

    #[test]
    fn rebuild_fence_rewrites_existing_fence_with_collapsed_patterns() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let cmd_dir = tmp.path().join(".mycode").join("commands");
        std::fs::create_dir_all(&cmd_dir).unwrap();
        std::fs::write(cmd_dir.join("a.md"), "").unwrap();
        std::fs::write(cmd_dir.join("b.md"), "").unwrap();
        std::fs::write(
            tmp.path().join(".gitignore"),
            "#region dotagents\n.mycode/commands/a.md\n.mycode/commands/b.md\n#endregion dotagents\n",
        ).unwrap();

        let targets = vec![
            tmp.path().join(".mycode/commands/a.md"),
            tmp.path().join(".mycode/commands/b.md"),
        ];
        rebuild_fence_from_cache(&targets, tmp.path()).unwrap();

        let content = std::fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
        assert!(
            content.contains(".mycode/"),
            "should collapse to directory pattern; got:\n{content}"
        );
        assert!(
            !content.contains(".mycode/commands/a.md"),
            "individual paths should not appear; got:\n{content}"
        );
    }

    #[test]
    fn rebuild_fence_preserves_user_content() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("CLAUDE.md"), "").unwrap();
        std::fs::write(tmp.path().join(".gitignore"), "*.log\n.DS_Store\n").unwrap();

        let targets = vec![tmp.path().join("CLAUDE.md")];
        rebuild_fence_from_cache(&targets, tmp.path()).unwrap();

        let content = std::fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
        assert!(content.contains("*.log"));
        assert!(content.contains(".DS_Store"));
        assert!(content.contains("CLAUDE.md"));
    }

    #[test]
    fn rebuild_fence_skips_write_when_unchanged() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("CLAUDE.md"), "").unwrap();
        let initial = "\n#region dotagents\nCLAUDE.md\n#endregion dotagents\n";
        std::fs::write(tmp.path().join(".gitignore"), initial).unwrap();

        let targets = vec![tmp.path().join("CLAUDE.md")];
        rebuild_fence_from_cache(&targets, tmp.path()).unwrap();

        let content = std::fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
        assert_eq!(content, initial);
    }

    #[test]
    fn rebuild_fence_clears_fence_when_no_targets() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join(".gitignore"),
            "*.log\n\n#region dotagents\nCLAUDE.md\n#endregion dotagents\n",
        )
        .unwrap();

        rebuild_fence_from_cache(&[], tmp.path()).unwrap();

        let content = std::fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
        assert!(!content.contains(FENCE_START));
        assert!(content.contains("*.log"));
    }

    #[test]
    fn rebuild_fence_handles_missing_gitignore() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("CLAUDE.md"), "").unwrap();

        let targets = vec![tmp.path().join("CLAUDE.md")];
        rebuild_fence_from_cache(&targets, tmp.path()).unwrap();

        assert!(tmp.path().join(".gitignore").exists());
        let content = std::fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
        assert!(content.contains("CLAUDE.md"));
    }

    #[test]
    fn test_remove_fence_only_fence() {
        let content = "#region dotagents\n.claude/mcp.json\n#endregion dotagents\n";
        let result = remove_fence(content);
        assert_eq!(result, "");
    }

    #[test]
    fn test_remove_fence_with_preceding_content() {
        let content =
            "node_modules/\n.env\n\n#region dotagents\n.claude/mcp.json\n#endregion dotagents\n";
        let result = remove_fence(content);
        assert_eq!(result, "node_modules/\n.env\n");
    }

    #[test]
    fn test_remove_fence_no_fence_unchanged() {
        let content = "node_modules/\n.env\n";
        let result = remove_fence(content);
        assert_eq!(result, content);
    }

    #[test]
    fn test_remove_fence_preserves_content_after_fence() {
        let content = "#region dotagents\n.claude/mcp.json\n#endregion dotagents\n.DS_Store\n";
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
            "node_modules/\n\n#region dotagents\n.claude/mcp.json\n#endregion dotagents\n",
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
            "#region dotagents\n.claude/mcp.json\n#endregion dotagents\n",
        )
        .unwrap();
        clear_gitignore_fence(tmp.path()).unwrap();
        let after = fs::read_to_string(&gi).unwrap();
        assert_eq!(after, "");
    }

    #[test]
    fn collapse_paths_empty_input() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let result = collapse_paths(&[], tmp.path());
        assert!(result.is_empty());
    }

    #[test]
    fn collapse_paths_root_files_stay_individual() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("CLAUDE.md"), "").unwrap();
        std::fs::write(tmp.path().join("AGENTS.md"), "").unwrap();
        let paths = vec!["CLAUDE.md".to_string(), "AGENTS.md".to_string()];
        let result = collapse_paths(&paths, tmp.path());
        assert_eq!(result, vec!["AGENTS.md", "CLAUDE.md"]);
    }

    #[test]
    fn collapse_paths_all_files_in_dir_collapses() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let cmd_dir = tmp.path().join(".claude").join("commands");
        std::fs::create_dir_all(&cmd_dir).unwrap();
        std::fs::write(cmd_dir.join("a.md"), "").unwrap();
        std::fs::write(cmd_dir.join("b.md"), "").unwrap();
        let paths = vec![
            ".claude/commands/a.md".to_string(),
            ".claude/commands/b.md".to_string(),
        ];
        let result = collapse_paths(&paths, tmp.path());
        // .claude/ only contains commands/ (all generated), so it collapses to .claude/
        assert_eq!(result, vec![".claude/"]);
    }

    #[test]
    fn collapse_paths_non_generated_file_prevents_collapse() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let cmd_dir = tmp.path().join(".claude").join("commands");
        std::fs::create_dir_all(&cmd_dir).unwrap();
        std::fs::write(cmd_dir.join("a.md"), "").unwrap();
        std::fs::write(cmd_dir.join("b.md"), "").unwrap();
        std::fs::write(cmd_dir.join("custom.md"), "").unwrap();
        let paths = vec![
            ".claude/commands/a.md".to_string(),
            ".claude/commands/b.md".to_string(),
        ];
        let result = collapse_paths(&paths, tmp.path());
        assert!(result.contains(&".claude/commands/a.md".to_string()));
        assert!(result.contains(&".claude/commands/b.md".to_string()));
        assert!(!result.contains(&".claude/commands/".to_string()));
    }

    #[test]
    fn collapse_paths_nested_collapse() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let cmd_dir = tmp.path().join(".opencode").join("commands");
        let skill_dir = tmp.path().join(".opencode").join("skills").join("my-skill");
        std::fs::create_dir_all(&cmd_dir).unwrap();
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(cmd_dir.join("a.md"), "").unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "").unwrap();
        let paths = vec![
            ".opencode/commands/a.md".to_string(),
            ".opencode/skills/my-skill/SKILL.md".to_string(),
        ];
        let result = collapse_paths(&paths, tmp.path());
        assert_eq!(result, vec![".opencode/"]);
    }

    #[test]
    fn collapse_paths_mixed_collapsible_and_non_generated() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let cmd_dir = tmp.path().join(".claude").join("commands");
        let skill_dir = tmp.path().join(".claude").join("skills").join("s1");
        std::fs::create_dir_all(&cmd_dir).unwrap();
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(cmd_dir.join("a.md"), "").unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "").unwrap();
        // non-generated file in .claude/ prevents full collapse
        std::fs::write(tmp.path().join(".claude").join("settings.json"), "").unwrap();
        let paths = vec![
            ".claude/commands/a.md".to_string(),
            ".claude/skills/s1/SKILL.md".to_string(),
        ];
        let result = collapse_paths(&paths, tmp.path());
        assert!(result.contains(&".claude/commands/".to_string()));
        assert!(result.contains(&".claude/skills/".to_string()));
        assert!(!result.contains(&".claude/".to_string()));
    }
}
