import { existsSync, readFileSync, unlinkSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { expect, test } from "@microsoft/tui-test";
import {
	cleanup,
	initWithLocalProvider,
	makeTmpDir,
	run,
	shellProgram,
} from "./helpers.js";

// ── CLI flows ────────────────────────────────────────────────────────────────

test.describe("deploy CLI – output structure", () => {
	// C14: deploy creates expected output files
	test("creates mycode output files after init", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode } = run(["deploy", "--offline", "--no-gitignore"], d);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".mycode/commands/hello.md"))).toBe(true);
			expect(existsSync(join(d, ".mycode/instructions.md"))).toBe(true);
			expect(existsSync(join(d, ".mycode/mcp.json"))).toBe(true);
			expect(existsSync(join(d, ".mycode/skills/hello-skill/SKILL.md"))).toBe(
				true,
			);
		} finally {
			cleanup(d);
		}
	});

	// C15: --force re-deploys even if cache is fresh
	test("--force re-deploys without error", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);
			const { exitCode } = run(
				["deploy", "--force", "--offline", "--no-gitignore"],
				d,
			);
			expect(exitCode).toBe(0);
		} finally {
			cleanup(d);
		}
	});

	// C17: --no-cache bypasses cache entirely
	test("--no-cache deploys without reading cache.toml", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode } = run(
				["deploy", "--no-cache", "--offline", "--no-gitignore"],
				d,
			);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".mycode/commands/hello.md"))).toBe(true);
		} finally {
			cleanup(d);
		}
	});
});

test.describe("deploy CLI – rendered content", () => {
	// command output has no YAML frontmatter
	test("command output file has no frontmatter", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);
			const content = readFileSync(
				join(d, ".mycode/commands/hello.md"),
				"utf8",
			);
			expect(content).not.toMatch(/^---/);
		} finally {
			cleanup(d);
		}
	});

	// command output contains source body
	test("command output contains source body text", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);
			const content = readFileSync(
				join(d, ".mycode/commands/hello.md"),
				"utf8",
			);
			expect(content).toContain("Greet");
		} finally {
			cleanup(d);
		}
	});

	// mcp output is valid JSON with mcpServers key
	test("mcp.json is valid JSON with mcpServers key", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);
			const raw = readFileSync(join(d, ".mycode/mcp.json"), "utf8");
			const parsed = JSON.parse(raw);
			expect(parsed).toHaveProperty("mcpServers");
		} finally {
			cleanup(d);
		}
	});

	test("expanded MCP config renders mapped provider fields", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const appDir = join(d, ".dotagents");
			const configPath = join(appDir, "local.config.toml");
			const codexTemplate = resolve(
				process.cwd(),
				"../../public/v1/templates/codex/mcp.hbs",
			);
			const geminiTemplate = resolve(
				process.cwd(),
				"../../public/v1/templates/gemini/mcp.hbs",
			);
			writeFileSync(
				configPath,
				`${readFileSync(configPath, "utf8")}

[providers.codex.mcp]
template = "${codexTemplate}"
target = "{{ dir.workspace }}/.codex/config.toml"

[providers.gemini.mcp]
template = "${geminiTemplate}"
target = "{{ dir.workspace }}/.gemini/settings.json"
`,
			);
			writeFileSync(
				join(appDir, "mcp.jsonc"),
				JSON.stringify(
					{
						$schema:
							"https://dotagents.soorya-u.dev/v1/schemas/mcp.schema.json",
						servers: {
							"stdio-server": {
								type: "stdio",
								command: "node",
								args: ["server.js"],
								enabledTools: ["read"],
								disabledTools: ["delete"],
								required: true,
								startupTimeoutSec: 11,
								toolTimeoutSec: 22,
								bearerTokenEnvVar: "TOKEN",
								envVars: ["TOKEN"],
							},
							"http-server": {
								type: "http",
								url: "https://example.com/mcp",
								headers: { Authorization: "Bearer token" },
							},
							"sse-server": {
								type: "sse",
								url: "https://example.com/sse",
								headers: { "X-Test": "1" },
							},
						},
					},
					null,
					2,
				),
			);

			const { exitCode } = run(["deploy", "--offline", "--no-gitignore"], d);
			expect(exitCode).toBe(0);

			const codex = readFileSync(join(d, ".codex/config.toml"), "utf8");
			expect(codex).toContain('enabled_tools = ["read"]');
			expect(codex).toContain('disabled_tools = ["delete"]');
			expect(codex).toContain("startup_timeout_sec = 11");
			expect(codex).toContain("tool_timeout_sec = 22");
			expect(codex).toContain('bearer_token_env_var = "TOKEN"');
			expect(codex).toContain('env_vars = ["TOKEN"]');

			const gemini = JSON.parse(
				readFileSync(join(d, ".gemini/settings.json"), "utf8"),
			);
			expect(gemini.mcpServers["http-server"].httpUrl).toBe(
				"https://example.com/mcp",
			);
			expect(gemini.mcpServers["sse-server"].url).toBe(
				"https://example.com/sse",
			);
		} finally {
			cleanup(d);
		}
	});

	// mcp stdio server type rendered as "local"
	test("stdio server type is rendered as local", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);
			const raw = readFileSync(join(d, ".mycode/mcp.json"), "utf8");
			expect(raw).toContain('"type": "local"');
		} finally {
			cleanup(d);
		}
	});

	// variable interpolation: var.agent_name
	test("instructions.md interpolates var.agent_name", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);
			const content = readFileSync(join(d, ".mycode/instructions.md"), "utf8");
			expect(content).toContain("Mycode");
			expect(content).not.toContain("{{");
		} finally {
			cleanup(d);
		}
	});

	// variable interpolation: env.app_name
	test("instructions.md interpolates env.app_name", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);
			const content = readFileSync(join(d, ".mycode/instructions.md"), "utf8");
			expect(content).toContain("dotagents");
		} finally {
			cleanup(d);
		}
	});

	// idempotency: deploy twice produces identical output
	test("deploy twice produces identical output", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--no-cache", "--offline", "--no-gitignore"], d);
			const first = readFileSync(join(d, ".mycode/commands/hello.md"), "utf8");
			run(["deploy", "--no-cache", "--offline", "--no-gitignore"], d);
			const second = readFileSync(join(d, ".mycode/commands/hello.md"), "utf8");
			expect(first).toBe(second);
		} finally {
			cleanup(d);
		}
	});
});

