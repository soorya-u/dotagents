use std::sync::OnceLock;

/// Trusted remote domain for template and registry fetches, baked in at compile time; override with `DOTAGENTS_TRUSTED_TEMPLATE_DOMAIN=https://my-host.example.com/ cargo build`.
pub(crate) const TRUSTED_DOMAIN: &str =
    if let Some(domain) = option_env!("DOTAGENTS_TRUSTED_TEMPLATE_DOMAIN") {
        domain
    } else {
        "https://dotagents.soorya-u.dev/"
    };

/// Returns the canonical URL for the published provider registry, derived from `TRUSTED_DOMAIN`.
pub(crate) fn registry_url() -> &'static str {
    static URL: OnceLock<String> = OnceLock::new();
    URL.get_or_init(|| format!("{}v1/templates/registry.json", TRUSTED_DOMAIN))
}
