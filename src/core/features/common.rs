use strum_macros::{AsRefStr, Display, EnumIter, EnumString, VariantNames};

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, EnumString, AsRefStr, EnumIter, VariantNames, Display,
)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Feature {
    Command,
    Instruction,
    Mcp,
    Skill,
    AgentIgnore,
}

impl Feature {
    pub(crate) fn feature_filename(&self) -> String {
        format!("{}.hbs", self.as_ref())
    }

    pub(crate) fn is_provider_agnostic(&self) -> bool {
        matches!(self, Feature::Skill | Feature::AgentIgnore)
    }
}
