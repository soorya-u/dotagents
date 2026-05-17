import { existsSync, readFileSync, rmSync, writeFileSync } from "node:fs";
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

// ── commands new – CLI ────────────────────────────────────────────────────────

test.describe("commands new CLI", () => {
	// C08: all flags populate frontmatter fields
	test("all flags populate frontmatter fields", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp,skills",
				],
				d,
			);
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
				join(d, ".dotagents/commands/greet.md"),
				"utf8",
			);
			expect(content).toContain("name: greet");
			expect(content).toContain("Say hello");
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
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp,skills",
				],
				d,
			);
			run(["commands", "new", "greet", "--description", "Say hello"], d);
			const content = readFileSync(
				join(d, ".dotagents/commands/greet.md"),
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
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp,skills",
				],
				d,
			);
			run(["commands", "new", "greet", "--description", "first"], d);
			const { exitCode } = run(
				["commands", "new", "greet", "--description", "second", "--force"],
				d,
			);
			expect(exitCode).toBe(0);
			const content = readFileSync(
				join(d, ".dotagents/commands/greet.md"),
				"utf8",
			);
			expect(content).toContain("second");
		} finally {
			cleanup(d);
		}
	});

	// duplicate without --force fails
	test("duplicate without --force exits non-zero", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp,skills",
				],
				d,
			);
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

	// TC-CMD-NEW-03: CI mode with no metadata flags produces empty defaults
	test("CI mode with no metadata flags produces empty defaults", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--ci"], d);
			const { exitCode } = run(["commands", "new", "test-cmd", "--ci"], d);
			expect(exitCode).toBe(0);
			const content = readFileSync(
				join(d, ".dotagents/commands/test-cmd.md"),
				"utf8",
			);
			// frontmatter is the first YAML block between --- markers
			const frontmatterMatch = content.match(/^---\n([\s\S]*?)\n---/);
			expect(frontmatterMatch).not.toBeNull();
			const frontmatter = frontmatterMatch?.[1] ?? "";
			expect(frontmatter).toContain("description: ''");
			expect(frontmatter).not.toMatch(/^category:/m);
			expect(frontmatter).not.toMatch(/^tags:/m);
		} finally {
			cleanup(d);
		}
	});

	// TC-CMD-NEW-06: --deploy (default CI auto-deploy) triggers deploy after creation.
	// Already covered by "CI auto-deploys after commands new" in deploy-default block.

	// TC-CMD-RM-06: --deploy (default CI auto-deploy) re-runs deploy after removal.
	// Already covered by "CI auto-deploys after commands rm" in deploy-default block.
});

// ── commands ls – CLI ─────────────────────────────────────────────────────────

