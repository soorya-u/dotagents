/// Trusted remote domain for template and registry fetches, baked in at compile time.
///
/// Includes a trailing slash so `url.starts_with(TRUSTED_DOMAIN)` cannot be bypassed
/// by a domain that merely shares the prefix (e.g. `dotagents.soorya-u.dev.evil.com`).
///
/// Override at build time:
///   `DOTAGENTS_TRUSTED_TEMPLATE_DOMAIN=https://my-host.example.com/ cargo build`
pub(crate) const TRUSTED_DOMAIN: &str =
    if let Some(domain) = option_env!("DOTAGENTS_TRUSTED_TEMPLATE_DOMAIN") {
        domain
    } else {
        "https://dotagents.soorya-u.dev/"
    };

/// Canonical URL for the published provider registry.
pub(crate) const REGISTRY_URL: &str = "https://dotagents.soorya-u.dev/v1/templates/registry.json";