// deploy CI – summary output visibility
test.describe("deploy CLI – CI mode summary", () => {
	// CI mode with providers: stdout contains written count
	test("CI mode prints written count on stdout when providers are configured", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { stdout, exitCode } = run(
				["deploy", "--ci", "--offline", "--no-gitignore"],
				d,
			);
			expect(exitCode).toBe(0);
			expect(stdout).toMatch(/\d+ written/);
		} finally {
			cleanup(d);
		}
	});

	// CI mode with no providers: stderr contains warning
	test("no providers configured emits warning on stderr", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--force"], d);
			const { stderr, exitCode } = run(
				["deploy", "--ci", "--offline", "--no-gitignore"],
				d,
			);
			expect(exitCode).toBe(0);
			expect(stderr).toContain("No providers configured");
		} finally {
			cleanup(d);
		}
	});
});

// ── TUI flows ────────────────────────────────────────────────────────────────
// Each TUI test has its own describe block so test.use() is at describe level.

// T15: --offline flag uses cached templates (CLI test, no TUI needed)
test.describe("deploy CLI – T15 offline flag", () => {
	test("--offline flag deploys from cache", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode } = run(["deploy", "--offline", "--no-gitignore"], d);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".mycode/commands/hello.md"))).toBe(true);
		} finally {
			cleanup(d);
		}
	});
});

// ── --env flag ────────────────────────────────────────────────────────────────

