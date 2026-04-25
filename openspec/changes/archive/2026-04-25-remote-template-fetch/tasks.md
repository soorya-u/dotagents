## 1. Dependency

- [x] 1.1 Add `ureq = { version = "2", features = ["rustls"] }` to `[dependencies]` in `Cargo.toml` (pure-Rust TLS, no OpenSSL system dependency)

## 2. Remote Fetch Implementation

- [x] 2.1 Implement `fetch_template(url: &str) -> Result<String>` in `src/templates/remote.rs`:
  - Validate `url` starts with `"https://dotagents.soorya-u.dev/"` — return hard error if not
  - Return hard error if `url` starts with `"http://"` (non-HTTPS)
  - Return hard error if `url` starts with `"https://"` but host is not `dotagents.soorya-u.dev`
  - Perform `ureq::get(url).call()` — return hard error on network failure
  - Check response status: return hard error with URL + status code if not 200
  - Return `response.into_string()` as the template content
- [x] 2.2 Export `fetch_template` from `src/templates/mod.rs`
- [x] 2.3 Write unit tests for `fetch_template`:
  - Untrusted HTTPS domain → error
  - Plain HTTP URL → error
  - (Integration test or mock) 404 response → error with status in message
  - (Integration test or mock) 200 response → returns body string

## 3. Renderer Integration

- [x] 3.1 In `src/templates/renderer.rs`, replace the current `PathBuf::from(template_str)` / `read_file` block with a branch:
  - If `template_str` starts with `"https://"` → call `fetch_template(template_str)` to get `template_file_content: String`; propagate errors
  - Otherwise → existing `PathBuf::from` / `.exists()` / `read_file` logic unchanged
- [x] 3.2 Ensure the fetched `String` flows into the same `templater.render_template(RenderType::Content(template_file_content), ...)` call that local content uses — no other changes to rendering logic

## 4. Verification

- [x] 4.1 Run `cargo build` — no compilation errors
- [x] 4.2 Run `cargo test` — all tests pass
- [x] 4.3 Configure a provider with `template = "https://dotagents.soorya-u.dev/templates/claude/command.hbs"` and run `dotagents deploy`; confirm the template is fetched and the output file is rendered correctly
- [x] 4.4 Set `template` to an untrusted HTTPS URL; confirm deploy exits with a clear error and does not make a network request
- [x] 4.5 Set `template` to a plain `http://` URL; confirm deploy exits with a clear error
- [x] 4.6 Set `template` to a valid trusted URL that returns 404; confirm deploy exits with an error containing "404"
- [x] 4.7 Confirm a local file path template still works unchanged after the refactor
- [x] 4.8 Run `cargo fmt && cargo clippy` — no warnings
