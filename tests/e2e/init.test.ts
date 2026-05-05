import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { expect, Key, test } from "@microsoft/tui-test";
import { cleanup, makeTmpDir, run, shellProgram } from "./helpers.js";

// ── CLI flows (non-interactive) ──────────────────────────────────────────────

test.describe("init CLI – file tree", () => {
	// C01: starter template creates expected files
	test("starter template creates core files", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode } = run(["init", "--template", "starter"], d);
			expect(exitCode).toBe(0);
			const root = join(d, ".dotagents-debug");
			expect(existsSync(join(root, "config.toml"))).toBe(true);
			expect(existsSync(join(root, ".env"))).toBe(true);
			expect(existsSync(join(root, ".env.example"))).toBe(true);
			expect(existsSync(join(root, ".gitignore"))).toBe(true);
			expect(existsSync(join(root, "INSTRUCTIONS.md"))).toBe(true);
			expect(existsSync(join(root, "mcp.jsonc"))).toBe(true);
			expect(existsSync(join(root, "commands/hello.md"))).toBe(true);
			expect(existsSync(join(root, "skills/hello-skill/SKILL.md"))).toBe(true);
			// starter does NOT create mycode templates
			expect(existsSync(join(root, "templates/mycode"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});

	// C02: with-custom-provider adds mycode templates
	test("with-custom-provider adds template files", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode } = run(
				["init", "--template", "with-custom-provider"],
				d,
			);
			expect(exitCode).toBe(0);
			const root = join(d, ".dotagents-debug");
			expect(existsSync(join(root, "templates/mycode/command.hbs"))).toBe(true);
			expect(existsSync(join(root, "templates/mycode/instructions.hbs"))).toBe(
				true,
			);
			expect(existsSync(join(root, "templates/mycode/mcp.hbs"))).toBe(true);
			expect(existsSync(join(root, "templates/mycode/skill.hbs"))).toBe(true);
		} finally {
			cleanup(d);
		}
	});

	// C03: --force overwrites existing dir
	test("--force overwrites existing workspace", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode } = run(["init", "--force", "--template", "starter"], d);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".dotagents-debug/config.toml"))).toBe(true);
		} finally {
			cleanup(d);
		}
	});

	// C04: --features without commands omits commands dir
	test("--features without commands omits commands directory", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode } = run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"instructions,mcp,skills",
				],
				d,
			);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".dotagents-debug/commands"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});

	// C05: --features without skills omits skills dir
	test("--features without skills omits skills directory", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode } = run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp",
				],
				d,
			);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".dotagents-debug/skills"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});

	// C06: --features without mcp omits mcp.jsonc
	test("--features without mcp omits mcp.jsonc", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode } = run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,skills",
				],
				d,
			);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".dotagents-debug/mcp.jsonc"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});

	// C07: --features without instructions omits INSTRUCTIONS.md
	test("--features without instructions omits INSTRUCTIONS.md", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode } = run(
				["init", "--template", "starter", "--features", "commands,mcp,skills"],
				d,
			);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".dotagents-debug/INSTRUCTIONS.md"))).toBe(
				false,
			);
		} finally {
			cleanup(d);
		}
	});
});

test.describe("init CLI – config content", () => {
	test("config.toml declares expected features", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const config = readFileSync(
				join(d, ".dotagents-debug/config.toml"),
				"utf8",
			);
			expect(config).toContain("features");
			expect(config).toContain('"commands"');
			expect(config).toContain('"instructions"');
			expect(config).toContain('"mcp"');
			expect(config).toContain('"skills"');
		} finally {
			cleanup(d);
		}
	});

	// C08: --features flag persists only selected features to config.toml
	test("--features flag persists selected features to config.toml", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode } = run(
				["init", "--template", "starter", "--features", "commands,mcp"],
				d,
			);
			expect(exitCode).toBe(0);
			const config = readFileSync(
				join(d, ".dotagents-debug/config.toml"),
				"utf8",
			);
			expect(config).toContain('"commands"');
			expect(config).toContain('"mcp"');
			expect(config).not.toContain('"instructions"');
			expect(config).not.toContain('"skills"');
		} finally {
			cleanup(d);
		}
	});

	// C09: --features none writes empty features array to config.toml
	test("--features none writes empty features array to config.toml", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode } = run(
				["init", "--template", "starter", "--features", "none"],
				d,
			);
			expect(exitCode).toBe(0);
			const config = readFileSync(
				join(d, ".dotagents-debug/config.toml"),
				"utf8",
			);
			expect(config).toContain("features = []");
		} finally {
			cleanup(d);
		}
	});

	test(".gitignore excludes local.config.toml and .env", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const gi = readFileSync(join(d, ".dotagents-debug/.gitignore"), "utf8");
			expect(gi).toContain("local.config.toml");
			expect(gi).toContain(".env");
		} finally {
			cleanup(d);
		}
	});

	test("hello.md has YAML frontmatter with name and description", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const cmd = readFileSync(
				join(d, ".dotagents-debug/commands/hello.md"),
				"utf8",
			);
			expect(cmd).toMatch(/^---/);
			expect(cmd).toContain("name:");
			expect(cmd).toContain("description:");
		} finally {
			cleanup(d);
		}
	});
});