test.describe("deploy CLI – --env flag", () => {
	// --env replaces the default .dotagents/.env entirely
	test("single --env file replaces default .env", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			// Write a custom env with a different APP_NAME than the default "dotagents"
			writeFileSync(join(d, "custom.env"), "APP_NAME=myapp\n");
			const { exitCode } = run(
				[
					"deploy",
					"--no-cache",
					"--offline",
					"--no-gitignore",
					"--env",
					"./custom.env",
				],
				d,
			);
			expect(exitCode).toBe(0);
			const content = readFileSync(join(d, ".mycode/instructions.md"), "utf8");
			// custom file value used
			expect(content).toContain("myapp");
			// default .env value NOT used
			expect(content).not.toContain("dotagents");
		} finally {
			cleanup(d);
		}
	});

	// later --env file wins on duplicate keys
	test("multiple --env files merge left-to-right with last file winning", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			writeFileSync(join(d, "base.env"), "APP_NAME=from-base\n");
			writeFileSync(join(d, "override.env"), "APP_NAME=from-override\n");
			const { exitCode } = run(
				[
					"deploy",
					"--no-cache",
					"--offline",
					"--no-gitignore",
					"--env",
					"./base.env",
					"--env",
					"./override.env",
				],
				d,
			);
			expect(exitCode).toBe(0);
			const content = readFileSync(join(d, ".mycode/instructions.md"), "utf8");
			expect(content).toContain("from-override");
			expect(content).not.toContain("from-base");
		} finally {
			cleanup(d);
		}
	});

	// missing --env file exits non-zero with the file path in the error message
	test("missing --env file exits non-zero with file path in error", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode, stderr } = run(
				["deploy", "--offline", "--no-gitignore", "--env", "./nonexistent.env"],
				d,
			);
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain("nonexistent.env");
		} finally {
			cleanup(d);
		}
	});

	// no --env flag: existing behaviour unchanged (default .env loaded silently)
	test("no --env flag loads default .env as before", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode } = run(
				["deploy", "--no-cache", "--offline", "--no-gitignore"],
				d,
			);
			expect(exitCode).toBe(0);
			const content = readFileSync(join(d, ".mycode/instructions.md"), "utf8");
			// default .env has APP_NAME=dotagents
			expect(content).toContain("dotagents");
		} finally {
			cleanup(d);
		}
	});
});

// ── CLI flows – PATH argument ────────────────────────────────────────────────

test.describe("deploy CLI – PATH argument", () => {
	// C-PA04: explicit absolute PATH deploys to correct workspace
	test("absolute PATH deploys to the specified workspace", async () => {
		const cwd = makeTmpDir();
		const target = makeTmpDir();
		try {
			initWithLocalProvider(target);
			const { exitCode } = run(
				["deploy", target, "--offline", "--no-gitignore"],
				cwd,
			);
			expect(exitCode).toBe(0);
			// Output files should be inside the target workspace, not CWD
			expect(existsSync(join(target, ".mycode/commands/hello.md"))).toBe(true);
			expect(existsSync(join(cwd, ".mycode"))).toBe(false);
		} finally {
			cleanup(cwd);
			cleanup(target);
		}
	});

	// C-PA05: PATH missing .dotagents exits non-zero with error message
	test("PATH without .dotagents exits non-zero with error", async () => {
		const cwd = makeTmpDir();
		const target = makeTmpDir(); // exists but has no .dotagents
		try {
			const { exitCode, stderr } = run(
				["deploy", target, "--offline", "--no-gitignore"],
				cwd,
			);
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain(".dotagents");
		} finally {
			cleanup(cwd);
			cleanup(target);
		}
	});
});

// ── deploy --dry-run ──────────────────────────────────────────────────────────

