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

// ── TUI flows ────────────────────────────────────────────────────────────────
// Each TUI test has its own describe block so test.use() is at describe level.

// T14: offline prompt (No) + no-gitignore
test.describe("deploy TUI – T14 offline prompt", () => {
	const d = makeTmpDir();
	initWithLocalProvider(d);
	test.use({ program: shellProgram(d, ["deploy", "--no-gitignore"]) });

	test("pressing Enter accepts offline=No default", async ({ terminal }) => {
		try {
			// offline prompt appears — Enter accepts the default (No = online mode)
			await expect(terminal.getByText("Run in offline mode?")).toBeVisible();
			await expect(
				terminal.getByText("No, fetch latest templates"),
			).toBeVisible();
			terminal.keyPress("Enter"); // accept No (online)
			// Deploy runs silently (no output text to await); just verifying the
			// prompt appeared and the keypress was accepted without error.
		} finally {
			cleanup(d);
		}
	});
});

// T15: --offline flag skips the interactive offline prompt (CLI test, no TUI needed)
test.describe("deploy CLI – T15 offline flag suppresses prompt", () => {
	test("--offline flag suppresses offline prompt", async () => {
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
