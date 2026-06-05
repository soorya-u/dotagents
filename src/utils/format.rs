use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeFormat {
    Json,
    Jsonc,
    Toml,
    Yaml,
}

impl MergeFormat {
    pub fn from_extension(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext.to_lowercase().as_str() {
                "json" => Some(Self::Json),
                "jsonc" => Some(Self::Jsonc),
                "toml" => Some(Self::Toml),
                "yaml" | "yml" => Some(Self::Yaml),
                _ => None,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_json_extension() {
        assert_eq!(
            MergeFormat::from_extension(&PathBuf::from("config.json")),
            Some(MergeFormat::Json)
        );
    }

    #[test]
    fn test_jsonc_extension() {
        assert_eq!(
            MergeFormat::from_extension(&PathBuf::from("config.jsonc")),
            Some(MergeFormat::Jsonc)
        );
    }

    #[test]
    fn test_toml_extension() {
        assert_eq!(
            MergeFormat::from_extension(&PathBuf::from("config.toml")),
            Some(MergeFormat::Toml)
        );
    }

    #[test]
    fn test_yaml_extension() {
        assert_eq!(
            MergeFormat::from_extension(&PathBuf::from("config.yaml")),
            Some(MergeFormat::Yaml)
        );
    }

    #[test]
    fn test_yml_extension() {
        assert_eq!(
            MergeFormat::from_extension(&PathBuf::from("config.yml")),
            Some(MergeFormat::Yaml)
        );
    }

    #[test]
    fn test_unknown_extension() {
        assert_eq!(
            MergeFormat::from_extension(&PathBuf::from("config.txt")),
            None
        );
    }

    #[test]
    fn test_no_extension() {
        assert_eq!(MergeFormat::from_extension(&PathBuf::from("config")), None);
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(
            MergeFormat::from_extension(&PathBuf::from("config.JSON")),
            Some(MergeFormat::Json)
        );
        assert_eq!(
            MergeFormat::from_extension(&PathBuf::from("config.TOML")),
            Some(MergeFormat::Toml)
        );
    }
}
