import {
	existsSync,
	mkdirSync,
	readFileSync,
	rmSync,
	writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { expect, test } from "@microsoft/tui-test";
import {
	cleanup,
	initWithLocalProvider,
	makeTmpDir,
	makeTwoDirs,
	run,
	shellProgram,
} from "./helpers.js";

// ── skills new – CLI ──────────────────────────────────────────────────────────

test.describe("skills new CLI", () => {
	// C09: all flags populate frontmatter fields
	test("all flags populate frontmatter fields", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const { exitCode } = run(
				[
					"skills",
					"new",
					"my-skill",
					"--description",
					"Greet users",
					"--license",
					"MIT",
					"--compatibility",
					"Requires node",
				],
				d,
			);
			expect(exitCode).toBe(0);
			const content = readFileSync(
				join(d, ".dotagents/skills/my-skill/SKILL.md"),
				"utf8",
			);
			expect(content).toContain("name: my-skill");
			expect(content).toContain("Greet users");
			expect(content).toContain("license: MIT");
			expect(content).toContain("Requires node");
			expect(content).toContain("version: '1.0'");
		} finally {
			cleanup(d);
		}
	});

	// generated file contains expected sections
	test("skill file contains Instructions and When to use sections", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			run(["skills", "new", "my-skill", "--description", "Greet users"], d);
			const content = readFileSync(
				join(d, ".dotagents/skills/my-skill/SKILL.md"),
				"utf8",
			);
			expect(content).toContain("## When to use");
			expect(content).toContain("## Instructions");
		} finally {
			cleanup(d);
		}
	});

	// C11: --force overwrites existing skill
	test("--force overwrites existing skill", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			run(["skills", "new", "my-skill", "--description", "first"], d);
			const { exitCode } = run(
				["skills", "new", "my-skill", "--description", "second", "--force"],
				d,
			);
			expect(exitCode).toBe(0);
			const content = readFileSync(
				join(d, ".dotagents/skills/my-skill/SKILL.md"),
				"utf8",
			);
			expect(content).toContain("second");
		} finally {
			cleanup(d);
		}
	});

	// TC-SKILL-NEW-03: CI mode with no metadata flags produces empty defaults
	test("CI mode with no metadata flags produces empty defaults", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--ci"], d);
			const { exitCode } = run(["skills", "new", "test-skill", "--ci"], d);
			expect(exitCode).toBe(0);
			const content = readFileSync(
				join(d, ".dotagents/skills/test-skill/SKILL.md"),
				"utf8",
			);
			const frontmatterMatch = content.match(/^---\n([\s\S]*?)\n---/);
			expect(frontmatterMatch).not.toBeNull();
			const frontmatter = frontmatterMatch?.[1] ?? "";
			expect(frontmatter).toContain("description: ''");
			expect(frontmatter).not.toMatch(/^license:/m);
			expect(frontmatter).not.toMatch(/^compatibility:/m);
		} finally {
			cleanup(d);
		}
	});

	// TC-SKILL-NEW-04: duplicate skill without --force exits non-zero
	test("duplicate skill without --force exits non-zero", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			run(["skills", "new", "dup-skill", "--description", "first"], d);
			const { exitCode, stderr } = run(
				["skills", "new", "dup-skill", "--description", "second"],
				d,
			);
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain("already exists");
			expect(stderr).toContain("--force");
		} finally {
			cleanup(d);
		}
	});

	// TC-SKILL-NEW-06: --deploy (default CI auto-deploy) triggers deploy after creation.
	// Already covered by "CI auto-deploys after skills new" in deploy-default block.

	// TC-SKILL-RM-06: --deploy (default CI auto-deploy) re-runs deploy after removal.
	// Already covered by "CI auto-deploys after skills rm" in deploy-default block.
});

// ── skills ls – CLI ───────────────────────────────────────────────────────────

