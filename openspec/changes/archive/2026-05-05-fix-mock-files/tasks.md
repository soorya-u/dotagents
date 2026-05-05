## 1. Fix .gitignore mock

- [x] 1.1 In `src/constants/mocks.rs`, remove `cache/\n` from the `GITIGNORE` constant (line 79). Result: `"cache.toml\nlocal.config.toml\n.env\n"`.
- [x] 1.2 Search for any test asserting on `.gitignore` content (e.g. in `tests/e2e/init.test.ts` or integration tests) and update the expected string to omit `cache/`.

## 2. Fix MCP JSON Schema — allow $schema at root

- [x] 2.1 In `public/v1/schemas/mcp.schema.json`, add a `"$schema"` entry under the top-level `"properties"` object with `"type": "string"`, `"description": "JSON Schema URI for editor tooling."`, and `"format": "uri"`.
- [x] 2.2 Confirm the updated schema file is itself valid JSON (no trailing commas, correct nesting).

## 3. Fix MCP mock — env_file → envFile

- [x] 3.1 In `src/constants/mocks.rs`, in the `MCP_MOCK` constant, change `"env_file": null` to `"envFile": ".env.local"` inside the `server-stdio` example.
- [x] 3.2 Confirm the updated mock parses correctly through `McpFeature::from_string` — run `cargo test` and check that existing mock-parse unit tests pass.

## 4. Verification

- [x] 4.1 Run `mise check` — no fmt/clippy errors.
- [x] 4.2 Run `mise tests` — all tests pass.
- [x] 4.3 Run `cargo run -- init` in a temp dir; open the generated `mcp.jsonc` and `.gitignore` in an editor with JSON Schema support and confirm zero validation warnings.
