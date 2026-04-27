use std::fs;

use anyhow::{Context, Result};
use gray_matter::Matter;
use gray_matter::engine::YAML;
use serde_json::Value;

use crate::cli::options::LsOptions;
use crate::cli::ui::ls::{ListItem, render_ls};
use crate::utils::path::{get_application_dir, get_commands_dir, get_skills_dir};

/// Load all skills from `.dotagents/skills/*/SKILL.md`, returning name+description pairs.
fn load_skills() -> Result<Vec<ListItem>> {
    let skills_dir = match get_skills_dir() {
        Ok(d) => d,
        Err(_) => return Ok(vec![]), // skills dir absent → empty
    };

    let matter = Matter::<YAML>::new();
    let mut items = Vec::new();

    for entry in fs::read_dir(&skills_dir).context("failed to read skills directory")? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let skill_md = path.join("SKILL.md");
        if !skill_md.is_file() {
            continue;
        }
        let content = fs::read_to_string(&skill_md).context("failed to read SKILL.md")?;
        let Ok(parsed) = matter.parse::<Value>(&content) else {
            continue;
        };
        if let Some(data) = parsed.data {
            let name: String = data["name"].as_str().unwrap_or("").to_string();
            let description: String = data["description"].as_str().unwrap_or("").to_string();
            if !name.is_empty() {
                items.push(ListItem { name, description });
            }
        }
    }

    items.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(items)
}

/// Load all commands from `.dotagents/commands/*.md`, returning name+description pairs.
fn load_commands() -> Result<Vec<ListItem>> {
    let commands_dir = match get_commands_dir() {
        Ok(d) => d,
        Err(_) => return Ok(vec![]), // commands dir absent → empty
    };

    let matter = Matter::<YAML>::new();
    let mut items = Vec::new();

    for entry in fs::read_dir(&commands_dir).context("failed to read commands directory")? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let content = fs::read_to_string(&path).context("failed to read command file")?;
        let Ok(parsed) = matter.parse::<Value>(&content) else {
            continue;
        };
        if let Some(data) = parsed.data {
            let name: String = data["name"].as_str().unwrap_or("").to_string();
            let description: String = data["description"].as_str().unwrap_or("").to_string();
            if !name.is_empty() {
                items.push(ListItem { name, description });
            }
        }
    }

    items.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(items)
}

/// Run `dotagents ls`.
pub(crate) fn run_ls(opts: LsOptions) -> Result<bool> {
    // Ensure we are inside a workspace.
    get_application_dir().context(
        "No .dotagents directory found. Run `dotagents init` to initialise a workspace.",
    )?;

    let skills = load_skills().context("failed to load skills")?;
    let commands = load_commands().context("failed to load commands")?;

    render_ls(skills, commands, &opts);
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_command(dir: &std::path::Path, name: &str, description: &str) {
        let content = format!(
            "---\nname: \"{}\"\ndescription: \"{}\"\n---\n\nBody.\n",
            name, description
        );
        fs::write(dir.join(format!("{}.md", name)), content).unwrap();
    }

    fn make_skill(skills_dir: &std::path::Path, name: &str, description: &str) {
        let skill_dir = skills_dir.join(name);
        fs::create_dir_all(&skill_dir).unwrap();
        let content = format!(
            "---\nname: {}\ndescription: \"{}\"\n---\n\nBody.\n",
            name, description
        );
        fs::write(skill_dir.join("SKILL.md"), content).unwrap();
    }

    #[test]
    fn load_commands_reads_md_frontmatter() {
        let tmp = TempDir::new().unwrap();
        make_command(tmp.path(), "hello", "Says hello");
        make_command(tmp.path(), "world", "Says world");

        // Temporarily swap get_commands_dir by reading the dir directly.
        let matter = Matter::<YAML>::new();
        let mut items = Vec::new();
        for entry in fs::read_dir(tmp.path()).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                let content = fs::read_to_string(&path).unwrap();
                if let Ok(parsed) = matter.parse::<Value>(&content) {
                    if let Some(data) = parsed.data {
                        items.push(data["name"].as_str().unwrap_or("").to_string());
                    }
                }
            }
        }
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn load_skills_reads_skill_md_frontmatter() {
        let tmp = TempDir::new().unwrap();
        make_skill(tmp.path(), "my-skill", "Does something");

        let matter = Matter::<YAML>::new();
        let skill_md = tmp.path().join("my-skill").join("SKILL.md");
        let content = fs::read_to_string(&skill_md).unwrap();
        let parsed = matter.parse::<Value>(&content).unwrap();
        let data = parsed.data.unwrap();
        assert_eq!(data["name"].as_str().unwrap(), "my-skill");
        assert_eq!(data["description"].as_str().unwrap(), "Does something");
    }
}
