## 1. Commands --deploy flag tests

- [x] 1.1 TC-CMD-NEW-06: Already covered by existing "CI auto-deploys after commands new" test in `commands deploy-default` block. The CLI has no `--deploy` flag; CI mode auto-deploys by default via `--no-deploy` (default false).
- [x] 1.2 TC-CMD-RM-06: Already covered by existing "CI auto-deploys after commands rm" test in `commands deploy-default` block. Same reasoning as 1.1.

## 2. Commands error path tests

- [x] 2.1 Add e2e test in `tests/e2e/commands.test.ts`: `init --ci`, run `commands new NAME --ci` with no metadata flags, read file, assert `description: ''` in frontmatter and no `category`/`tags` keys (TC-CMD-NEW-03)
- [x] 2.2 Add e2e test in `tests/e2e/commands.test.ts`: create an empty temp directory (not a workspace), run `commands new NAME --cwd <empty-temp-dir> --ci`, assert exit 1 and stderr mentions missing `.dotagents` (TC-CMD-NEW-10)

## 3. Skills --deploy flag tests

- [x] 3.1 TC-SKILL-NEW-06: Already covered by existing "CI auto-deploys after skills new" test in `skills deploy-default` block. The CLI has no `--deploy` flag; CI mode auto-deploys by default via `--no-deploy` (default false).
- [x] 3.2 TC-SKILL-RM-06: Already covered by existing "CI auto-deploys after skills rm" test in `skills deploy-default` block. Same reasoning as 3.1.

## 4. Skills validation tests

- [x] 4.1 Add e2e test in `tests/e2e/skills.test.ts`: `init --ci`, run `skills new NAME --ci` with no metadata flags, read file, assert `description: ''` and no `license`/`compatibility` keys (TC-SKILL-NEW-03)
- [x] 4.2 Add e2e test in `tests/e2e/skills.test.ts`: create skill, then run `skills new NAME --ci` again without `--force`, assert exit 1 and stderr contains "already exists" (TC-SKILL-NEW-04)
- [x] 4.3 Add e2e test in `tests/e2e/skills.test.ts`: create multiple skills, run `skills ls --json --skill NAME`, assert valid JSON array with exactly one matching element (TC-SKILL-LS-06)
- [x] 4.4 Add e2e test in `tests/e2e/skills.test.ts`: run `skills add NAME --runner maven`, assert exit 2 and stderr contains "invalid value" with valid runner list (TC-SKILL-ADD-05)
- [x] 4.5 Add e2e test in `tests/e2e/skills.test.ts`: run `skills add NAME --runner yarn` (yarn not on PATH), assert exit 1 and stderr mentions runner not found (TC-SKILL-ADD-04). PATH restricted to `/usr/bin:/bin` to guarantee yarn is absent.

## 5. Verification

- [x] 5.1 Run `mise check` (fmt + clippy) — exit 0, no warnings
- [x] 5.2 Run `mise tests` (unit + integration + e2e) — 202 passed, 202 total