test.describe("skills ls CLI", () => {
	// shows skills from init scaffold
	test("shows hello-skill from init scaffold", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const { exitCode, stdout } = run(["skills", "ls"], d);
			expect(exitCode).toBe(0);
			expect(stdout).toMatch(/hello-skill/);
		} finally {
			cleanup(d);
		}
	});

	// shows count summary
	test("shows skill count summary line", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const { exitCode, stderr } = run(["skills", "ls"], d);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/skill\(s\)/);
		} finally {
			cleanup(d);
		}
	});

	// --content flag succeeds
	test("--content exits zero", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const { exitCode } = run(["skills", "ls", "--content"], d);
			expect(exitCode).toBe(0);
		} finally {
			cleanup(d);
		}
	});

	// --json outputs valid JSON array with frontmatter fields
	test("--json outputs valid JSON array with name and description", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const { exitCode, stdout } = run(["skills", "ls", "--json"], d);
			expect(exitCode).toBe(0);
			const parsed = JSON.parse(stdout);
			expect(Array.isArray(parsed)).toBe(true);
			expect(parsed.length).toBeGreaterThanOrEqual(1);
			expect(parsed[0]).toHaveProperty("name");
			expect(parsed[0]).toHaveProperty("description");
		} finally {
			cleanup(d);
		}
	});

	// --json does not include content key without --content
	test("--json without --content does not include content key", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const { exitCode, stdout } = run(["skills", "ls", "--json"], d);
			expect(exitCode).toBe(0);
			const parsed = JSON.parse(stdout);
			expect(parsed[0]).not.toHaveProperty("content");
		} finally {
			cleanup(d);
		}
	});

	// --json --content includes content key with body
	test("--json --content includes content key with body", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const { exitCode, stdout } = run(
				["skills", "ls", "--json", "--content"],
				d,
			);
			expect(exitCode).toBe(0);
			const parsed = JSON.parse(stdout);
			expect(parsed[0]).toHaveProperty("content");
			expect(typeof parsed[0].content).toBe("string");
			expect(parsed[0].content.length).toBeGreaterThan(0);
		} finally {
			cleanup(d);
		}
	});

	// --content shows body content in text output
	test("--content shows body content in text output", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const { exitCode, stdout } = run(["skills", "ls", "--content"], d);
			expect(exitCode).toBe(0);
			expect(stdout).toMatch(/Hello Skill/);
		} finally {
			cleanup(d);
		}
	});

	// default (no --content) does NOT show body content
	test("default output does not show body content", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const { exitCode, stdout } = run(["skills", "ls"], d);
			expect(exitCode).toBe(0);
			expect(stdout).not.toMatch(/var\.agent_name/);
		} finally {
			cleanup(d);
		}
	});

	// --json outputs pipeable JSON (no extra output)
	test("--json outputs pipeable JSON", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const { exitCode, stdout } = run(["skills", "ls", "--json"], d);
			expect(exitCode).toBe(0);
			const parsed = JSON.parse(stdout);
			expect(Array.isArray(parsed)).toBe(true);
		} finally {
			cleanup(d);
		}
	});

	// --skill filters by skill name
	test("--skill filters to matching skill", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const { exitCode, stdout } = run(
				["skills", "ls", "--skill", "hello-skill"],
				d,
			);
			expect(exitCode).toBe(0);
			expect(stdout).toMatch(/hello-skill/);
		} finally {
			cleanup(d);
		}
	});

	// --skill with no match shows "No skills found"
	test("--skill with unmatched name shows no skills found", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const { exitCode, stdout } = run(
				["skills", "ls", "--skill", "nonexistent"],
				d,
			);
			expect(exitCode).toBe(0);
			expect(stdout).toMatch(/No skills found/);
		} finally {
			cleanup(d);
		}
	});

	// --json with empty workspace outputs []
	test("--json with no skills outputs []", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			// Remove the scaffolded hello-skill to make it empty
			rmSync(join(d, ".dotagents/skills/hello-skill"), {
				recursive: true,
				force: true,
			});
			const { exitCode, stdout } = run(["skills", "ls", "--json"], d);
			expect(exitCode).toBe(0);
			expect(stdout.trim()).toBe("[]");
		} finally {
			cleanup(d);
		}
	});

	// TC-SKILL-LS-06: --json --skill combined filter returns filtered JSON array
	test("--json --skill returns filtered JSON array with one element", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			run(["skills", "new", "another-skill", "--description", "Another"], d);
			const { exitCode, stdout } = run(
				["skills", "ls", "--json", "--skill", "hello-skill"],
				d,
			);
			expect(exitCode).toBe(0);
			const parsed = JSON.parse(stdout);
			expect(Array.isArray(parsed)).toBe(true);
			expect(parsed).toHaveLength(1);
			expect(parsed[0].name).toBe("hello-skill");
		} finally {
			cleanup(d);
		}
	});
});

