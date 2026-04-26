mod remote;
mod renderer;
mod templater;

pub(crate) mod helpers;
pub(crate) mod registry_resolver;
pub(crate) mod template_cache;
pub(crate) mod variables;
pub(crate) use registry_resolver::{REGISTRY_URL, resolve_provider_defaults};
pub(crate) use remote::{do_get, fetch_template};
pub(crate) use renderer::render_feature_with_settings;
pub(crate) use template_cache::TemplateCache;
pub(crate) use templater::{RenderType, TemplateSource, Templater, get_templater};
