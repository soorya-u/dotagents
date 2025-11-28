mod remote;
mod renderer;
mod templater;

pub(crate) mod helpers;
pub(crate) mod variables;
pub(crate) use renderer::render_feature_with_settings;
pub(crate) use templater::{RenderType, TemplateSource, Templater, get_templater};
