// Static content used to scaffold workspace files during `dotagents init`.

/// Starter hello.md command mock content used during `init`.
pub(crate) const COMMAND_MOCK: &str = "\
---
name: hello
description: A Hello Command to greet the User
---

# Hello Command for {{ var.agent_name }}

Greet the User with his name if present, else greet user as stranger. Tell him you are {{ env.app_name }} command.

Context: $USER_INPUT
";

/// Starter INSTRUCTIONS.md mock content used during `init`.
pub(crate) const INSTRUCTION_MOCK: &str = "\
# Instructions for {{ var.agent_name }} from {{ env.app_name }}

This is a custom instructions for {{ var.agent_name }} for a given repository by {{ env.app_name }}.
";

/// Starter hello-skill SKILL.md mock content used during `init`.
pub(crate) const SKILL_MOCK: &str = "\
---
name: hello-skill
description: Greets the user and demonstrates skill capabilities. Use when asked to say hello or to show how skills work.
license: MIT
compatibility: Any agent supporting the Agent Skills specification
metadata:
  author: dotagents
  version: \"1.0.0\"
---

# Hello Skill for {{ var.agent_name }}

This is a sample skill demonstrating the Agent Skills specification format using {{ env.app_name }}

## Instructions

When activated, respond with a friendly greeting and briefly explain that
skills are reusable, model-invoked capabilities that bundle instructions,
scripts, and resources for specific tasks.

## Example

> Hello! I'm using the hello-skill. Skills let agents load focused,
> task-specific knowledge on demand.
";

/// Starter mcp.jsonc mock content used during `init`.
pub(crate) const MCP_MOCK: &str = r#"{
  "$schema": "https://dotagents.soorya-u.dev/v1/schemas/mcp.schema.json",
  "servers": {
    "server-stdio": {
      "type": "stdio",
      "command": "python",
      "args": [],
      "cwd": "{{ dir.workspace }}",
      "env": {},
      "envFile": ".env.local"
    },
    "server-mcp": {
      "type": "http",
      "url": "http://localhost:9000",
      "headers": {
        "Authorization": "Bearer ${API_KEY}"
      }
    }
  }
}
"#;

/// Default .env.example content.
pub(crate) const ENV_EXAMPLE: &str = "APP_NAME=dotagents\n";

/// Default .gitignore content.
pub(crate) const GITIGNORE: &str = "cache.toml\nlocal.config.toml\n.env\n";

/// Mycode custom-provider command template.
pub(crate) const TEMPLATE_MYCODE_COMMAND: &str = "{{command.content}}";