test.describe("deploy --dry-run – no side effects", () => {
	// D-DR01: --dry-run writes no files
	test("no output files are written to disk", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode } = run(["deploy", "--dry-run", "--offline"], d);
			expect(exitCode).toBe(0);
			// Output files must not exist after a dry run
			expect(existsSync(join(d, ".mycode/commands/hello.md"))).toBe(false);
			expect(existsSync(join(d, ".mycode/instructions.md"))).toBe(false);
			expect(existsSync(join(d, ".mycode/mcp.json"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});

	// D-DR02: --dry-run does not write cache.toml
	test("cache.toml is not written", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--dry-run", "--offline"], d);
			const cachePath = join(d, ".dotagents/cache.toml");
			// cache.toml must not exist (or must be empty if pre-existing)
			if (existsSync(cachePath)) {
				const content = readFileSync(cachePath, "utf8");
				expect(content).not.toContain("[providers.");
			}
		} finally {
			cleanup(d);
		}
	});

	// D-DR03: --dry-run does not modify .gitignore
	test(".gitignore is not modified", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const giBefore = existsSync(join(d, ".gitignore"))
				? readFileSync(join(d, ".gitignore"), "utf8")
				: "";
			run(["deploy", "--dry-run", "--offline"], d);
			const giAfter = existsSync(join(d, ".gitignore"))
				? readFileSync(join(d, ".gitignore"), "utf8")
				: "";
			expect(giAfter).toBe(giBefore);
		} finally {
			cleanup(d);
		}
	});
});

test.describe("deploy --dry-run – output format", () => {
	// D-DR04: stdout shows [+] for new files
	test("stdout contains [+] for each new target file", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { stdout, exitCode } = run(["deploy", "--dry-run", "--offline"], d);
			expect(exitCode).toBe(0);
			expect(stdout).toContain("[+]");
		} finally {
			cleanup(d);
		}
	});

	// D-DR05: stdout shows [~] when source changes and target already exists on disk
	test("stdout contains [~] when source changed and target exists on disk", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			// Deploy for real first so the target file exists on disk and cache is populated
			run(["deploy", "--offline", "--no-gitignore"], d);
			// Modify the SOURCE command file so rendered output will differ from cached hash
			writeFileSync(
				join(d, ".dotagents/commands/hello.md"),
				"---\nname: hello\ndescription: modified\n---\nModified body content",
			);
			// Dry-run: rendered_hash != cached_hash → bypasses cache-skip → DryRun path → [~]
			const { stdout, exitCode } = run(["deploy", "--dry-run", "--offline"], d);
			expect(exitCode).toBe(0);
			expect(stdout).toContain("[~]");
		} finally {
			cleanup(d);
		}
	});

	// D-DR06: unchanged files are hidden from dry-run output
	test("unchanged files do not appear in dry-run output", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			// Deploy for real so all files are up-to-date
			run(["deploy", "--offline", "--no-gitignore"], d);
			// Dry-run again — nothing should appear as [+] or [~]
			const { stdout, exitCode } = run(["deploy", "--dry-run", "--offline"], d);
			expect(exitCode).toBe(0);
			expect(stdout).toContain("0 files would be affected");
		} finally {
			cleanup(d);
		}
	});

	// D-DR07: stdout contains summary line
	test("stdout ends with 'files would be affected'", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { stdout } = run(["deploy", "--dry-run", "--offline"], d);
			expect(stdout).toContain("files would be affected");
		} finally {
			cleanup(d);
		}
	});

	// D-DR08: header line appears in output
	test("stdout contains dry run header", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { stdout } = run(["deploy", "--dry-run", "--offline"], d);
			expect(stdout).toContain("Dry run");
		} finally {
			cleanup(d);
		}
	});
});

test.describe("deploy --dry-run – error handling", () => {
	// D-DR09: missing template file causes exit code 1 with error on stderr
	test("missing template file causes exit code 1 with error on stderr", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			// Remove a provider template file so the renderer returns an error
			unlinkSync(join(d, ".dotagents/templates/mycode/instructions.hbs"));
			const { exitCode, stderr } = run(["deploy", "--dry-run", "--offline"], d);
			expect(exitCode).toBe(1);
			expect(stderr.length).toBeGreaterThan(0);
		} finally {
			cleanup(d);
		}
	});
});

test.describe("deploy --dry-run – flag interactions", () => {
	// D-DR10: --dry-run combined with --no-cache still writes nothing
	test("--dry-run --no-cache writes no files", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode } = run(
				["deploy", "--dry-run", "--offline", "--no-cache"],
				d,
			);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".mycode/commands/hello.md"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});
});

// ── Deploy --gitignore flag (CLI) ────────────────────────────────────────────

