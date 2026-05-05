## Why

Three small correctness bugs in the mock files scaffolded by `dotagents init` cause immediate validator noise in editors that support JSON Schema — users see errors the moment they open the generated files, undermining confidence in the tool.

## What Changes

- Remove `cache/` from the `.gitignore` mock in `src/constants/mocks.rs` — no such directory is created by the tool, so including it is misleading dead noise.
- Add `"$schema"` as an allowed property in `public/v1/schemas/mcp.schema.json` — the root object has `additionalProperties: false` with only `servers` in `properties`, so the `$schema` key that `mcp.jsonc` emits for editor tooling is currently rejected.
- Fix `"env_file": null` → `"envFile": ".env.local"` in the `MCP_MOCK` constant — the schema defines the property as `envFile` (camelCase), and `null` is not a valid `string` value; this causes two separate validation errors on the generated file.

## Capabilities

### New Capabilities

*(none — pure correctness fix, no new capabilities introduced)*

### Modified Capabilities

*(none — no existing specs affected)*

## Impact

- `src/constants/mocks.rs` — two constant edits (`GITIGNORE` and `MCP_MOCK`)
- `public/v1/schemas/mcp.schema.json` — add `"$schema"` property to root `properties` object
- Any test that asserts on generated `.gitignore` or `mcp.jsonc` content must be updated to match the corrected values
