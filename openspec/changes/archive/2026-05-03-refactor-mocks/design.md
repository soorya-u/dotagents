## Context

`src/constants/mocks.rs` currently holds fourteen `include_str!` macros pointing into `src/mocks/`. The files range from a one-line `{{command.content}}` Handlebars template to a 25-line SKILL.md with YAML frontmatter. None of them need to be separate files — they are small, rarely edited, and have no non-Rust consumers. The `config.toml` mock is the only one with meaningful variance: it will need to accept selected features and targets after `fix-init-ux`.

The `InitFile` struct in `src/cli/init.rs` holds a `content: &'static str` field — it needs a string slice at compile time. This means content must remain `&'static str` where possible, but `config.toml` generation becomes a runtime `String`.

## Goals / Non-Goals

**Goals:**
- All mock content lives in Rust source files, no external `src/mocks/` assets.
- Each feature type owns its example file content via `mock()`.
- `default_config(features, targets)` generates a valid TOML string at runtime.
- `InitFile` can handle both `&'static str` (most files) and owned `String` (config) — or the config entry is handled separately outside the `init_files` vec.

**Non-Goals:**
- Changing what mock content says (content is preserved verbatim during the refactor).
- Making mock content user-configurable.
- Touching `src/constants/templates.rs`.

## Decisions

**D1 — `InitFile` content stays `&'static str`; config is written separately**

`InitFile.content: &'static str` cannot hold a runtime `String` without lifetime gymnastics or `Cow`. Rather than change the struct, the two config files (`config.toml`, `local.config.toml`) are written directly via `write_file` outside the `init_files` loop, using `default_config(features, targets)`. This is the minimal change: one loop handles all static files, two explicit writes handle the dynamic configs.

Alternative considered: change `content` to `Cow<'static, str>`. Rejected — adds complexity for only two call sites.

**D2 — `mock()` returns `&'static str` via `const` string**

Each feature's `mock()` method returns a `&'static str` referencing a module-level `const` string defined in the same file. This keeps the content colocated with the feature type, accessible to `init.rs` without reaching into `mocks.rs` for feature-specific content.

**D3 — `default_config` lives in `src/constants/mocks.rs`**

It is configuration-scaffolding content, consistent with the rest of `mocks.rs`. It takes `features: &[&str]` and `targets: &[&str]` and formats them into a TOML string using `format!`. No TOML serialisation library needed — the structure is simple and fixed.

Example output:
```toml
schema = "https://dotagents.soorya-u.dev/schemas/config.schema.json"
features = ["commands", "instructions"]
targets = ["claude"]
variables = { "agent_name" = "my agent" }
```

**D4 — Remove unused constants from `src/constants/file.rs` and `src/constants/dir.rs`**

After the refactor, constants like `MOCK_COMMAND_FILE`, `MOCK_SKILL_DIR`, `MOCK_COMMAND_TEMPLATE_FILE`, `MOCK_CUSTOM_AGENT_DIR`, `TEMPLATE_DIR` etc. may become dead code. Remove any that are no longer referenced.

## Risks / Trade-offs

- **Inline strings are harder to preview** — a developer can no longer open `src/mocks/commands/hello.md` to see the scaffold content. Mitigated by colocating with the feature type and adding a clear `// mock content for dotagents init` comment.
- **`default_config` format! output must match TOML** — a typo in the format string won't be caught by the compiler. Mitigated by a unit test asserting the output parses as valid TOML with expected keys.