test.describe("deploy CLI – --gitignore flag", () => {
	// C-GI01: --gitignore writes fence to .gitignore
	test("--gitignore writes managed fence to .gitignore", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode } = run(["deploy", "--offline", "--gitignore"], d);
			expect(exitCode).toBe(0);
			const gi = readFileSync(join(d, ".gitignore"), "utf8");
			expect(gi).toContain("region dotagents");
			expect(gi).toContain(".mycode/");
		} finally {
			cleanup(d);
		}
	});

	// C-GI02: --gitignore is idempotent — second run does not duplicate entries
	test("--gitignore is idempotent on second run", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--gitignore"], d);
			const firstGi = readFileSync(join(d, ".gitignore"), "utf8");
			const countBefore = (firstGi.match(/region dotagents/g) || []).length;

			run(["deploy", "--offline", "--gitignore"], d);
			const secondGi = readFileSync(join(d, ".gitignore"), "utf8");
			const countAfter = (secondGi.match(/region dotagents/g) || []).length;
			expect(countAfter).toBe(countBefore);
		} finally {
			cleanup(d);
		}
	});
});

// ── Deploy gitignore prompt (TUI) ───────────────────────────────────────────

// T-GP01: gitignore prompt shows with default No, pressing Enter skips
test.describe("deploy TUI – T-GP01 gitignore prompt No", () => {
	const d = makeTmpDir();
	initWithLocalProvider(d);
	test.use({ program: shellProgram(d, ["deploy", "--offline"]) });

	test("pressing Enter on default No skips gitignore update", async ({
		terminal,
	}) => {
		try {
			await expect(
				terminal.getByText("deployed path(s) to .gitignore?"),
			).toBeVisible();
			terminal.keyPress("Enter"); // accept default No
			await expect(terminal.getByText("written")).toBeVisible();
			// .gitignore should NOT contain the dotagents fence
			const giPath = join(d, ".gitignore");
			if (existsSync(giPath)) {
				const content = readFileSync(giPath, "utf8");
				expect(content).not.toContain("region dotagents");
			}
		} finally {
			cleanup(d);
		}
	});
});

// T-GP02: selecting Yes on gitignore prompt adds fence
test.describe("deploy TUI – T-GP02 gitignore prompt Yes", () => {
	const d = makeTmpDir();
	initWithLocalProvider(d);
	test.use({ program: shellProgram(d, ["deploy", "--offline"]) });

	test("selecting Yes adds paths to .gitignore fence", async ({ terminal }) => {
		try {
			await expect(
				terminal.getByText("deployed path(s) to .gitignore?"),
			).toBeVisible();
			terminal.keyDown(); // navigate to Yes
			terminal.keyPress("Enter");
			await expect(terminal.getByText("written")).toBeVisible();
			const gi = readFileSync(join(d, ".gitignore"), "utf8");
			expect(gi).toContain("region dotagents");
		} finally {
			cleanup(d);
		}
	});
});

// ── Deploy user-edit protection (CLI) ────────────────────────────────────────

test.describe("deploy CLI – user-edit protection", () => {
	// TC-DEPLOY-07: redeploy preserves user-edited deployed file
	test("redeploy preserves user-edited deployed file", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);
			const originalContent = readFileSync(
				join(d, ".mycode/instructions.md"),
				"utf8",
			);

			// Simulate user editing the deployed file
			writeFileSync(
				join(d, ".mycode/instructions.md"),
				"User edited this file.",
			);

			// Redeploy without --force — file should be preserved
			const { exitCode } = run(["deploy", "--offline", "--no-gitignore"], d);
			expect(exitCode).toBe(0);
			const afterContent = readFileSync(
				join(d, ".mycode/instructions.md"),
				"utf8",
			);
			expect(afterContent).toBe("User edited this file.");
			expect(afterContent).not.toBe(originalContent);
		} finally {
			cleanup(d);
		}
	});

	// TC-DEPLOY-08: --force overwrites user-edited deployed file
	test("--force overwrites user-edited deployed file", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);
			const originalContent = readFileSync(
				join(d, ".mycode/instructions.md"),
				"utf8",
			);

			writeFileSync(
				join(d, ".mycode/instructions.md"),
				"User edited this file.",
			);

			// Redeploy with --force — file should be overwritten
			const { exitCode } = run(
				["deploy", "--force", "--offline", "--no-gitignore"],
				d,
			);
			expect(exitCode).toBe(0);
			const afterContent = readFileSync(
				join(d, ".mycode/instructions.md"),
				"utf8",
			);
			expect(afterContent).toBe(originalContent);
			expect(afterContent).not.toBe("User edited this file.");
		} finally {
			cleanup(d);
		}
	});
});