// ── skills rm – CLI ───────────────────────────────────────────────────────────

test.describe("skills rm CLI", () => {
	// C27: --force deletes the skill directory
	test("--force deletes skill directory", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			expect(existsSync(join(d, ".dotagents/skills/hello-skill"))).toBe(true);
			const { exitCode } = run(["skills", "rm", "hello-skill", "--force"], d);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".dotagents/skills/hello-skill"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});

	// 6.1: skills rm removes deployed file and gitignore entry after deploy
	test("--force removes deployed skill file and gitignore entry", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--gitignore"], d);

			expect(existsSync(join(d, ".mycode/skills/hello-skill/SKILL.md"))).toBe(
				true,
			);

			const giBefore = readFileSync(join(d, ".gitignore"), "utf8");
			expect(giBefore).toContain(".mycode/");

			const { exitCode } = run(["skills", "rm", "hello-skill", "--force"], d);
			expect(exitCode).toBe(0);

			expect(existsSync(join(d, ".mycode/skills/hello-skill/SKILL.md"))).toBe(
				false,
			);

			const giAfter = readFileSync(join(d, ".gitignore"), "utf8");
			expect(giAfter).toContain(".mycode/");
		} finally {
			cleanup(d);
		}
	});

	// 6.3: skills rm warns when skill was never deployed
	test("--force exits 0 and warns when skill was never deployed", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const { exitCode, stderr } = run(
				["skills", "rm", "hello-skill", "--force"],
				d,
			);
			expect(exitCode).toBe(0);
			expect(stderr).toContain("No deployed files found");
		} finally {
			cleanup(d);
		}
	});
});

// ── skills deploy-default behavior ─────────────────────────────────────────────

test.describe("skills deploy-default", () => {
	// --no-deploy skips auto-deploy in CI
	test("--no-deploy skips deploy after skills new", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode } = run(
				[
					"skills",
					"new",
					"my-skill",
					"--ci",
					"--no-deploy",
					"--description",
					"test",
				],
				d,
			);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".mycode/skills/my-skill/SKILL.md"))).toBe(
				false,
			);
		} finally {
			cleanup(d);
		}
	});

	// CI auto-deploys after skills new
	test("CI auto-deploys after skills new", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode } = run(
				["skills", "new", "my-skill", "--ci", "--description", "test"],
				d,
			);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".mycode/skills/my-skill/SKILL.md"))).toBe(
				true,
			);
		} finally {
			cleanup(d);
		}
	});

	// --no-deploy skips redeploy after skills rm
	test("--no-deploy skips deploy after skills rm", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--gitignore"], d);
			expect(existsSync(join(d, ".mycode/skills/hello-skill/SKILL.md"))).toBe(
				true,
			);
			const { exitCode } = run(
				["skills", "rm", "hello-skill", "--ci", "--no-deploy", "--force"],
				d,
			);
			expect(exitCode).toBe(0);
		} finally {
			cleanup(d);
		}
	});

	// CI auto-deploys after skills rm
	test("CI auto-deploys after skills rm", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--gitignore"], d);
			expect(existsSync(join(d, ".mycode/skills/hello-skill/SKILL.md"))).toBe(
				true,
			);
			const { exitCode } = run(
				["skills", "rm", "hello-skill", "--ci", "--force"],
				d,
			);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".mycode/skills/hello-skill/SKILL.md"))).toBe(
				false,
			);
		} finally {
			cleanup(d);
		}
	});
});

// ── skills deploy – new fields ─────────────────────────────────────────────────

