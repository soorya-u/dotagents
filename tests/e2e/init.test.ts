import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "@microsoft/tui-test";
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
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"instructions,mcp,skills",
				],
				d,
			);
			expect(existsSync(join(d, ".dotagents-debug/commands"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});

	// C05: --features without skills omits skills dir
	test("--features without skills omits skills directory", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,mcp",
				],
				d,
			);
			expect(existsSync(join(d, ".dotagents-debug/skills"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});

	// C06: --features without mcp omits mcp.jsonc
	test("--features without mcp omits mcp.jsonc", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"commands,instructions,skills",
				],
				d,
			);
			expect(existsSync(join(d, ".dotagents-debug/mcp.jsonc"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});

	// C07: --features without instructions omits INSTRUCTIONS.md
	test("--features without instructions omits INSTRUCTIONS.md", async () => {
		const d = makeTmpDir();
		try {
			run(
				["init", "--template", "starter", "--features", "commands,mcp,skills"],
				d,
			);
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
			// intro and first prompt
			await expect(terminal.getByText("dotagents · init")).toBeVisible();
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
