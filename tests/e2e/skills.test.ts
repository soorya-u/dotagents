import { existsSync, readFileSync, rmSync } from "node:fs";
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
			run(["init", "--template", "starter"], d);
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
			run(["init", "--template", "starter"], d);
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
			run(["init", "--template", "starter"], d);
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
});

// ── skills ls – CLI ───────────────────────────────────────────────────────────

test.describe("skills ls CLI", () => {
	// shows skills from init scaffold
	test("shows hello-skill from init scaffold", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode, stderr } = run(["skills", "ls"], d);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/hello-skill/);
		} finally {
			cleanup(d);
		}
	});

	// shows count summary
	test("shows skill count summary line", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
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
			run(["init", "--template", "starter"], d);
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
			run(["init", "--template", "starter"], d);
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
			run(["init", "--template", "starter"], d);
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
			run(["init", "--template", "starter"], d);
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
			run(["init", "--template", "starter"], d);
			const { exitCode, stderr } = run(["skills", "ls", "--content"], d);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/var\.agent_name/);
		} finally {
			cleanup(d);
		}
	});

	// default (no --content) does NOT show body content
	test("default output does not show body content", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode, stderr } = run(["skills", "ls"], d);
			expect(exitCode).toBe(0);
			expect(stderr).not.toMatch(/var\.agent_name/);
		} finally {
			cleanup(d);
		}
	});

	// --json outputs pipeable JSON (no extra output)
	test("--json outputs pipeable JSON", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
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
			run(["init", "--template", "starter"], d);
			const { exitCode, stderr } = run(
				["skills", "ls", "--skill", "hello-skill"],
				d,
			);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/hello-skill/);
		} finally {
			cleanup(d);
		}
	});

	// --skill with no match shows "No skills found"
	test("--skill with unmatched name shows no skills found", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode, stderr } = run(
				["skills", "ls", "--skill", "nonexistent"],
				d,
			);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/No skills found/);
		} finally {
			cleanup(d);
		}
	});

	// --json with empty workspace outputs []
	test("--json with no skills outputs []", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
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
});

// ── skills rm – CLI ───────────────────────────────────────────────────────────

test.describe("skills rm CLI", () => {
	// C27: --force deletes the skill directory
	test("--force deletes skill directory", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
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
			run(["init", "--template", "starter"], d);
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
				["skills", "new", "my-skill", "--ci", "--no-deploy", "--description", "test"],
				d,
			);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".mycode/skills/my-skill/SKILL.md"))).toBe(false);
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
			expect(existsSync(join(d, ".mycode/skills/my-skill/SKILL.md"))).toBe(true);
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
			expect(existsSync(join(d, ".mycode/skills/hello-skill/SKILL.md"))).toBe(true);
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
			expect(existsSync(join(d, ".mycode/skills/hello-skill/SKILL.md"))).toBe(true);
			const { exitCode } = run(
				["skills", "rm", "hello-skill", "--ci", "--force"],
				d,
			);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".mycode/skills/hello-skill/SKILL.md"))).toBe(false);
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
			run(["init", "--template", "starter"], workspace);
			run(["skills", "new", "my-skill", "--description", "A skill"], workspace);
			const { exitCode, stderr } = run(
				["skills", "ls", "--cwd", workspace],
				cwd,
			);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/my-skill/);
		} finally {
			cleanup(cwd);
			cleanup(workspace);
		}
	});

	// skills new --cwd creates skill in target workspace
	test("skills new --cwd creates skill in target workspace", async () => {
		const { cwd, workspace } = makeTwoDirs();
		try {
			run(["init", "--template", "starter"], workspace);
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
			run(["init", "--template", "starter"], workspace);
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
	run(["init", "--template", "starter"], d);
	test.use({ program: shellProgram(d, ["skills", "new", "my-skill"]) });

	test("prompts for description, license, compatibility", async ({
		terminal,
	}) => {
		try {
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
		} finally {
			cleanup(d);
		}
	});
});

// ── skills rm – TUI ───────────────────────────────────────────────────────────

// T12: confirm Yes removes the skill directory
test.describe("skills rm TUI – T12 confirm Yes", () => {
	const d = makeTmpDir();
	run(["init", "--template", "starter"], d);
	test.use({ program: shellProgram(d, ["skills", "rm", "hello-skill"]) });

	test("confirm Yes removes the skill", async ({ terminal }) => {
		try {
			await expect(
				terminal.getByText("Remove skill 'hello-skill'?"),
			).toBeVisible();
			terminal.keyUp(); // navigate to Yes
			terminal.keyPress("Enter");

			await expect(terminal.getByText("Removed")).toBeVisible();

			expect(existsSync(join(d, ".dotagents/skills/hello-skill"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});
});

// T13: confirm No leaves skill intact
test.describe("skills rm TUI – T13 confirm No", () => {
	const d = makeTmpDir();
	run(["init", "--template", "starter"], d);
	test.use({ program: shellProgram(d, ["skills", "rm", "hello-skill"]) });

	test("confirm No leaves the skill directory intact", async ({ terminal }) => {
		try {
			await expect(
				terminal.getByText("Remove skill 'hello-skill'?"),
			).toBeVisible();
			terminal.keyPress("Enter"); // accept default No
			await expect(terminal.getByText("Cancelled")).toBeVisible();

			expect(existsSync(join(d, ".dotagents/skills/hello-skill"))).toBe(true);
		} finally {
			cleanup(d);
		}
	});
});
