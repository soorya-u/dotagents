## Context

`renderer.rs::render_feature_with_settings` currently handles template loading in two lines:

```rust
let template_path = PathBuf::from(template_str);   // line 36
// ...
if !template_path.exists() { return Err(...) }      // line 47
let template_file_content = read_file(&template_path) // line 71
```

Any `template_str` that is a URL (`https://dotagents.soorya-u.dev/templates/claude/command.hbs`) is blindly converted to a `PathBuf`, fails `.exists()`, and returns an error. `src/templates/remote.rs` exists as a one-line stub, signalling the intent to implement remote fetching there.

The project's release profile (`lto = true`, `opt-level = "s"`, `strip = true`) targets a small binary. This rules out `reqwest` (pulls in tokio). `ureq` is a sync, minimal HTTP client that compiles to ~200KB and has no async runtime dependency.

## Goals / Non-Goals

**Goals:**
- Detect `https://dotagents.soorya-u.dev/` prefix in `template_str` and fetch via HTTP GET
- Return the response body as the template content string, used exactly as a local `.hbs` file would be
- Hard-error on untrusted domains, non-HTTPS schemes, network failures, and non-200 responses
- Keep the local file path as the fallback for all non-URL template values

**Non-Goals:**
- Local caching of fetched templates (separate future proposal)
- Support for any domain other than `dotagents.soorya-u.dev`
- Retry logic on failure
- Redirect following beyond `ureq`'s built-in single redirect

## Decisions

### 1. `ureq` as the HTTP client

**Decision**: Add `ureq` (latest stable) to `[dependencies]`. It is sync, has no async runtime, compiles small, and is widely used in CLI tooling.

**Alternatives considered**:
- `reqwest` — requires tokio; inflates binary and compile time significantly. Rejected.
- `minreq` — even smaller, but less maintained and fewer features (no TLS without extra flags). Rejected.
- `curl` via bindings — system dependency, poor cross-platform story. Rejected.

### 2. URL detection by prefix, not full URL parsing

**Decision**: Check `template_str.starts_with("https://dotagents.soorya-u.dev/")` as the routing condition. No full URL parsing library needed.

**Trusted domain validation**: If `template_str` starts with `"https://"` but NOT with the trusted prefix → hard error with a message explaining only `dotagents.soorya-u.dev` is supported. If it starts with `"http://"` → hard error (non-HTTPS). This ordering means local paths (e.g. `{{ dir.application }}/templates/mycode/command.hbs`) that don't start with `https://` fall through to the existing local file logic unchanged.

**Alternatives considered**:
- Full URL parsing with the `url` crate to extract host — adds a dependency for a single string check. Rejected.

### 3. Fetch logic lives in `src/templates/remote.rs`

**Decision**: Implement a single public function `fetch_template(url: &str) -> Result<String>` in `remote.rs`. `renderer.rs` calls it and uses the returned `String` as `template_file_content` — exactly where `read_file(&template_path)` is used today. The diff to `renderer.rs` is minimal: replace the `PathBuf::from` block with a match on the URL prefix.

### 4. Non-200 response is a hard error with status code in message

**Decision**: `ureq` returns a `Response`; check `.status()`. If not 200, return `Err` with the URL and status code included in the message (e.g. `"Remote template fetch failed: 404 Not Found for https://..."`). This makes debugging provider.toml misconfiguration straightforward.

### 5. No caching

**Decision**: Every deploy re-fetches all remote templates. This is intentional and temporary. Caching will be a separate proposal at the application level (`~/.config/dotagents/template-cache/`) rather than per-project (`.dotagents/`), so it doesn't clutter project directories and is shared across all projects using the same templates.

## Risks / Trade-offs

- **Every deploy makes network requests** → Deploy will fail in offline environments if any provider uses a remote template. Acceptable for now; caching proposal will address this.
- **Binary size increases with `ureq` + TLS stack** → `ureq` with `native-tls` or `rustls` adds ~1–2MB. Using `rustls` (pure Rust, no system dependency) is preferred; avoids OpenSSL linking on Linux/Windows.
- **Parallel provider deploys all fetch the same template** → If 3 providers use the same remote template, it is fetched 3 times. Not a correctness issue; caching will eliminate this.
- **URL typos in provider.toml cause hard errors** → Intentional — silent failures would produce empty output silently.

## Open Questions

*(none)*