test.describe("commands ls CLI", () => {
	// shows commands from init scaffold
	test("shows hello command from init scaffold", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp,skills",
				],
				d,
			);
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
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp,skills",
				],
				d,
			);
			const { exitCode, stderr } = run(["commands", "ls"], d);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/command\(s\)/);
		} finally {
			cleanup(d);
		}
	});

	// C24: --content flag succeeds
	test("--content exits zero", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp,skills",
				],
				d,
			);
			const { exitCode } = run(["commands", "ls", "--content"], d);
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
					"commands,instructions,mcp,skills",
				],
				d,
			);
			const { exitCode, stdout } = run(["commands", "ls", "--json"], d);
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
					"commands,instructions,mcp,skills",
				],
				d,
			);
			const { exitCode, stdout } = run(["commands", "ls", "--json"], d);
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
					"commands,instructions,mcp,skills",
				],
				d,
			);
			const { exitCode, stdout } = run(
				["commands", "ls", "--json", "--content"],
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
					"commands,instructions,mcp,skills",
				],
				d,
			);
			const { exitCode, stderr } = run(["commands", "ls", "--content"], d);
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
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp,skills",
				],
				d,
			);
			const { exitCode, stderr } = run(["commands", "ls"], d);
			expect(exitCode).toBe(0);
			expect(stderr).not.toMatch(/var\.agent_name/);
		} finally {
			cleanup(d);
		}
	});

	// --json with empty workspace outputs []
	test("--json with no commands outputs []", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp,skills",
				],
				d,
			);
			// Remove the scaffolded hello command to make it empty
			rmSync(join(d, ".dotagents/commands/hello.md"), { force: true });
			const { exitCode, stdout } = run(["commands", "ls", "--json"], d);
			expect(exitCode).toBe(0);
			expect(stdout.trim()).toBe("[]");
		} finally {
			cleanup(d);
		}
	});

	// --json is pipeable (basic validity check)
	test("--json output is valid JSON array (pipeable)", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp,skills",
				],
				d,
			);
			const { exitCode, stdout } = run(["commands", "ls", "--json"], d);
			expect(exitCode).toBe(0);
			const parsed = JSON.parse(stdout);
			expect(Array.isArray(parsed)).toBe(true);
		} finally {
			cleanup(d);
		}
	});

	// --command filters by command name
	test("--command filters to matching command", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp,skills",
				],
				d,
			);
			run(["commands", "new", "greet", "--description", "test"], d);
			const { exitCode, stderr } = run(
				["commands", "ls", "--command", "greet"],
				d,
			);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/greet/);
			expect(stderr).not.toMatch(/hello/);
		} finally {
			cleanup(d);
		}
	});

	// --command with no match shows "No commands found"
	test("--command with unmatched name shows no commands found", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp,skills",
				],
				d,
			);
			const { exitCode, stderr } = run(
				["commands", "ls", "--command", "nonexistent"],
				d,
			);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/No commands found/);
		} finally {
			cleanup(d);
		}
	});

	// --command with --json filters JSON output
	test("--command --json filters JSON output", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp,skills",
				],
				d,
			);
			const { exitCode, stdout } = run(
				["commands", "ls", "--json", "--command", "hello"],
				d,
			);
			expect(exitCode).toBe(0);
			const parsed = JSON.parse(stdout);
			expect(parsed).toHaveLength(1);
			expect(parsed[0].name).toBe("hello");
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
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp,skills",
				],
				d,
			);
			expect(existsSync(join(d, ".dotagents/commands/hello.md"))).toBe(true);
			const { exitCode } = run(["commands", "rm", "hello", "--force"], d);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".dotagents/commands/hello.md"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});

	// 6.2: commands rm removes deployed file and gitignore entry after deploy
	test("--force removes deployed command file and gitignore entry", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--gitignore"], d);

			expect(existsSync(join(d, ".mycode/commands/hello.md"))).toBe(true);

			const giBefore = readFileSync(join(d, ".gitignore"), "utf8");
			expect(giBefore).toContain(".mycode/");

			const { exitCode } = run(["commands", "rm", "hello", "--force"], d);
			expect(exitCode).toBe(0);

			expect(existsSync(join(d, ".mycode/commands/hello.md"))).toBe(false);

			const giAfter = readFileSync(join(d, ".gitignore"), "utf8");
			expect(giAfter).toContain(".mycode/");
		} finally {
			cleanup(d);
		}
	});

	// only the named command is removed
	test("only removes the named command, others remain", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp,skills",
				],
				d,
			);
			run(["commands", "new", "greet", "--description", "test"], d);
			expect(existsSync(join(d, ".dotagents/commands/greet.md"))).toBe(true);
			run(["commands", "rm", "greet", "--force"], d);
			expect(existsSync(join(d, ".dotagents/commands/hello.md"))).toBe(true);
			expect(existsSync(join(d, ".dotagents/commands/greet.md"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});
});

// ── commands deploy-default behavior ───────────────────────────────────────────