test.describe("skills deploy new fields", () => {
	// deployed output contains metadata field when present
	test("deployed output contains metadata when present", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const skillDir = join(d, ".dotagents/skills/meta-skill");
			mkdirSync(skillDir, { recursive: true });
			writeFileSync(
				join(skillDir, "SKILL.md"),
				`---
name: meta-skill
description: Has metadata
metadata:
  author: tester
  version: "2.0"
---

Body content`,
			);
			run(["deploy", "--offline"], d);
			const deployed = readFileSync(
				join(d, ".mycode/skills/meta-skill/SKILL.md"),
				"utf8",
			);
			expect(deployed).toContain("metadata:");
			expect(deployed).toContain("author: tester");
			expect(deployed).toContain("version:");
		} finally {
			cleanup(d);
		}
	});

	// deployed output omits metadata when absent
	test("deployed output omits metadata when absent", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const skillDir = join(d, ".dotagents/skills/no-meta");
			mkdirSync(skillDir, { recursive: true });
			writeFileSync(
				join(skillDir, "SKILL.md"),
				`---
name: no-meta
description: No metadata
---

Body`,
			);
			run(["deploy", "--offline"], d);
			const deployed = readFileSync(
				join(d, ".mycode/skills/no-meta/SKILL.md"),
				"utf8",
			);
			expect(deployed).not.toContain("metadata:");
		} finally {
			cleanup(d);
		}
	});

	// deployed output contains disable-model-invocation when true
	test("deployed output contains disable-model-invocation when true", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const skillDir = join(d, ".dotagents/skills/dmi-skill");
			mkdirSync(skillDir, { recursive: true });
			writeFileSync(
				join(skillDir, "SKILL.md"),
				`---
name: dmi-skill
description: Has disable-model-invocation
disable-model-invocation: true
---

Body`,
			);
			run(["deploy", "--offline"], d);
			const deployed = readFileSync(
				join(d, ".mycode/skills/dmi-skill/SKILL.md"),
				"utf8",
			);
			expect(deployed).toContain("disable-model-invocation: true");
		} finally {
			cleanup(d);
		}
	});

	// deployed output omits disable-model-invocation when absent
	test("deployed output omits disable-model-invocation when absent", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const skillDir = join(d, ".dotagents/skills/no-dmi");
			mkdirSync(skillDir, { recursive: true });
			writeFileSync(
				join(skillDir, "SKILL.md"),
				`---
name: no-dmi
description: No dmi
---

Body`,
			);
			run(["deploy", "--offline"], d);
			const deployed = readFileSync(
				join(d, ".mycode/skills/no-dmi/SKILL.md"),
				"utf8",
			);
			expect(deployed).not.toContain("disable-model-invocation");
		} finally {
			cleanup(d);
		}
	});

	// deployed output contains disable-model-invocation when false
	test("deployed output contains disable-model-invocation when false", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const skillDir = join(d, ".dotagents/skills/dmi-false");
			mkdirSync(skillDir, { recursive: true });
			writeFileSync(
				join(skillDir, "SKILL.md"),
				`---
name: dmi-false
description: Has disable-model-invocation false
disable-model-invocation: false
---

Body`,
			);
			run(["deploy", "--offline"], d);
			const deployed = readFileSync(
				join(d, ".mycode/skills/dmi-false/SKILL.md"),
				"utf8",
			);
			expect(deployed).toContain("disable-model-invocation: false");
		} finally {
			cleanup(d);
		}
	});

	// deployed output contains user-invocable when set
	test("deployed output contains user-invocable when set", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const skillDir = join(d, ".dotagents/skills/ui-skill");
			mkdirSync(skillDir, { recursive: true });
			writeFileSync(
				join(skillDir, "SKILL.md"),
				`---
name: ui-skill
description: Has user-invocable
user-invocable: true
---

Body`,
			);
			run(["deploy", "--offline"], d);
			const deployed = readFileSync(
				join(d, ".mycode/skills/ui-skill/SKILL.md"),
				"utf8",
			);
			expect(deployed).toContain("user-invocable: true");
		} finally {
			cleanup(d);
		}
	});

	// deployed output omits user-invocable when absent
	test("deployed output omits user-invocable when absent", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const skillDir = join(d, ".dotagents/skills/no-ui");
			mkdirSync(skillDir, { recursive: true });
			writeFileSync(
				join(skillDir, "SKILL.md"),
				`---
name: no-ui
description: No ui
---

Body`,
			);
			run(["deploy", "--offline"], d);
			const deployed = readFileSync(
				join(d, ".mycode/skills/no-ui/SKILL.md"),
				"utf8",
			);
			expect(deployed).not.toContain("user-invocable");
		} finally {
			cleanup(d);
		}
	});

	// deployed output contains user-invocable when false
	test("deployed output contains user-invocable when false", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const skillDir = join(d, ".dotagents/skills/ui-false");
			mkdirSync(skillDir, { recursive: true });
			writeFileSync(
				join(skillDir, "SKILL.md"),
				`---
name: ui-false
description: Has user-invocable false
user-invocable: false
---

Body`,
			);
			run(["deploy", "--offline"], d);
			const deployed = readFileSync(
				join(d, ".mycode/skills/ui-false/SKILL.md"),
				"utf8",
			);
			expect(deployed).toContain("user-invocable: false");
		} finally {
			cleanup(d);
		}
	});

	// deployed output contains paths when set
	test("deployed output contains paths when set", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const skillDir = join(d, ".dotagents/skills/paths-skill");
			mkdirSync(skillDir, { recursive: true });
			writeFileSync(
				join(skillDir, "SKILL.md"),
				`---
name: paths-skill
description: Has paths
paths:
  - src/**/*.rs
  - tests/**
---

Body`,
			);
			run(["deploy", "--offline"], d);
			const deployed = readFileSync(
				join(d, ".mycode/skills/paths-skill/SKILL.md"),
				"utf8",
			);
			expect(deployed).toContain("paths:");
			expect(deployed).toContain("- src/**/*.rs");
			expect(deployed).toContain("- tests/**");
		} finally {
			cleanup(d);
		}
	});

	// deployed output omits paths when absent
	test("deployed output omits paths when absent", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const skillDir = join(d, ".dotagents/skills/no-paths");
			mkdirSync(skillDir, { recursive: true });
			writeFileSync(
				join(skillDir, "SKILL.md"),
				`---
name: no-paths
description: No paths
---

Body`,
			);
			run(["deploy", "--offline"], d);
			const deployed = readFileSync(
				join(d, ".mycode/skills/no-paths/SKILL.md"),
				"utf8",
			);
			expect(deployed).not.toMatch(/^paths:/m);
		} finally {
			cleanup(d);
		}
	});
});

