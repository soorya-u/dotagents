## 1. Commands --deploy flag tests

- [ ] 1.1 Add e2e test in `tests/e2e/commands.test.ts`: `initWithLocalProvider`, run `commands new NAME --deploy --ci --offline --no-gitignore`, assert command file created and deployed output exists in `.mycode/commands/` (TC-CMD-NEW-06)
- [ ] 1.2 Add e2e test in `tests/e2e/commands.test.ts`: `initWithLocalProvider`, create multiple commands, deploy, run `commands rm NAME --force --deploy --ci --offline --no-gitignore`, assert named command deleted, remaining commands still deployed (TC-CMD-RM-06)

## 2. Commands error path tests

- [ ] 2.1 Add e2e test in `tests/e2e/commands.test.ts`: `init --ci`, run `commands new NAME --ci` with no metadata flags, read file, assert `description: ''` in frontmatter and no `category`/`tags` keys (TC-CMD-NEW-03)
- [ ] 2.2 Add e2e test in `tests/e2e/commands.test.ts`: run `commands new NAME --cwd /tmp/nonexistent --ci`, assert exit 1 and stderr mentions missing `.dotagents` (TC-CMD-NEW-10)

## 3. Skills --deploy flag tests

- [ ] 3.1 Add e2e test in `tests/e2e/skills.test.ts`: `initWithLocalProvider`, run `skills new NAME --deploy --ci --offline --no-gitignore`, assert skill dir created and deployed output exists in `.mycode/skills/` (TC-SKILL-NEW-06)
- [ ] 3.2 Add e2e test in `tests/e2e/skills.test.ts`: `initWithLocalProvider`, create multiple skills, deploy, run `skills rm NAME --force --deploy --ci --offline --no-gitignore`, assert named skill deleted, remaining skills still deployed (TC-SKILL-RM-06)

## 4. Skills validation tests

- [ ] 4.1 Add e2e test in `tests/e2e/skills.test.ts`: `init --ci`, run `skills new NAME --ci` with no metadata flags, read file, assert `description: ''` and no `license`/`compatibility` keys (TC-SKILL-NEW-03)
- [ ] 4.2 Add e2e test in `tests/e2e/skills.test.ts`: create skill, then run `skills new NAME --ci` again without `--force`, assert exit 1 and stderr contains "already exists" (TC-SKILL-NEW-04)
- [ ] 4.3 Add e2e test in `tests/e2e/skills.test.ts`: create multiple skills, run `skills ls --json --skill NAME`, assert valid JSON array with exactly one matching element (TC-SKILL-LS-06)
- [ ] 4.4 Add e2e test in `tests/e2e/skills.test.ts`: run `skills add NAME --runner maven`, assert exit 2 and stderr contains "invalid value" with valid runner list (TC-SKILL-ADD-05)
- [ ] 4.5 Add e2e test in `tests/e2e/skills.test.ts`: run `skills add NAME --runner yarn` (yarn not on PATH), assert exit 1 and stderr mentions runner not found (TC-SKILL-ADD-04). Skip if yarn is detected on PATH.

## 5. Verification

- [ ] 5.1 Run `mise check` (fmt + clippy) — must exit 0
- [ ] 5.2 Run `mise tests` (unit + integration + e2e) — must exit 0
