use crate::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::core::config::FeatureSettings;
use crate::core::features::traits::FeatureTrait;
use crate::templates::{Templater, resolve_target_path};
use crate::utils::json::merge_json;

/// Tracks deduplication info for a skipped provider.
#[derive(Debug, Clone)]
pub(crate) struct DedupInfo {
    pub(crate) winner: String,
}

/// A single unit of work for deploy_feature: one (provider, item) pair.
pub(crate) struct DeployWorkItem<'a, T: FeatureTrait> {
    pub(crate) provider_name: String,
    pub(crate) settings: &'a FeatureSettings,
    pub(crate) item: &'a T,
    pub(crate) dedup: Option<DedupInfo>,
}

/// Resolves target paths for all providers for a single item, grouping by path.
pub(crate) fn resolve_provider_paths<'a, T: FeatureTrait>(
    item: &'a T,
    providers: &'a HashMap<String, FeatureSettings>,
    templater: &Templater,
    variables: Option<&Value>,
) -> Result<HashMap<PathBuf, Vec<(&'a String, &'a FeatureSettings)>>> {
    let name_var: Option<Value> = item
        .get_file_name()
        .map(|filename| item.get_name_variable(&filename))
        .transpose()?
        .flatten();
    let item_base_vars = merge_json(variables, name_var.as_ref());

    let mut path_groups: HashMap<PathBuf, Vec<(&String, &FeatureSettings)>> = HashMap::new();
    for (provider_name, settings) in providers {
        let target_str = settings
            .target
            .as_deref()
            .ok_or_else(|| anyhow!("Target config not found for provider {}", provider_name))?;
        let target_path = resolve_target_path(templater, target_str, Some(&item_base_vars))?;
        path_groups
            .entry(target_path)
            .or_default()
            .push((provider_name, settings));
    }
    Ok(path_groups)
}

/// Groups providers by resolved target path and marks dedup winners/losers.
/// Returns `(target_path, winner, losers)` for each unique path.
pub(crate) fn dedup_by_path(
    providers: &[(String, PathBuf)],
) -> Vec<(PathBuf, String, Vec<String>)> {
    let mut path_groups: HashMap<PathBuf, Vec<String>> = HashMap::new();
    for (provider_name, target_path) in providers {
        path_groups
            .entry(target_path.clone())
            .or_default()
            .push(provider_name.clone());
    }

    let mut result: Vec<(PathBuf, String, Vec<String>)> = path_groups
        .into_iter()
        .map(|(path, mut group)| {
            group.sort();
            let winner = group.remove(0);
            (path, winner, group)
        })
        .collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

/// Builds the dedup-aware work list for a single item across all providers.
pub(crate) fn build_item_work_items<'a, T: FeatureTrait>(
    item: &'a T,
    providers: &'a HashMap<String, FeatureSettings>,
    templater: &Templater,
    variables: Option<&Value>,
) -> Result<Vec<DeployWorkItem<'a, T>>> {
    let path_groups = resolve_provider_paths(item, providers, templater, variables)?;

    let provider_pairs: Vec<(String, PathBuf)> = path_groups
        .into_iter()
        .flat_map(|(path, group)| {
            group
                .into_iter()
                .map(|(name, _)| (name.clone(), path.clone()))
                .collect::<Vec<_>>()
        })
        .collect();

    let dedup_results = dedup_by_path(&provider_pairs);

    let mut work_list = Vec::new();
    for (_path, winner, losers) in dedup_results {
        let settings = providers.get(&winner).unwrap();
        work_list.push(DeployWorkItem {
            provider_name: winner.clone(),
            settings,
            item,
            dedup: None,
        });
        for loser in losers {
            let settings = providers.get(&loser).unwrap();
            work_list.push(DeployWorkItem {
                provider_name: loser,
                settings,
                item,
                dedup: Some(DedupInfo {
                    winner: winner.clone(),
                }),
            });
        }
    }
    Ok(work_list)
}

/// Builds the full dedup-aware work list for all items.
pub(crate) fn build_work_list<'a, T: FeatureTrait>(
    items: &'a [T],
    providers: &'a HashMap<String, FeatureSettings>,
    templater: &Templater,
    variables: Option<&Value>,
) -> Result<Vec<DeployWorkItem<'a, T>>> {
    let mut all_work = Vec::new();
    for item in items {
        all_work.extend(build_item_work_items(
            item, providers, templater, variables,
        )?);
    }
    Ok(all_work)
}

#[cfg(test)]
mod tests {
    use super::dedup_by_path;
    use std::path::PathBuf;

    // alphabetical winner selected when 3 providers target same path
    #[test]
    fn dedup_alphabetical_winner_three_providers() {
        let providers = vec![
            ("zebra".to_string(), PathBuf::from("AGENTS.md")),
            ("alpha".to_string(), PathBuf::from("AGENTS.md")),
            ("middle".to_string(), PathBuf::from("AGENTS.md")),
        ];
        let result = dedup_by_path(&providers);
        assert_eq!(result.len(), 1);
        let (path, winner, losers) = &result[0];
        assert_eq!(path, &PathBuf::from("AGENTS.md"));
        assert_eq!(winner, "alpha");
        assert_eq!(losers, &vec!["middle".to_string(), "zebra".to_string()]);
    }

    // no dedup when providers target different paths
    #[test]
    fn dedup_no_collision_different_paths() {
        let providers = vec![
            ("claude".to_string(), PathBuf::from(".claude/AGENTS.md")),
            ("codex".to_string(), PathBuf::from(".openai/AGENTS.md")),
        ];
        let result = dedup_by_path(&providers);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|(_, _, losers)| losers.is_empty()));
    }

    // mixed: some providers share path, others don't
    #[test]
    fn dedup_mixed_some_collide() {
        let providers = vec![
            ("a".to_string(), PathBuf::from("shared.md")),
            ("b".to_string(), PathBuf::from("shared.md")),
            ("c".to_string(), PathBuf::from("unique.md")),
        ];
        let result = dedup_by_path(&providers);
        assert_eq!(result.len(), 2);
        let shared = result
            .iter()
            .find(|(p, _, _)| p == &PathBuf::from("shared.md"))
            .unwrap();
        let unique = result
            .iter()
            .find(|(p, _, _)| p == &PathBuf::from("unique.md"))
            .unwrap();
        assert_eq!(shared.1, "a");
        assert_eq!(shared.2, vec!["b"]);
        assert_eq!(unique.1, "c");
        assert_eq!(unique.2, Vec::<String>::new());
    }
}