/// Mycode custom-provider skill template.
pub(crate) const TEMPLATE_MYCODE_SKILL: &str = r#"---
name: {{skill.name}}
description: {{skill.description}}
{{#if skill.license}}
license: {{skill.license}}
{{/if}}
{{#if skill.compatibility}}
compatibility: {{skill.compatibility}}
{{/if}}
{{#if skill.metadata}}
metadata:
{{#each skill.metadata}}
  {{@key}}: {{this}}
{{/each}}
{{/if}}
{{#if skill.[allowed-tools]}}
allowed-tools: {{skill.[allowed-tools]}}
{{/if}}
---

{{{skill.content}}}
"#;

/// Mycode custom-provider instructions template.
pub(crate) const TEMPLATE_MYCODE_INSTRUCTIONS: &str = "{{instruction.content}}";

/// Mycode custom-provider MCP template.
pub(crate) const TEMPLATE_MYCODE_MCP: &str = r#"{
  "mcpServers": {
    {{#each mcp.servers}}
    "{{@key}}": {
      "type": {{#ifEq this.type "stdio"}}"local"{{else}}"{{this.type}}"{{/ifEq}},
      {{#ifEq this.type "http"}}
      "url": "{{this.url}}",
      "headers": {{json this.headers}},
      {{else}}
      "command": "{{this.command}}",
      "args": {{json this.args}},
      "env": {{json this.env}},
      {{/ifEq}}
      "tools": {{json this.enabledTools}}
    }{{#unless @last}},{{/unless}}
    {{/each}}
  }
}
"#;

/// Provider config block appended to `local.config.toml` when using the `with-custom-provider` template.
pub(crate) const MYCODE_PROVIDER_CONFIG: &str = r#"
[providers.mycode.mcp]
template = "{{ dir.application }}/templates/mycode/mcp.hbs"
target = "{{ dir.workspace }}/.mycode/mcp.json"

[providers.mycode.instructions]
template = "{{ dir.application }}/templates/mycode/instructions.hbs"
target = "{{ dir.workspace }}/.mycode/instructions.md"
variables = {agent_name = "Mycode"}

[providers.mycode.commands]
template = "{{ dir.application }}/templates/mycode/command.hbs"
target = "{{ dir.workspace }}/.mycode/commands/{{ command.name }}.md"
variables = {agent_name = "Mycode"}

[providers.mycode.skills]
template = "{{ dir.application }}/templates/mycode/skill.hbs"
target = "{{ dir.workspace }}/.mycode/skills/{{ skill.name }}/SKILL.md"
variables = {agent_name = "Mycode"}
"#;

/// Generates a starter `config.toml` / `local.config.toml` with the given features and targets.
pub(crate) fn default_config(features: &[&str], targets: &[&str]) -> String {
    let features_toml = if features.is_empty() {
        "[]".to_string()
    } else {
        let items = features
            .iter()
            .map(|f| format!("    \"{f}\""))
            .collect::<Vec<_>>()
            .join(",\n");
        format!("[\n{items},\n]")
    };

    let targets_toml = {
        let items = targets
            .iter()
            .map(|t| format!("\"{t}\""))
            .collect::<Vec<_>>()
            .join(", ");
        format!("[{items}]")
    };

    format!(
        "schema = \"https://dotagents.soorya-u.dev/schemas/config.schema.json\"\nfeatures = {features_toml}\n\ntargets = {targets_toml}\nvariables = {{ \"agent_name\" = \"my agent\" }}\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // default_config with standard features and targets produces valid TOML
    #[test]
    fn default_config_produces_valid_toml() {
        let output = default_config(&["commands", "instructions", "mcp", "skills"], &["claude"]);
        let value: toml::Value = toml::from_str(&output).expect("should be valid TOML");
        let features = value["features"].as_array().unwrap();
        let feature_strs: Vec<&str> = features.iter().filter_map(|v| v.as_str()).collect();
        assert!(feature_strs.contains(&"commands"));
        assert!(feature_strs.contains(&"instructions"));
        assert!(feature_strs.contains(&"mcp"));
        assert!(feature_strs.contains(&"skills"));
        let targets = value["targets"].as_array().unwrap();
        assert_eq!(targets[0].as_str(), Some("claude"));
    }

    // default_config with empty slices produces features = [] and targets = []
    #[test]
    fn default_config_empty_slices() {
        let output = default_config(&[], &[]);
        let value: toml::Value = toml::from_str(&output).expect("should be valid TOML");
        assert!(value["features"].as_array().unwrap().is_empty());
        assert!(value["targets"].as_array().unwrap().is_empty());
    }

    // CommandFeature::mock() returns non-empty string with YAML frontmatter containing name:
    #[test]
    fn command_mock_has_frontmatter() {
        use crate::core::features::command::CommandFeature;
        let mock = CommandFeature::mock();
        assert!(!mock.is_empty());
        assert!(mock.starts_with("---"));
        assert!(mock.contains("name:"));
    }

    // SkillFeature::mock() returns non-empty string with YAML frontmatter containing name:
    #[test]
    fn skill_mock_has_frontmatter() {
        use crate::core::features::skill::SkillFeature;
        let mock = SkillFeature::mock();
        assert!(!mock.is_empty());
        assert!(mock.starts_with("---"));
        assert!(mock.contains("name:"));
    }
}
