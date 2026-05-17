## 1. Deploy user-edit protection tests

- [x] 1.1 Add e2e test in `tests/e2e/deploy.test.ts`: deploy, modify a deployed file, redeploy without `--force`, assert modified file retains user edits (TC-DEPLOY-07)
- [x] 1.2 Add e2e test in `tests/e2e/deploy.test.ts`: deploy, modify a deployed file, redeploy with `--force --offline --no-gitignore`, assert file is overwritten (TC-DEPLOY-08)

## 2. Deploy flag and error path tests

- [x] 2.1 Add e2e test in `tests/e2e/deploy.test.ts`: init, deploy with `--no-gitignore`, assert no `.gitignore` created (TC-DEPLOY-10)
- [x] 2.2 Add e2e test in `tests/e2e/deploy.test.ts`: delete `.dotagents/.env` before deploy, assert exit 0 and files deployed (TC-DEPLOY-20)
- [x] 2.3 Add e2e test in `tests/e2e/deploy.test.ts`: write a `.hbs` template with `{{ unclosed` syntax, deploy, assert exit 1 (TC-DEPLOY-ERR-01)
- [x] 2.4 Add e2e test in `tests/e2e/deploy.test.ts`: write provider config with `template = "http://example.com/template.hbs"`, deploy, assert exit 1 and stderr mentions non-HTTPS (TC-DEPLOY-ERR-02)

## 3. Deploy TUI prompt tests

- [x] 3.1 Run tui-devtools discovery on the deploy offline prompt flow (selecting Yes) and the full deploy journey
- [x] 3.2 Add TUI e2e test in `tests/e2e/deploy.test.ts`: offline prompt, navigate down to "Yes", press Enter, assert deploy completes (TC-DEPLOY-16)
- [x] 3.3 Add TUI e2e test in `tests/e2e/deploy.test.ts`: full deploy journey — offline prompt Enter (No) → gitignore prompt Enter (No) → summary → "Done." (TC-DEPLOY-01)

## 4. Undeploy edge case tests

- [x] 4.1 Add e2e test in `tests/e2e/undeploy.test.ts`: deploy, manual delete one deployed file, run `undeploy --force --no-gitignore`, assert exit 0 and remaining files deleted (TC-UNDEPLOY-14)
- [x] 4.2 Update gitignore removal logic in `src/cli/undeploy.rs` to call `clear_gitignore_fence` even when cache is empty

## 5. Verification

- [x] 5.1 Run `mise check` (fmt + clippy) — exit 0
- [x] 5.2 Run `mise tests` (unit + integration + e2e) — exit 0