// ── Deploy flag and error path coverage ──────────────────────────────────────

test.describe("deploy CLI – --no-gitignore flag", () => {
	// TC-DEPLOY-10: --no-gitignore skips gitignore update
	test("--no-gitignore does not create .gitignore", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode } = run(["deploy", "--offline", "--no-gitignore"], d);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".mycode/commands/hello.md"))).toBe(true);
			const giPath = join(d, ".gitignore");
			if (existsSync(giPath)) {
				const content = readFileSync(giPath, "utf8");
				expect(content).not.toContain("region dotagents");
			}
		} finally {
			cleanup(d);
		}
	});
});

test.describe("deploy CLI – missing .env", () => {
	// TC-DEPLOY-20: missing .env is silently ignored
	test("deploy succeeds when .dotagents/.env does not exist", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const envPath = join(d, ".dotagents/.env");
			if (existsSync(envPath)) {
				unlinkSync(envPath);
			}
			const { exitCode } = run(["deploy", "--offline", "--no-gitignore"], d);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".mycode/commands/hello.md"))).toBe(true);
			expect(existsSync(join(d, ".mycode/instructions.md"))).toBe(true);
		} finally {
			cleanup(d);
		}
	});
});

test.describe("deploy CLI – malformed template error", () => {
	// TC-DEPLOY-ERR-01: malformed Handlebars template causes exit 1
	test("malformed template causes exit 1 with render error", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			writeFileSync(
				join(d, ".dotagents/templates/mycode/instructions.hbs"),
				"{{ unclosed",
			);
			const { exitCode, stderr } = run(
				["deploy", "--offline", "--no-gitignore"],
				d,
			);
			expect(exitCode).toBe(1);
			expect(stderr.length).toBeGreaterThan(0);
		} finally {
			cleanup(d);
		}
	});
});

test.describe("deploy CLI – untrusted URL rejection", () => {
	// TC-DEPLOY-ERR-02: non-HTTPS URL causes exit 1
	test("non-HTTPS template URL causes exit 1 with security error", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const configPath = join(d, ".dotagents/local.config.toml");
			const config = readFileSync(configPath, "utf8");
			const patched = config.replace(
				'template = "{{ dir.application }}/templates/mycode/instructions.hbs"',
				'template = "http://example.com/template.hbs"',
			);
			writeFileSync(configPath, patched);

			const { exitCode, stderr } = run(
				["deploy", "--offline", "--no-gitignore"],
				d,
			);
			expect(exitCode).toBe(1);
			expect(stderr).toContain("non-HTTPS");
		} finally {
			cleanup(d);
		}
	});
});

// ── Deploy TUI prompts ───────────────────────────────────────────────────────

// TC-DEPLOY-01: full TUI deploy journey
test.describe("deploy TUI – TC-DEPLOY-01 full deploy journey", () => {
	const d = makeTmpDir();
	initWithLocalProvider(d);
	test.use({ program: shellProgram(d, ["deploy"]) });

	test("full interactive deploy: gitignore No → summary → Done", async ({
		terminal,
	}) => {
		try {
			await expect(
				terminal.getByText("deployed path(s) to .gitignore?"),
			).toBeVisible();
			terminal.keyPress("Enter");

			await expect(terminal.getByText("written")).toBeVisible();
			expect(existsSync(join(d, ".mycode/commands/hello.md"))).toBe(true);
			expect(existsSync(join(d, ".mycode/instructions.md"))).toBe(true);
		} finally {
			cleanup(d);
		}
	});
});