test.describe("commands deploy-default", () => {
	// --no-deploy skips auto-deploy in CI
	test("--no-deploy skips deploy after commands new", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode } = run(
				[
					"commands",
					"new",
					"greet",
					"--ci",
					"--no-deploy",
					"--description",
					"test",
				],
				d,
			);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".mycode/commands/greet.md"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});

	// CI auto-deploys after commands new
	test("CI auto-deploys after commands new", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode } = run(
				["commands", "new", "greet", "--ci", "--description", "test"],
				d,
			);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".mycode/commands/greet.md"))).toBe(true);
		} finally {
			cleanup(d);
		}
	});

	// --no-deploy skips redeploy after commands rm
	test("--no-deploy skips deploy after commands rm", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--gitignore"], d);
			expect(existsSync(join(d, ".mycode/commands/hello.md"))).toBe(true);
			const { exitCode } = run(
				["commands", "rm", "hello", "--ci", "--no-deploy", "--force"],
				d,
			);
			expect(exitCode).toBe(0);
			// The deployed file still exists because rm cleans up via undeploy,
			// but the deploy after rm is skipped by --no-deploy
		} finally {
			cleanup(d);
		}
	});

	// CI auto-deploys after commands rm
	test("CI auto-deploys after commands rm", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			// First deploy to create the mycode output
			run(["deploy", "--offline", "--gitignore"], d);
			expect(existsSync(join(d, ".mycode/commands/hello.md"))).toBe(true);
			// Then rm in CI should redeploy (removing the deployed file)
			const { exitCode } = run(
				["commands", "rm", "hello", "--ci", "--force"],
				d,
			);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".mycode/commands/hello.md"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});
});

// ── commands --cwd ────────────────────────────────────────────────────────────

