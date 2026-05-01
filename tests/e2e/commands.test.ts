import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "@microsoft/tui-test";
import { cleanup, makeTmpDir, run, shellProgram } from "./helpers.js";

// ── commands new – CLI ────────────────────────────────────────────────────────

test.describe("commands new CLI", () => {
	// C08: all flags populate frontmatter fields
	test("all flags populate frontmatter fields", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode } = run(
				[
					"commands",
					"new",
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

	// generated file contains expected sections
	test("command file contains Steps and When to use sections", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			run(["commands", "new", "greet", "--description", "Say hello"], d);
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
			run(["commands", "new", "greet", "--description", "first"], d);
			const { exitCode } = run(
				["commands", "new", "greet", "--description", "second", "--force"],
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
			run(["commands", "new", "greet", "--description", "first"], d);
			const { exitCode, stderr } = run(
				["commands", "new", "greet", "--description", "second"],
				d,
			);
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain("--force");
		} finally {
			cleanup(d);
		}
	});
});

// ── commands ls – CLI ─────────────────────────────────────────────────────────

test.describe("commands ls CLI", () => {
	// shows commands from init scaffold
	test("shows hello command from init scaffold", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode, stderr } = run(["commands", "ls"], d);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/hello/);
		} finally {
			cleanup(d);
		}
	});

	// shows count summary
	test("shows command count summary line", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode, stderr } = run(["commands", "ls"], d);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/command\(s\)/);
		} finally {
			cleanup(d);
		}
	});

	// C24: --full flag succeeds
	test("--full exits zero", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode } = run(["commands", "ls", "--full"], d);
			expect(exitCode).toBe(0);
		} finally {
			cleanup(d);
		}
	});
});

// ── commands rm – CLI ─────────────────────────────────────────────────────────

test.describe("commands rm CLI", () => {
	// C26: --force deletes the file
	test("--force deletes command file", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			expect(existsSync(join(d, ".dotagents-debug/commands/hello.md"))).toBe(
				true,
			);
			const { exitCode } = run(["commands", "rm", "hello", "--force"], d);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".dotagents-debug/commands/hello.md"))).toBe(
				false,
			);
		} finally {
			cleanup(d);
		}
	});

	// only the named command is removed
	test("only removes the named command, others remain", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			run(["commands", "new", "greet", "--description", "test"], d);
			expect(existsSync(join(d, ".dotagents-debug/commands/greet.md"))).toBe(
				true,
			);
			run(["commands", "rm", "greet", "--force"], d);
			expect(existsSync(join(d, ".dotagents-debug/commands/hello.md"))).toBe(
				true,
			);
			expect(existsSync(join(d, ".dotagents-debug/commands/greet.md"))).toBe(
				false,
			);
		} finally {
			cleanup(d);
		}
	});
});

// ── commands new – TUI ────────────────────────────────────────────────────────
// Each TUI test has its own describe block so test.use() is at describe level.
// Workspace setup runs synchronously at describe evaluation time.

// T06: all three prompts appear, deploy prompt defaults to No
test.describe("commands new TUI – T06 interactive prompts", () => {
	const d = makeTmpDir();
	run(["init", "--template", "starter"], d);
	test.use({ program: shellProgram(d, ["commands", "new", "greet"]) });

	test("prompts for description, category, tags then deploy", async ({
		terminal,
	}) => {
		try {
			await expect(terminal.getByText("dotagents commands new")).toBeVisible();
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
test.describe("commands new TUI – T07 deploy on Yes (skipped)", () => {
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

// ── commands rm – TUI ─────────────────────────────────────────────────────────

// T10: confirm prompt appears, navigating to Yes removes the file
test.describe("commands rm TUI – T10 confirm Yes", () => {
	const d = makeTmpDir();
	run(["init", "--template", "starter"], d);
	test.use({ program: shellProgram(d, ["commands", "rm", "hello"]) });

	test("confirm Yes removes the command", async ({ terminal }) => {
		try {
			await expect(terminal.getByText("Remove command 'hello'?")).toBeVisible();
			await expect(terminal.getByText("This cannot be undone.")).toBeVisible();
			// default is No; navigate up to Yes
			terminal.keyUp();
			terminal.keyPress("Enter");

			await expect(terminal.getByText("Removed")).toBeVisible();

			expect(existsSync(join(d, ".dotagents-debug/commands/hello.md"))).toBe(
				false,
			);
		} finally {
			cleanup(d);
		}
	});
});

// T11: pressing Enter on default No cancels the removal
test.describe("commands rm TUI – T11 confirm No", () => {
	const d = makeTmpDir();
	run(["init", "--template", "starter"], d);
	test.use({ program: shellProgram(d, ["commands", "rm", "hello"]) });

	test("confirm No leaves the command file intact", async ({ terminal }) => {
		try {
			await expect(terminal.getByText("Remove command 'hello'?")).toBeVisible();
			terminal.keyPress("Enter"); // accept default No
			await expect(terminal.getByText("Cancelled")).toBeVisible();

			expect(existsSync(join(d, ".dotagents-debug/commands/hello.md"))).toBe(
				true,
			);
		} finally {
			cleanup(d);
		}
	});
});
