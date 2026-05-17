## 1. Init validation error tests

- [ ] 1.1 Add e2e test in `tests/e2e/init.test.ts`: run `init --features none,commands --ci`, assert exit 1 and stderr mentions `none` exclusivity (TC-INIT-07)
- [ ] 1.2 Add e2e test in `tests/e2e/init.test.ts`: run `init --features bogus --ci`, assert exit 2 and stderr contains "invalid value" with valid feature names (TC-INIT-ERR-01)
- [ ] 1.3 Add e2e test in `tests/e2e/init.test.ts`: run `init --template bogus --ci`, assert exit 2 and stderr contains "invalid value" with valid template names (TC-INIT-ERR-02)

## 2. Config missing-file tests

- [ ] 2.1 Add e2e test in `tests/e2e/config.test.ts`: init, delete `local.config.toml`, run `config local --ci`, assert exit 0 and stdout contains "No local config found" (TC-CFG-08 text)
- [ ] 2.2 Add e2e test in `tests/e2e/config.test.ts`: init, delete `local.config.toml`, run `config local --json`, assert exit 0 and stdout is `{}` (TC-CFG-08 JSON)
- [ ] 2.3 Add e2e test in `tests/e2e/config.test.ts`: init, delete `config.toml`, run `config global --ci`, assert exit 1 and stderr contains "not found" (TC-CFG-09)
- [ ] 2.4 Add e2e test in `tests/e2e/config.test.ts`: init, delete `config.toml`, run `config global --json`, assert exit 1 and stderr contains "not found" (TC-CFG-09 JSON)

## 3. Providers TUI test

- [ ] 3.1 Run tui-devtools discovery on `providers --offline` with seeded cache to capture exact terminal output (select widget, navigation, Enter, Escape)
- [ ] 3.2 Add TUI e2e test in `tests/e2e/providers.test.ts`: seed registry cache, launch `providers --offline` in PTY, assert select widget renders with provider names, navigate with arrow-down, press Enter, assert outro shown (TC-PROV-01)

## 4. Providers --quiet/--verbose fixes and tests

- [ ] 4.1 Update `src/cli/providers.rs` to gate text output behind quiet flag check — when `--quiet` is active, skip `println!()` output (TC-PROV-09 implementation)
- [ ] 4.2 Add `debug!()` calls in providers fetch/cache path to emit diagnostic info visible at `-v` level (TC-PROV-10 implementation)
- [ ] 4.3 Add e2e test in `tests/e2e/providers.test.ts`: seed cache, run `providers --ci --quiet --offline`, assert exit 0 and stdout is empty (TC-PROV-09)
- [ ] 4.4 Add e2e test in `tests/e2e/providers.test.ts`: seed cache, run `providers --ci -v --offline`, assert exit 0 and stderr contains debug diagnostic output (TC-PROV-10); both tests use seeded cache + `--offline` for determinism

## 5. Verification

- [ ] 5.1 Run `mise check` (fmt + clippy) — must exit 0
- [ ] 5.2 Run `mise tests` (unit + integration + e2e) — must exit 0