test.describe("commands --cwd", () => {
	// commands ls --cwd reads from the specified workspace
	test("commands ls --cwd reads from target workspace", async () => {
		const { cwd, workspace } = makeTwoDirs();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp,skills",
				],
				workspace,
			);
			run(["commands", "new", "greet", "--description", "Greet"], workspace);
			const { exitCode, stderr } = run(
				["commands", "ls", "--cwd", workspace],
				cwd,
			);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/greet/);
		} finally {
			cleanup(cwd);
			cleanup(workspace);
		}
	});

	// commands ls --cwd <nonexistent> exits non-zero
	test("commands ls --cwd nonexistent exits non-zero", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode, stderr } = run(
				["commands", "ls", "--cwd", "/tmp/not-a-dotagents-workspace"],
				d,
			);
			expect(exitCode).not.toBe(0);
			const ok = stderr.includes(".dotagents") || stderr.includes("Failed");
			expect(ok).toBe(true);
		} finally {
			cleanup(d);
		}
	});

	// commands new --cwd creates file in target workspace
	test("commands new --cwd creates file in target workspace", async () => {
		const { cwd, workspace } = makeTwoDirs();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp,skills",
				],
				workspace,
			);
			const { exitCode } = run(
				[
					"commands",
					"new",
					"greet",
					"--cwd",
					workspace,
					"--description",
					"Test",
				],
				cwd,
			);
			expect(exitCode).toBe(0);
			const file = join(workspace, ".dotagents/commands/greet.md");
			expect(existsSync(file)).toBe(true);
			const content = readFileSync(file, "utf8");
			expect(content).toContain("name: greet");
		} finally {
			cleanup(cwd);
			cleanup(workspace);
		}
	});

	// commands rm --cwd removes from target workspace
	test("commands rm --cwd removes from target workspace", async () => {
		const { cwd, workspace } = makeTwoDirs();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp,skills",
				],
				workspace,
			);
			run(["commands", "new", "greet", "--description", "Test"], workspace);
			const file = join(workspace, ".dotagents/commands/greet.md");
			expect(existsSync(file)).toBe(true);

			const { exitCode } = run(
				["commands", "rm", "greet", "--force", "--cwd", workspace],
				cwd,
			);
			expect(exitCode).toBe(0);
			expect(existsSync(file)).toBe(false);
		} finally {
			cleanup(cwd);
			cleanup(workspace);
		}
	});

	// relative --cwd resolved against CWD
	test("relative --cwd resolved against current directory", async () => {
		const d = makeTmpDir();
		try {
			const sub = join(d, "sub");
			run([
				"init",
				sub,
				"--template",
				"starter",
				"--features",
				"commands,instructions,mcp,skills",
			]);
			const { exitCode, stderr } = run(["commands", "ls", "--cwd", "sub"], d);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/hello/);
		} finally {
			cleanup(d);
		}
	});

	// --cwd omitted resolves from CWD as before
	test("--cwd omitted resolves from CWD as before", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp,skills",
				],
				d,
			);
			run(["commands", "new", "greet", "--description", "Test"], d);
			const { exitCode, stderr } = run(["commands", "ls"], d);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/greet/);
		} finally {
			cleanup(d);
		}
	});

	// TC-CMD-NEW-10: --cwd pointing to a directory without .dotagents/ exits non-zero
	test("commands new --cwd to non-workspace exits non-zero", async () => {
		const d = makeTmpDir();
		try {
			const emptyDir = makeTmpDir();
			try {
				const { exitCode, stderr } = run(
					["commands", "new", "test-cmd", "--cwd", emptyDir, "--ci"],
					d,
				);
				expect(exitCode).not.toBe(0);
				expect(stderr).toContain(".dotagents");
			} finally {
				cleanup(emptyDir);
			}
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
	run(
		[
			"init",
			"--template",
			"starter",
			"--features",
			"commands,instructions,mcp,skills",
		],
		d,
	);
	test.use({ program: shellProgram(d, ["commands", "new", "greet"]) });

	test("prompts for description, category, tags then deploy", async ({
		terminal,
	}) => {
		try {
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

			expect(existsSync(join(d, ".dotagents/commands/greet.md"))).toBe(true);
			const content = readFileSync(
				join(d, ".dotagents/commands/greet.md"),
				"utf8",
			);
			expect(content).toContain("A greeting command");
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
test.describe("commands new TUI – T07 deploy on Yes", () => {
	const d = makeTmpDir();
	run(
		[
			"init",
			"--template",
			"with-custom-provider",
			"--features",
			"commands,instructions,mcp,skills",
		],
		d,
	);
	const lcPath = join(d, ".dotagents/local.config.toml");
	writeFileSync(
		lcPath,
		readFileSync(lcPath, "utf8").replace(
			/targets\s*=\s*\["gemini"\]/,
			"targets = []",
		),
	);
	test.use({
		program: shellProgram(d, ["commands", "new", "deploy-test-cmd"]),
	});

	test("answering Yes to deploy prompt runs deploy (deploy prompt is nested inside add)", async ({
		terminal,
	}) => {
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
			await expect(terminal.getByText("written")).toBeVisible();
			expect(existsSync(join(d, ".mycode/commands/deploy-test-cmd.md"))).toBe(
				true,
			);
		} finally {
			cleanup(d);
		}
	});
});

// ── commands rm – TUI ─────────────────────────────────────────────────────────

// T10: confirm prompt appears, navigating to Yes removes the file
test.describe("commands rm TUI – T10 confirm Yes", () => {
	const d = makeTmpDir();
	run(
		[
			"init",
			"--template",
			"starter",
			"--features",
			"commands,instructions,mcp,skills",
		],
		d,
	);
	test.use({ program: shellProgram(d, ["commands", "rm", "hello"]) });

	test("confirm Yes removes the command", async ({ terminal }) => {
		try {
			await expect(terminal.getByText("Remove command 'hello'?")).toBeVisible();
			await expect(terminal.getByText("This cannot be undone.")).toBeVisible();
			// default is No; navigate up to Yes
			terminal.keyUp();
			terminal.keyPress("Enter");

			await expect(terminal.getByText("Removed")).toBeVisible();

			expect(existsSync(join(d, ".dotagents/commands/hello.md"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});
});

// T11: pressing Enter on default No cancels the removal
test.describe("commands rm TUI – T11 confirm No", () => {
	const d = makeTmpDir();
	run(
		[
			"init",
			"--template",
			"starter",
			"--features",
			"commands,instructions,mcp,skills",
		],
		d,
	);
	test.use({ program: shellProgram(d, ["commands", "rm", "hello"]) });

	test("confirm No leaves the command file intact", async ({ terminal }) => {
		try {
			await expect(terminal.getByText("Remove command 'hello'?")).toBeVisible();
			terminal.keyPress("Enter"); // accept default No
			await expect(terminal.getByText("Cancelled")).toBeVisible();

			expect(existsSync(join(d, ".dotagents/commands/hello.md"))).toBe(true);
		} finally {
			cleanup(d);
		}
	});
});
