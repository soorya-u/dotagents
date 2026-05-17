## 1. Fix error message

- [x] 1.1 In `src/cli/providers.rs`, change the `"run 'dotagents providers ls'"` string (cold-cache error message, line ~63) to `"run 'dotagents providers'"`

## 2. Graceful Escape in TUI

- [x] 2.1 In `run_tui()`, replace `.map_err(|e| anyhow!("TUI interaction failed: {}", e))?` with `.unwrap_or_default()` (or `match` returning `Ok(true)`) so that any interaction error (Escape, Ctrl-C) exits cleanly with `Ok(true)` instead of propagating

## 3. Debug logging

- [x] 3.1 Add `debug!("Fetching provider registry from {}", url)` before the HTTP GET call in `fetch_registry()`
- [x] 3.2 Add `debug!("Registry cached at {}", path.display())` after successful `std::fs::write` in `cache_registry()`

## 4. Suppress spinner in non-TUI mode

- [x] 4.1 In `run_providers()`, gate the spinner start on `is_tui_enabled()` in addition to `is_tty()` — replace the `is_tty() && !opts.offline && !opts.json` branch condition to also check `is_tui_enabled()`

## 5. Unit tests

- [x] 5.1 Update or add unit test that asserts the cold-cache error message contains `"dotagents providers"` and NOT `"dotagents providers ls"`

## 6. E2e tests

- [x] 6.1 Add or update e2e test in `tests/e2e/providers.test.ts`: `providers --offline` when cache is cold — assert exit 1 and error mentions `"dotagents providers"` (not `"providers ls"`)
- [x] 6.2 Add e2e test: `providers -v --offline` when cache is cold — assert debug line with registry URL is present on stderr (or skip if registry is not available in CI)

## 7. Verification

- [x] 7.1 Run `mise check` (fmt + clippy) — must exit 0
- [x] 7.2 Run `mise tests` (unit + integration + e2e) — must exit 0
