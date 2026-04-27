/// Starter body template for a new command file. `{name}` is replaced with the command name.
pub(crate) const COMMAND_STARTER: &str = "# {name}

Brief description of what this command does.

## When to use

Describe when this command should be triggered.

## Steps

1. First step
2. Second step
3. Additional steps as needed
";

/// Starter body template for a new skill file. `{name}` is replaced with the skill name.
pub(crate) const SKILL_STARTER: &str = "# {name}

Instructions for the agent to follow when this skill is activated.

## When to use

Describe when this skill should be used.

## Instructions

1. First step
2. Second step
3. Additional steps as needed
";

/// Substitute `{name}` placeholder in a starter template with the given name.
pub(crate) fn render_starter(template: &str, name: &str) -> String {
    template.replace("{name}", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_starter_substitutes_name_in_command_template() {
        let result = render_starter(COMMAND_STARTER, "my-cmd");
        assert!(result.contains("# my-cmd"));
        assert!(!result.contains("{name}"));
    }

    #[test]
    fn render_starter_substitutes_name_in_skill_template() {
        let result = render_starter(SKILL_STARTER, "my-skill");
        assert!(result.contains("# my-skill"));
        assert!(!result.contains("{name}"));
    }
}