// ── CLI flows – PATH argument ────────────────────────────────────────────────

test.describe("init CLI – PATH argument", () => {
	// C-PA01: absolute PATH creates .dotagents inside that directory
	test("absolute PATH scaffolds .dotagents inside the target dir", async () => {
		const cwd = makeTmpDir();
		const target = makeTmpDir();
		try {
			const { exitCode } = run(["init", target, "--template", "starter"], cwd);
			expect(exitCode).toBe(0);
			expect(existsSync(join(target, ".dotagents-debug/config.toml"))).toBe(
				true,
			);
			// CWD should NOT have been initialised
			expect(existsSync(join(cwd, ".dotagents-debug"))).toBe(false);
		} finally {
			cleanup(cwd);
			cleanup(target);
		}
	});

	// C-PA02: relative PATH is resolved against CWD
	test("relative PATH resolves against CWD", async () => {
		const cwd = makeTmpDir();
		try {
			const { exitCode } = run(
				["init", "./subproject", "--template", "starter"],
				cwd,
			);
			expect(exitCode).toBe(0);
			expect(
				existsSync(join(cwd, "subproject/.dotagents-debug/config.toml")),
			).toBe(true);
		} finally {
			cleanup(cwd);
		}
	});

	// C-PA03: non-existent PATH is created automatically
	test("non-existent PATH is created before scaffolding", async () => {
		const cwd = makeTmpDir();
		const target = join(cwd, "brand", "new", "nested");
		try {
			const { exitCode } = run(["init", target, "--template", "starter"], cwd);
			expect(exitCode).toBe(0);
			expect(existsSync(join(target, ".dotagents-debug/config.toml"))).toBe(
				true,
			);
		} finally {
			cleanup(cwd);
		}
	});
});

// ── TUI flows (interactive, uses tui-test terminal) ──────────────────────────
// Each TUI test lives in its own describe block so that test.use() (which sets
// the terminal program) is evaluated at describe level — not inside the test body.
// The workspace directory is created synchronously at describe evaluation time
// so it is available to both test.use() and the test body via closure.

// T01: full wizard happy path
test.describe("init TUI – T01 wizard happy path", () => {
	const d = makeTmpDir();
	test.use({ program: shellProgram(d, ["init"]) });

	test("wizard shows all prompts and completes successfully", async ({
		terminal,
	}) => {
		try {
			await expect(
				terminal.getByText("Which features do you want to enable?"),
			).toBeVisible();

			// confirm multiselect defaults (all four features selected)
			terminal.keyPress("Enter");
			await expect(
				terminal.getByText("Which starting template?"),
			).toBeVisible();

			// select Starter (first option, already highlighted)
			terminal.keyPress("Enter");
			await expect(
				terminal.getByText("Which providers would you like to target?"),
			).toBeVisible();

			// skip providers
			terminal.keyPress("Enter");
			await expect(terminal.getByText("Done! Run")).toBeVisible();

			// workspace was created
			expect(existsSync(join(d, ".dotagents-debug/config.toml"))).toBe(true);
		} finally {
			cleanup(d);
		}
	});
});

