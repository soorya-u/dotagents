use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Feature {
    Command,
    Instruction,
    Mcp,
    Skill,
}

impl Feature {
    /// String key used in config files and the deploy pipeline.
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Feature::Command => "commands",
            Feature::Instruction => "instructions",
            Feature::Mcp => "mcp",
            Feature::Skill => "skills",
        }
    }

    /// Parse from the config-file string representation; returns None for unknown names.
    pub(crate) fn from_str(s: &str) -> Option<Self> {
        match s {
            "commands" => Some(Feature::Command),
            "instructions" => Some(Feature::Instruction),
            "mcp" => Some(Feature::Mcp),
            "skills" => Some(Feature::Skill),
            _ => None,
        }
    }

    /// All valid feature name strings, for use in error messages.
    pub(crate) fn all_names() -> &'static [&'static str] {
        &["commands", "instructions", "mcp", "skills"]
    }

    pub(crate) fn all() -> [Self; 4] {
        [
            Feature::Command,
            Feature::Instruction,
            Feature::Mcp,
            Feature::Skill,
        ]
    }

    pub(crate) fn feature_filename(&self) -> &'static str {
        match self {
            Feature::Command => "command.hbs",
            Feature::Instruction => "instruction.hbs",
            Feature::Mcp => "mcp.hbs",
            Feature::Skill => "skill.hbs",
        }
    }
}

impl fmt::Display for Feature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
