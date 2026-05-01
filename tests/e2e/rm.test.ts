import { existsSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "@microsoft/tui-test";
import { cleanup, makeTmpDir, run, shellProgram } from "./helpers.js";

// ── CLI flows ────────────────────────────────────────────────────────────────

test.describe("rm command CLI", () => {
	// C26: rm command --force deletes the file
	test("--force deletes command file", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			expect(existsSync(join(d, ".dotagents-debug/commands/hello.md"))).toBe(
				true,
			);
			const { exitCode } = run(["rm", "command", "hello", "--force"], d);
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
			run(["add", "command", "greet", "--description", "test"], d);
			run(["rm", "command", "greet", "--force"], d);
			// hello.md from init scaffold still exists
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

test.describe("rm skill CLI", () => {
	// C27: rm skill --force deletes the directory
	test("--force deletes skill directory", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			expect(existsSync(join(d, ".dotagents-debug/skills/hello-skill"))).toBe(
				true,
			);
			const { exitCode } = run(["rm", "skill", "hello-skill", "--force"], d);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".dotagents-debug/skills/hello-skill"))).toBe(
				false,
			);
		} finally {
			cleanup(d);
		}
	});
});

// ── TUI flows ────────────────────────────────────────────────────────────────
// Each TUI test has its own describe block so test.use() is at describe level.
// Workspace setup runs synchronously at describe evaluation time.

// T10: confirm prompt appears, navigating to Yes removes the file
test.describe("rm command TUI – T10 confirm Yes", () => {
	const d = makeTmpDir();
	run(["init", "--template", "starter"], d);
	test.use({ program: shellProgram(d, ["rm", "command", "hello"]) });

	test("confirm Yes removes the command", async ({ terminal }) => {
		try {
			// prompt text observed in discovery
			await expect(terminal.getByText("Remove command 'hello'?")).toBeVisible();
			await expect(terminal.getByText("This cannot be undone.")).toBeVisible();
			// default is No; navigate up to Yes
			terminal.keyUp();
			terminal.keyPress("Enter");

			// wait for the success message confirming deletion before checking filesystem
			await expect(terminal.getByText("Removed")).toBeVisible();

			// file is gone
			expect(existsSync(join(d, ".dotagents-debug/commands/hello.md"))).toBe(
				false,
			);
		} finally {
			cleanup(d);
		}
	});
});

// T11: pressing Enter on default No cancels the removal
test.describe("rm command TUI – T11 confirm No", () => {
	const d = makeTmpDir();
	run(["init", "--template", "starter"], d);
	test.use({ program: shellProgram(d, ["rm", "command", "hello"]) });

	test("confirm No leaves the command file intact", async ({ terminal }) => {
		try {
			await expect(terminal.getByText("Remove command 'hello'?")).toBeVisible();
			// press Enter accepts the default No
			terminal.keyPress("Enter");
			await expect(terminal.getByText("Cancelled")).toBeVisible();

			// file is still there
			expect(existsSync(join(d, ".dotagents-debug/commands/hello.md"))).toBe(
				true,
			);
		} finally {
			cleanup(d);
		}
	});
});

// T12: confirm Yes removes the skill directory
test.describe("rm skill TUI – T12 confirm Yes", () => {
	const d = makeTmpDir();
	run(["init", "--template", "starter"], d);
	test.use({ program: shellProgram(d, ["rm", "skill", "hello-skill"]) });

	test("confirm Yes removes the skill", async ({ terminal }) => {
		try {
			await expect(
				terminal.getByText("Remove skill 'hello-skill'?"),
			).toBeVisible();
			terminal.keyUp(); // navigate to Yes
			terminal.keyPress("Enter");

			// wait for the success message confirming deletion before checking filesystem
			await expect(terminal.getByText("Removed")).toBeVisible();

			expect(existsSync(join(d, ".dotagents-debug/skills/hello-skill"))).toBe(
				false,
			);
		} finally {
			cleanup(d);
		}
	});
});

// T13: confirm No leaves skill intact
test.describe("rm skill TUI – T13 confirm No", () => {
	const d = makeTmpDir();
	run(["init", "--template", "starter"], d);
	test.use({ program: shellProgram(d, ["rm", "skill", "hello-skill"]) });

	test("confirm No leaves the skill directory intact", async ({ terminal }) => {
		try {
			await expect(
				terminal.getByText("Remove skill 'hello-skill'?"),
			).toBeVisible();
			terminal.keyPress("Enter"); // accept default No
			await expect(terminal.getByText("Cancelled")).toBeVisible();

			expect(existsSync(join(d, ".dotagents-debug/skills/hello-skill"))).toBe(
				true,
			);
		} finally {
			cleanup(d);
		}
	});
});