// T02: deselect features in multiselect
test.describe("init TUI – T02 deselect features", () => {
	const d = makeTmpDir();
	test.use({ program: shellProgram(d, ["init"]) });

	test("deselecting mcp and skills omits those files", async ({ terminal }) => {
		try {
			await expect(
				terminal.getByText("Which features do you want to enable?"),
			).toBeVisible();

			// navigate to MCP option (3rd item: down twice) and space to deselect
			terminal.keyDown(2);
			terminal.keyPress("Space");
			// navigate to Skills (4th item: one more down) and space to deselect
			terminal.keyDown();
			terminal.keyPress("Space");
			// confirm
			terminal.keyPress("Enter");
			await expect(
				terminal.getByText("Which starting template?"),
			).toBeVisible();
			terminal.keyPress("Enter");
			// skip providers
			terminal.keyPress("Enter");
			await expect(terminal.getByText("Done! Run")).toBeVisible();

			expect(existsSync(join(d, ".dotagents-debug/mcp.jsonc"))).toBe(false);
			expect(existsSync(join(d, ".dotagents-debug/skills"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});
});

// T03: select WithCustomProvider template
test.describe("init TUI – T03 WithCustomProvider template", () => {
	const d = makeTmpDir();
	test.use({ program: shellProgram(d, ["init"]) });

	test("selecting WithCustomProvider template creates mycode templates", async ({
		terminal,
	}) => {
		try {
			await expect(
				terminal.getByText("Which features do you want to enable?"),
			).toBeVisible();
			terminal.keyPress("Enter");
			await expect(
				terminal.getByText("Which starting template?"),
			).toBeVisible();

			// move down to select "With Custom Provider"
			terminal.keyDown();
			terminal.keyPress("Enter");
			// skip providers
			terminal.keyPress("Enter");
			await expect(terminal.getByText("Done! Run")).toBeVisible();

			expect(
				existsSync(join(d, ".dotagents-debug/templates/mycode/command.hbs")),
			).toBe(true);
		} finally {
			cleanup(d);
		}
	});
});

// T-PA: PATH arg — wizard still runs when a PATH argument is provided
test.describe("init TUI – T-PA wizard runs with PATH argument", () => {
	const cwd = makeTmpDir();
	const target = makeTmpDir();
	test.use({ program: shellProgram(cwd, ["init", target]) });

	test("wizard appears and completes when PATH argument is provided", async ({
		terminal,
	}) => {
		try {
			// Wizard must start even though PATH was supplied
			await expect(
				terminal.getByText("Which features do you want to enable?"),
			).toBeVisible();

			terminal.keyPress("Enter");
			await expect(
				terminal.getByText("Which starting template?"),
			).toBeVisible();

			terminal.keyPress("Enter");
			// skip providers
			terminal.keyPress("Enter");
			await expect(terminal.getByText("Done! Run")).toBeVisible();

			// Files must land inside the PATH directory, not CWD
			expect(existsSync(join(target, ".dotagents-debug/config.toml"))).toBe(
				true,
			);
			expect(existsSync(join(cwd, ".dotagents-debug"))).toBe(false);
		} finally {
			cleanup(cwd);
			cleanup(target);
		}
	});
});

// T04: wizard cancellation — no directory created on disk
// Pressing Escape during the multiselect prompt cancels it and returns
// an error (cliclack raw mode prevents Ctrl-C from generating SIGINT).
// The important invariant: no directory is created regardless of exit code.
test.describe("init TUI – T04 cancel wizard leaves no directory", () => {
	const d = makeTmpDir();
	test.use({ program: shellProgram(d, ["init"]) });

	test("cancel at first prompt leaves no directory on disk", async ({
		terminal,
	}) => {
		await expect(
			terminal.getByText("Which features do you want to enable?"),
		).toBeVisible();

		// Press Escape to cancel the multiselect prompt.
		terminal.keyPress(Key.Escape);

		// The process should exit (with error output) once the prompt is cancelled.
		// Wait for the error message chain to appear.
		await expect(
			terminal.getByText("Failed to get feature selection"),
		).toBeVisible();

		// No directory of any kind should exist — `create_dir_all` runs after
		// the TUI wizard block, so cancellation prevents any filesystem writes.
		expect(existsSync(join(d, ".dotagents-debug"))).toBe(false);
	});

	test.afterEach(() => {
		cleanup(d);
	});
});

// T05: overwrite cancel — NOTE: debug binary defaults force=true so the
//      overwrite prompt is suppressed; mark as skip
test.describe("init TUI – T05 overwrite cancel (skipped)", () => {
	// stub program — setup runs inside the (skipped) body so no filesystem
	// mutations happen at describe evaluation time
	test.use({ program: { file: "bash", args: ["-c", "true"] } });

	test.skip("cancel overwrite exits without changes (release binary only)", async ({
		terminal,
	}) => {
		// setup deferred into body — never runs because test is skipped
		const d = makeTmpDir();
		run(["init", "--template", "starter"], d);
		try {
			await expect(terminal.getByText("already exists")).toBeVisible();
			terminal.keyDown(); // move to No
			terminal.keyPress("Enter");
			await expect(terminal.getByText("Init cancelled.")).toBeVisible();
		} finally {
			cleanup(d);
		}
	});
});
