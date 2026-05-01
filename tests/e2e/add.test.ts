import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "@microsoft/tui-test";
import { cleanup, makeTmpDir, run, shellProgram } from "./helpers.js";

// ── CLI flows ────────────────────────────────────────────────────────────────

test.describe("add command CLI", () => {
	// C08: add command with all flags populates frontmatter
	test("all flags populate frontmatter fields", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode } = run(
				[
					"add",
					"command",
					"greet",
					"--description",
					"Say hello",
					"--category",
					"Greetings",
					"--tags",
					"greeting,hello",
				],
				d,
			);
			expect(exitCode).toBe(0);
			const content = readFileSync(
				join(d, ".dotagents-debug/commands/greet.md"),
				"utf8",
			);
			expect(content).toContain('name: "greet"');
			expect(content).toContain('"Say hello"');
			expect(content).toContain("Greetings");
			expect(content).toContain("greeting");
		} finally {
			cleanup(d);
		}
	});

	// command file contains expected sections
	test("command file contains Steps and When to use sections", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			run(["add", "command", "greet", "--description", "Say hello"], d);
			const content = readFileSync(
				join(d, ".dotagents-debug/commands/greet.md"),
				"utf8",
			);
			expect(content).toContain("## When to use");
			expect(content).toContain("## Steps");
		} finally {
			cleanup(d);
		}
	});

	// C10: --force overwrites existing command
	test("--force overwrites existing command file", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			run(["add", "command", "greet", "--description", "first"], d);
			const { exitCode } = run(
				["add", "command", "greet", "--description", "second", "--force"],
				d,
			);
			expect(exitCode).toBe(0);
			const content = readFileSync(
				join(d, ".dotagents-debug/commands/greet.md"),
				"utf8",
			);
			expect(content).toContain('"second"');
		} finally {
			cleanup(d);
		}
	});

	// duplicate without --force fails
	test("duplicate without --force exits non-zero", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			run(["add", "command", "greet", "--description", "first"], d);
			const { exitCode, stderr } = run(
				["add", "command", "greet", "--description", "second"],
				d,
			);
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain("--force");
		} finally {
			cleanup(d);
		}
	});
});

test.describe("add skill CLI", () => {
	// C09: add skill with all flags populates frontmatter
	test("all flags populate frontmatter fields", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode } = run(
				[
					"add",
					"skill",
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
				join(d, ".dotagents-debug/skills/my-skill/SKILL.md"),
				"utf8",
			);
			expect(content).toContain("name: my-skill");
			expect(content).toContain('"Greet users"');
			expect(content).toContain("license: MIT");
			expect(content).toContain('"Requires node"');
			expect(content).toContain('version: "1.0"');
		} finally {
			cleanup(d);
		}
	});

	// skill file contains expected sections
	test("skill file contains Instructions and When to use sections", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			run(["add", "skill", "my-skill", "--description", "Greet users"], d);
			const content = readFileSync(
				join(d, ".dotagents-debug/skills/my-skill/SKILL.md"),
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
			run(["add", "skill", "my-skill", "--description", "first"], d);
			const { exitCode } = run(
				["add", "skill", "my-skill", "--description", "second", "--force"],
				d,
			);
			expect(exitCode).toBe(0);
			const content = readFileSync(
				join(d, ".dotagents-debug/skills/my-skill/SKILL.md"),
				"utf8",
			);
			expect(content).toContain('"second"');
		} finally {
			cleanup(d);
		}
	});
});

// ── TUI flows ────────────────────────────────────────────────────────────────
// Each TUI test has its own describe block so test.use() is at describe level.
// Workspace setup runs synchronously at describe evaluation time.

// T06: all three prompts appear, deploy prompt defaults to No
test.describe("add command TUI – T06 interactive prompts", () => {
	const d = makeTmpDir();
	run(["init", "--template", "starter"], d);
	test.use({ program: shellProgram(d, ["add", "command", "greet"]) });

	test("prompts for description, category, tags then deploy", async ({
		terminal,
	}) => {
		try {
			await expect(terminal.getByText("dotagents add command")).toBeVisible();
			await expect(terminal.getByText("Description")).toBeVisible();
			await expect(
				terminal.getByText("What does this command do?"),
			).toBeVisible();

			terminal.write("A greeting command");
			terminal.keyPress("Enter");

			await expect(terminal.getByText("Category")).toBeVisible();
			terminal.write("Greetings");
			terminal.keyPress("Enter");

			await expect(terminal.getByText("Tags (comma-separated)")).toBeVisible();
			terminal.write("greet,demo");
			terminal.keyPress("Enter");

			// wait for deploy prompt — it appears after the file is written
			await expect(terminal.getByText("Deploy now?")).toBeVisible();
			terminal.keyPress("Enter"); // accept default No

			// file created with the provided metadata
			expect(existsSync(join(d, ".dotagents-debug/commands/greet.md"))).toBe(
				true,
			);
			const content = readFileSync(
				join(d, ".dotagents-debug/commands/greet.md"),
				"utf8",
			);
			expect(content).toContain('"A greeting command"');
			expect(content).toContain("Greetings");
		} finally {
			cleanup(d);
		}
	});
});

// T07: answering Yes to "Deploy now?" triggers deploy.
// Skipped: the embedded deploy call shows its own offline-mode prompt, making
// the terminal interaction flow too complex for reliable TUI testing. The
// deploy functionality is covered by the deploy CLI tests (T15) and journey
// tests (J07).
test.describe("add command TUI – T07 deploy on Yes (skipped)", () => {
	// stub program — setup runs inside the (skipped) body so no filesystem
	// mutations happen at describe evaluation time
	test.use({ program: { file: "bash", args: ["-c", "true"] } });

	test.skip("answering Yes to deploy prompt runs deploy (deploy prompt is nested inside add)", async ({
		terminal,
	}) => {
		// setup deferred into body — never runs because test is skipped
		const d = makeTmpDir();
		run(["init", "--template", "with-custom-provider"], d);
		const lcPath = join(d, ".dotagents-debug/local.config.toml");
		writeFileSync(
			lcPath,
			readFileSync(lcPath, "utf8").replace(
				/targets\s*=\s*\["gemini"\]/,
				"targets = []",
			),
		);
		try {
			await expect(terminal.getByText("Description")).toBeVisible();
			terminal.write("Deploy test");
			terminal.keyPress("Enter");
			terminal.keyPress("Enter"); // category (empty)
			terminal.keyPress("Enter"); // tags (empty)
			await expect(terminal.getByText("Deploy now?")).toBeVisible();
			terminal.keyUp();
			terminal.keyPress("Enter"); // Yes to deploy
			await expect(terminal.getByText("Run in offline mode?")).toBeVisible();
			terminal.keyPress("Enter"); // accept online
		} finally {
			cleanup(d);
		}
	});
});

// T08: all three prompts appear for skill
test.describe("add skill TUI – T08 interactive prompts", () => {
	const d = makeTmpDir();
	run(["init", "--template", "starter"], d);
	test.use({ program: shellProgram(d, ["add", "skill", "my-skill"]) });

	test("prompts for description, license, compatibility", async ({
		terminal,
	}) => {
		try {
			await expect(terminal.getByText("dotagents add skill")).toBeVisible();
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

			expect(
				existsSync(join(d, ".dotagents-debug/skills/my-skill/SKILL.md")),
			).toBe(true);
		} finally {
			cleanup(d);
		}
	});
});
