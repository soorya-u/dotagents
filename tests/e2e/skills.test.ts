import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "@microsoft/tui-test";
import {
	cleanup,
	initWithLocalProvider,
	makeTmpDir,
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

	// --full flag succeeds
	test("--full exits zero", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode } = run(["skills", "ls", "--full"], d);
			expect(exitCode).toBe(0);
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
			expect(giBefore).toContain(".mycode/skills/hello-skill/SKILL.md");

			const { exitCode } = run(["skills", "rm", "hello-skill", "--force"], d);
			expect(exitCode).toBe(0);

			expect(existsSync(join(d, ".mycode/skills/hello-skill/SKILL.md"))).toBe(
				false,
			);

			const giAfter = readFileSync(join(d, ".gitignore"), "utf8");
			expect(giAfter).not.toContain(".mycode/skills/hello-skill/SKILL.md");
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
