## Context

Mock file contents are defined as `&'static str` constants in `src/constants/mocks.rs` and written verbatim to disk during `dotagents init`. The MCP JSON Schema lives at `public/v1/schemas/mcp.schema.json` and is served publicly for editor tooling. All three bugs are independent single-point edits with no runtime behaviour change — the mocks are write-only during init, and the schema is never read by the Rust binary itself.

## Goals / Non-Goals

**Goals:**
- Eliminate all JSON Schema validator warnings on freshly-initialised workspace files.
- Keep the generated `mcp.jsonc` valid against `public/v1/schemas/mcp.schema.json`.

**Non-Goals:**
- Changing any runtime parsing or deploy behaviour.
- Restructuring how mocks are stored or loaded.
- Modifying any provider template.

## Decisions

**`$schema` in the JSON Schema root — add as explicit property, not remove from mock**

Options considered:
1. Remove `"$schema"` from `MCP_MOCK` so validators never see it.
2. Add `"$schema"` to the root `properties` so validators allow it.

Option 2 is correct: `$schema` is standard JSON Schema practice for associating a file with its schema, and editors rely on it for autocompletion and validation. Removing it would break that tooling integration, which is the whole point of the key.

**`env_file` → `envFile` with a real example value**

The schema uses camelCase throughout (per the MCP specification and the CLAUDE.md convention). The value changes from `null` (invalid for `type: string`) to `".env.local"` — a concrete example that is also useful as documentation for the user.

## Risks / Trade-offs

- [Tests asserting on raw `.gitignore` or `mcp.jsonc` content will break] → Update those assertions as part of this change; the breakage is intentional and caught by `mise tests`.
- [No risk of runtime regression] → The mocks are never parsed by the Rust binary during normal operation; only `init` reads and writes them.