// ── skills --cwd ──────────────────────────────────────────────────────────────

test.describe("skills --cwd", () => {
	// skills ls --cwd reads from target workspace
	test("skills ls --cwd reads from target workspace", async () => {
		const { cwd, workspace } = makeTwoDirs();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				workspace,
			);
			run(["skills", "new", "my-skill", "--description", "A skill"], workspace);
			const { exitCode, stdout } = run(
				["skills", "ls", "--cwd", workspace],
				cwd,
			);
			expect(exitCode).toBe(0);
			expect(stdout).toMatch(/my-skill/);
		} finally {
			cleanup(cwd);
			cleanup(workspace);
		}
	});

	// skills new --cwd creates skill in target workspace
	test("skills new --cwd creates skill in target workspace", async () => {
		const { cwd, workspace } = makeTwoDirs();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				workspace,
			);
			const { exitCode } = run(
				[
					"skills",
					"new",
					"my-skill",
					"--cwd",
					workspace,
					"--description",
					"Test",
				],
				cwd,
			);
			expect(exitCode).toBe(0);
			const file = join(workspace, ".dotagents/skills/my-skill/SKILL.md");
			expect(existsSync(file)).toBe(true);
			const content = readFileSync(file, "utf8");
			expect(content).toContain("name: my-skill");
		} finally {
			cleanup(cwd);
			cleanup(workspace);
		}
	});

	// skills rm --cwd removes from target workspace
	test("skills rm --cwd removes from target workspace", async () => {
		const { cwd, workspace } = makeTwoDirs();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				workspace,
			);
			run(["skills", "new", "my-skill", "--description", "Test"], workspace);
			const skillDir = join(workspace, ".dotagents/skills/my-skill");
			expect(existsSync(skillDir)).toBe(true);

			const { exitCode } = run(
				["skills", "rm", "my-skill", "--force", "--cwd", workspace],
				cwd,
			);
			expect(exitCode).toBe(0);
			expect(existsSync(skillDir)).toBe(false);
		} finally {
			cleanup(cwd);
			cleanup(workspace);
		}
	});
});

// ── skills new – TUI ──────────────────────────────────────────────────────────
// Each TUI test has its own describe block so test.use() is at describe level.
// Workspace setup runs synchronously at describe evaluation time.

