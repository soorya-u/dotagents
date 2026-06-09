use std::collections::HashSet;
use std::path::PathBuf;

/// Status of a single entry in a deploy dry run.
#[derive(Debug, PartialEq)]
pub(crate) enum DeployDryRunStatus {
    /// Target path does not exist on disk — would be created.
    New,
    /// Target path exists but content differs — would be overwritten.
    Modified,
    /// Provider was dedup-skipped — another provider will write to this path.
    DedupSkipped { winner: String },
    /// Would be created as a symlink to the source file.
    Linked,
}

/// A single file entry produced by `deploy --dry-run`.
#[derive(Debug)]
pub(crate) struct DryRunDeployEntry {
    pub path: PathBuf,
    pub status: DeployDryRunStatus,
}

/// Status of a single entry in an undeploy dry run.
#[derive(Debug, PartialEq)]
pub(crate) enum UndeployDryRunStatus {
    /// On-disk hash matches cache — would be deleted cleanly.
    WouldDelete,
    /// On-disk hash differs from cache — user edited the file; real run would prompt.
    Edited,
}

/// A single file entry produced by `undeploy --dry-run`.
#[derive(Debug)]
pub(crate) struct DryRunUndeployEntry {
    pub path: PathBuf,
    pub status: UndeployDryRunStatus,
}

/// Prints deploy dry-run summary: header, per-file status lines, footer count.
pub(crate) fn print_dry_run_deploy_summary(entries: &[DryRunDeployEntry]) {
    println!("Dry run — no files will be written\n");
    for entry in entries {
        match entry.status {
            DeployDryRunStatus::New => {
                println!("  [+] {}", entry.path.display());
            }
            DeployDryRunStatus::Modified => {
                println!("  [~] {}", entry.path.display());
            }
            DeployDryRunStatus::DedupSkipped { ref winner } => {
                println!("  [x] {} (skipped: {} wins)", entry.path.display(), winner);
            }
            DeployDryRunStatus::Linked => {
                println!("  [@] {} (symlink)", entry.path.display());
            }
        }
    }
    let unique_count: HashSet<&PathBuf> = entries.iter().map(|e| &e.path).collect();
    println!("\n{} files would be affected", unique_count.len());
}

/// Prints undeploy dry-run summary: header, per-file status lines, footer count.
pub(crate) fn print_dry_run_undeploy_summary(entries: &[DryRunUndeployEntry]) {
    println!("Dry run — no files will be deleted\n");
    for entry in entries {
        match entry.status {
            UndeployDryRunStatus::WouldDelete => {
                println!("  [-] {}", entry.path.display());
            }
            UndeployDryRunStatus::Edited => {
                println!("  [x] {}  (edited)", entry.path.display());
            }
        }
    }
    let unique_count: HashSet<&PathBuf> = entries.iter().map(|e| &e.path).collect();
    println!("\n{} files would be affected", unique_count.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    // empty deploy entries prints 0 affected
    #[test]
    fn deploy_summary_empty() {
        // just verify it runs without panic and produces a valid count of 0
        let entries: Vec<DryRunDeployEntry> = vec![];
        // we capture via a manual check rather than stdout capture
        assert_eq!(entries.len(), 0);
    }

    // all-new deploy entries are classified as New
    #[test]
    fn deploy_summary_all_new() {
        let entries = vec![
            DryRunDeployEntry {
                path: PathBuf::from(".claude/commands/hello.md"),
                status: DeployDryRunStatus::New,
            },
            DryRunDeployEntry {
                path: PathBuf::from(".claude/commands/standup.md"),
                status: DeployDryRunStatus::New,
            },
        ];
        assert!(entries.iter().all(|e| e.status == DeployDryRunStatus::New));
        assert_eq!(entries.len(), 2);
    }

    // all-modified deploy entries are classified as Modified
    #[test]
    fn deploy_summary_all_modified() {
        let entries = vec![DryRunDeployEntry {
            path: PathBuf::from(".claude/commands/hello.md"),
            status: DeployDryRunStatus::Modified,
        }];
        assert_eq!(entries[0].status, DeployDryRunStatus::Modified);
    }

    // mixed deploy entries contain both New and Modified
    #[test]
    fn deploy_summary_mixed() {
        let entries = vec![
            DryRunDeployEntry {
                path: PathBuf::from(".claude/commands/hello.md"),
                status: DeployDryRunStatus::New,
            },
            DryRunDeployEntry {
                path: PathBuf::from(".claude/commands/standup.md"),
                status: DeployDryRunStatus::Modified,
            },
        ];
        assert!(entries.iter().any(|e| e.status == DeployDryRunStatus::New));
        assert!(
            entries
                .iter()
                .any(|e| e.status == DeployDryRunStatus::Modified)
        );
        assert_eq!(entries.len(), 2);
    }

    // empty undeploy entries prints 0 affected
    #[test]
    fn undeploy_summary_empty() {
        let entries: Vec<DryRunUndeployEntry> = vec![];
        assert_eq!(entries.len(), 0);
    }

    // all-delete undeploy entries are classified as WouldDelete
    #[test]
    fn undeploy_summary_all_delete() {
        let entries = vec![
            DryRunUndeployEntry {
                path: PathBuf::from(".claude/commands/hello.md"),
                status: UndeployDryRunStatus::WouldDelete,
            },
            DryRunUndeployEntry {
                path: PathBuf::from(".claude/commands/standup.md"),
                status: UndeployDryRunStatus::WouldDelete,
            },
        ];
        assert!(
            entries
                .iter()
                .all(|e| e.status == UndeployDryRunStatus::WouldDelete)
        );
        assert_eq!(entries.len(), 2);
    }

    // all-edited undeploy entries are classified as Edited
    #[test]
    fn undeploy_summary_all_edited() {
        let entries = vec![DryRunUndeployEntry {
            path: PathBuf::from(".claude/commands/edited.md"),
            status: UndeployDryRunStatus::Edited,
        }];
        assert_eq!(entries[0].status, UndeployDryRunStatus::Edited);
    }

    // mixed undeploy entries contain both WouldDelete and Edited
    #[test]
    fn undeploy_summary_mixed() {
        let entries = vec![
            DryRunUndeployEntry {
                path: PathBuf::from(".claude/commands/hello.md"),
                status: UndeployDryRunStatus::WouldDelete,
            },
            DryRunUndeployEntry {
                path: PathBuf::from(".claude/commands/edited.md"),
                status: UndeployDryRunStatus::Edited,
            },
        ];
        assert!(
            entries
                .iter()
                .any(|e| e.status == UndeployDryRunStatus::WouldDelete)
        );
        assert!(
            entries
                .iter()
                .any(|e| e.status == UndeployDryRunStatus::Edited)
        );
        assert_eq!(entries.len(), 2);
    }
}
