use anyhow::{Result, anyhow};

use crate::constants::domain::TRUSTED_DOMAIN;

/// Fetch a template from a remote URL, validating domain and HTTPS first.
pub(crate) fn fetch_template(url: &str) -> Result<String> {
    validate_url(url)?;
    do_get(url)
}

/// Validate that `url` is an allowed remote template URL.
fn validate_url(url: &str) -> Result<()> {
    if url.starts_with("http://") {
        return Err(anyhow!(
            "Remote template fetch failed: non-HTTPS URL not allowed: {}",
            url
        ));
    }

    if url.starts_with("https://") && !url.starts_with(TRUSTED_DOMAIN) {
        return Err(anyhow!(
            "Remote template fetch failed: untrusted domain. Only {} is supported, got: {}",
            TRUSTED_DOMAIN,
            url
        ));
    }

    if !url.starts_with(TRUSTED_DOMAIN) {
        return Err(anyhow!(
            "Remote template fetch failed: URL must start with {}",
            TRUSTED_DOMAIN
        ));
    }

    Ok(())
}

/// Perform an HTTP GET and return the response body as a `String`.
///
/// Exposed as `pub(crate)` for testing with a local mock server.
pub(crate) fn do_get(url: &str) -> Result<String> {
    let response = ureq::get(url)
        .call()
        .map_err(|e| anyhow!("Remote template fetch failed for {}: {}", url, e))?;

    let status = response.status().as_u16();
    if status != 200 {
        return Err(anyhow!(
            "Remote template fetch failed: {} for {}",
            status,
            url
        ));
    }

    response
        .into_body()
        .read_to_string()
        .map_err(|e| anyhow!("Failed to read remote template response body: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── validate_url ─────────────────────────────────────────────────────────

    // plain HTTP URL is rejected before any network request
    #[test]
    fn http_url_returns_error() {
        let result = fetch_template("http://example.com/template.hbs");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("non-HTTPS"));
    }

    // untrusted HTTPS domain is rejected before any network request
    #[test]
    fn untrusted_https_domain_returns_error() {
        let result = fetch_template("https://example.com/template.hbs");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("untrusted domain"));
    }

    // non-URL string is rejected
    #[test]
    fn missing_protocol_returns_error() {
        let result = fetch_template("example.com/template.hbs");
        assert!(result.is_err());
    }

    // ── do_get (mocked HTTP server) ───────────────────────────────────────────

    // 200 response returns the response body
    #[test]
    fn do_get_200_returns_body() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/template.hbs")
            .with_status(200)
            .with_body("{{command.content}}")
            .create();

        let result = do_get(&format!("{}/template.hbs", server.url()));
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        assert_eq!(result.unwrap(), "{{command.content}}");
        mock.assert();
    }

    // 404 response returns an error containing the status code
    #[test]
    fn do_get_404_returns_error_with_status() {
        let mut server = mockito::Server::new();
        let mock = server.mock("GET", "/missing.hbs").with_status(404).create();

        let result = do_get(&format!("{}/missing.hbs", server.url()));
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("404"),
            "error should mention the status code"
        );
        mock.assert();
    }

    // 500 response returns an error containing the status code
    #[test]
    fn do_get_500_returns_error_with_status() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/template.hbs")
            .with_status(500)
            .create();

        let result = do_get(&format!("{}/template.hbs", server.url()));
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("500"),
            "error should mention the status code"
        );
        mock.assert();
    }
}