// T08: all three prompts appear for skill
test.describe("skills new TUI – T08 interactive prompts", () => {
	const d = makeTmpDir();
	run(
		[
			"init",
			"--template",
			"starter",
			"--features",
			"command,instruction,mcp,skill",
		],
		d,
	);
	test.use({ program: shellProgram(d, ["skills", "new", "my-skill"]) });

	test("prompts for description, license, compatibility", async ({
		terminal,
	}) => {
		await expect(terminal.getByText("Description")).toBeVisible();

		terminal.write("A skill description");
		terminal.keyPress("Enter");

		await expect(terminal.getByText("License")).toBeVisible();
		terminal.write("MIT");
		terminal.keyPress("Enter");

		await expect(terminal.getByText("Compatibility")).toBeVisible();
		terminal.write("Requires node");
		terminal.keyPress("Enter");

		// wait for deploy prompt — it appears after the file is written
		await expect(terminal.getByText("Deploy now?")).toBeVisible();
		terminal.keyPress("Enter"); // accept default No

		expect(existsSync(join(d, ".dotagents/skills/my-skill/SKILL.md"))).toBe(
			true,
		);
	});
});

// ── skills rm – TUI ───────────────────────────────────────────────────────────

// T12: confirm Yes removes the skill directory
test.describe("skills rm TUI – T12 confirm Yes", () => {
	const d = makeTmpDir();
	run(
		[
			"init",
			"--template",
			"starter",
			"--features",
			"command,instruction,mcp,skill",
		],
		d,
	);
	test.use({ program: shellProgram(d, ["skills", "rm", "hello-skill"]) });

	test("confirm Yes removes the skill", async ({ terminal }) => {
		await expect(
			terminal.getByText("Remove skill 'hello-skill'?"),
		).toBeVisible();
		terminal.keyUp(); // navigate to Yes
		terminal.keyPress("Enter");

		await expect(terminal.getByText("Removed")).toBeVisible();

		expect(existsSync(join(d, ".dotagents/skills/hello-skill"))).toBe(false);
	});
});

// ── skills add – CLI ────────────────────────────────────────────────────────────
// `skills add` is always available (no longer feature-gated).
// Tests that require network access to the skills.sh registry are skipped.
test.describe("skills add CLI", () => {
	// skills add requires network; skip in CI
	test.skip("exits promptly in CI mode (does not hang)", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode, stderr } = run(
				["skills", "add", "some-skill", "--ci"],
				d,
				{
					DOTAGENTS_CI: "true",
				},
			);
			expect(typeof exitCode).toBe("number");
			expect(stderr).not.toMatch(/confirm/i);
		} finally {
			cleanup(d);
		}
	});

	// TC-SKILL-ADD-05: invalid --runner value exits with Clap error
	test("--runner maven exits 2 with invalid value error", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode, stderr } = run(
				["skills", "add", "test-skill", "--runner", "maven"],
				d,
			);
			expect(exitCode).toBe(2);
			expect(stderr.toLowerCase()).toContain("invalid");
		} finally {
			cleanup(d);
		}
	});

	// TC-SKILL-ADD-04: --runner yarn not on PATH exits non-zero with helpful error
	test("--runner yarn not on PATH exits non-zero", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode, stderr } = run(
				["skills", "add", "test-skill", "--runner", "yarn"],
				d,
				{
					PATH: "/usr/bin:/bin",
				},
			);
			expect(exitCode).not.toBe(0);
			expect(stderr).toMatch(/yarn/i);
		} finally {
			cleanup(d);
		}
	});
});

// T13: confirm No leaves skill intact
test.describe("skills rm TUI – T13 confirm No", () => {
	const d = makeTmpDir();
	run(
		[
			"init",
			"--template",
			"starter",
			"--features",
			"command,instruction,mcp,skill",
		],
		d,
	);
	test.use({ program: shellProgram(d, ["skills", "rm", "hello-skill"]) });

	test("confirm No leaves the skill directory intact", async ({ terminal }) => {
		await expect(
			terminal.getByText("Remove skill 'hello-skill'?"),
		).toBeVisible();
		terminal.keyPress("Enter"); // accept default No
		await expect(terminal.getByText("Cancelled")).toBeVisible();

		expect(existsSync(join(d, ".dotagents/skills/hello-skill"))).toBe(true);
	});
});

// ── integrations-skills-sh – config & rm provenance ─────────────────────────────

test.describe("integrations config", () => {
	// top-level package-runner (old format) is silently ignored, config still parses
	test("top-level package-runner in config is silently ignored", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--ci"], d);
			const rootDir = existsSync(join(d, ".dotagents-debug"))
				? ".dotagents-debug"
				: ".dotagents";
			const configPath = join(d, rootDir, "config.toml");
			const configContent = readFileSync(configPath, "utf8");
			writeFileSync(configPath, `${configContent}\npackage-runner = "bun"\n`);
			const { exitCode, stdout } = run(["config"], d);
			expect(exitCode).toBe(0);
			// top-level field is ignored; no "Package runner" line appears
			expect(stdout).not.toMatch(/Package runner/);
		} finally {
			cleanup(d);
		}
	});

	// [integrations.skills-sh] table in config is parsed and displayed
	test("integrations skills-sh table is parsed and displayed", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--ci"], d);
			const rootDir = existsSync(join(d, ".dotagents-debug"))
				? ".dotagents-debug"
				: ".dotagents";
			const configPath = join(d, rootDir, "config.toml");
			const configContent = readFileSync(configPath, "utf8");
			writeFileSync(
				configPath,
				`${configContent}\n[integrations.skills-sh]\npackage-runner = "bun"\n`,
			);
			const { exitCode, stdout } = run(["config"], d);
			expect(exitCode).toBe(0);
			expect(stdout).toMatch(/Package runner/);
			expect(stdout).toMatch(/bun/);
		} finally {
			cleanup(d);
		}
	});
});

test.describe("skills rm provenance", () => {
	// skills rm on a locally-created skill (no lockfile) takes the local path
	test("locally-created skill is removed via local path", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			run(["skills", "new", "local-skill", "--description", "test"], d);
			// Verify no skills-lock.json exists (local skill, not installed via skills add)
			const rootDir = existsSync(join(d, ".dotagents-debug"))
				? ".dotagents-debug"
				: ".dotagents";
			expect(existsSync(join(d, rootDir, "skills-lock.json"))).toBe(false);

			const { exitCode } = run(["skills", "rm", "local-skill", "--force"], d);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, rootDir, "skills/local-skill"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});

	// skills rm with a lockfile entry triggers external delegation (requires npx)
	test.skip("skills rm with lockfile present delegates to skills CLI", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const rootDir = existsSync(join(d, ".dotagents-debug"))
				? ".dotagents-debug"
				: ".dotagents";
			// Create a fake lockfile to simulate an externally-installed skill
			writeFileSync(
				join(d, rootDir, "skills-lock.json"),
				JSON.stringify({
					version: 1,
					skills: {
						"hello-skill": {
							source: "vercel-labs/skills",
							sourceType: "github",
							skillPath: "skills/hello-skill/SKILL.md",
							computedHash: "abc123",
						},
					},
				}),
			);
			// is_external returns true → delegates to `npx skills remove`
			const { exitCode } = run(["skills", "rm", "hello-skill", "--force"], d);
			expect(exitCode).toBe(0);
		} finally {
			cleanup(d);
		}
	});
});

// ── skills add – integrations (network-dependent) ─────────────────────────────
// These tests require network access to run `npx skills add`. They are skipped.

test.describe("skills add integrations", () => {
	// 10.1: skills add lands files in .dotagents-debug/skills/<name>/SKILL.md
	test.skip("skills add installs into .dotagents/skills/ with openclaw+copy", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--ci"], d);
			const { exitCode } = run(
				["skills", "add", "vercel-labs/skills@find-skills", "--ci"],
				d,
			);
			expect(exitCode).toBe(0);
			const rootDir = existsSync(join(d, ".dotagents-debug"))
				? ".dotagents-debug"
				: ".dotagents";
			expect(existsSync(join(d, rootDir, "skills/find-skills/SKILL.md"))).toBe(
				true,
			);
			expect(existsSync(join(d, ".claude/skills"))).toBe(false);
			expect(existsSync(join(d, rootDir, "skills-lock.json"))).toBe(true);
		} finally {
			cleanup(d);
		}
	});

	// 10.2: skills add with --runner flag
	test.skip("skills add --runner uses specified runner", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--ci"], d);
			const { exitCode } = run(
				[
					"skills",
					"add",
					"vercel-labs/skills@find-skills",
					"--ci",
					"--runner",
					"npx",
				],
				d,
			);
			expect(exitCode).toBe(0);
		} finally {
			cleanup(d);
		}
	});

	// 10.3: skills rm external path delegates to skills CLI
	test.skip("skills rm external skill delegates to skills CLI", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--ci"], d);
			run(["skills", "add", "vercel-labs/skills@find-skills", "--ci"], d);
			const { exitCode } = run(["skills", "rm", "find-skills", "--force"], d);
			expect(exitCode).toBe(0);
		} finally {
			cleanup(d);
		}
	});
});
