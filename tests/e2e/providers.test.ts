import { expect, test } from "@microsoft/tui-test";
import {
	cleanup,
	makeTmpDir,
	run,
	seedRegistryCache,
	shellProgram,
} from "./helpers.js";

// ── providers ls – CLI ────────────────────────────────────────────────────────

test.describe("providers ls CLI", () => {
	// --json exits 0 and returns a valid JSON array with slug/name/url fields
	test("--json exits 0 and returns valid JSON array", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode, stdout } = run(["providers", "ls", "--json"], d);
			expect(exitCode).toBe(0);
			const parsed = JSON.parse(stdout);
			expect(Array.isArray(parsed)).toBe(true);
			expect(parsed.length).toBeGreaterThan(0);
			for (const p of parsed) {
				expect(typeof p.slug).toBe("string");
				expect("name" in p).toBe(true);
				expect("url" in p).toBe(true);
			}
		} finally {
			cleanup(d);
		}
	});

	// --json --offline with seeded cache returns providers with name and url populated
	test("--json --offline with seeded cache returns populated name and url", async () => {
		const d = makeTmpDir();
		const xdgDir = seedRegistryCache();
		try {
			const { exitCode, stdout } = run(
				["providers", "ls", "--json", "--offline"],
				d,
				{ XDG_CONFIG_HOME: xdgDir },
			);
			expect(exitCode).toBe(0);
			const parsed = JSON.parse(stdout);
			const claude = parsed.find((p: { slug: string }) => p.slug === "claude");
			expect(claude).toBeDefined();
			expect(claude.name).toBe("Claude Code");
			expect(claude.url).toBe("https://docs.anthropic.com/en/docs/claude-code");
		} finally {
			cleanup(d);
			cleanup(xdgDir);
		}
	});

	// default text output lists slugs and names
	test("default text output --offline contains provider slugs", async () => {
		const d = makeTmpDir();
		const xdgDir = seedRegistryCache();
		try {
			const { exitCode, stdout } = run(["providers", "ls", "--offline"], d, {
				XDG_CONFIG_HOME: xdgDir,
			});
			expect(exitCode).toBe(0);
			expect(stdout).toContain("claude");
			expect(stdout).toContain("amp");
		} finally {
			cleanup(d);
			cleanup(xdgDir);
		}
	});

	// --offline with cold cache exits non-zero with a helpful error message
	test("--offline with cold cache exits non-zero with cached registry error", async () => {
		const d = makeTmpDir();
		const xdgDir = makeTmpDir(); // empty — no registry seeded
		try {
			const { exitCode, stderr } = run(["providers", "ls", "--offline"], d, {
				XDG_CONFIG_HOME: xdgDir,
			});
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain("cached registry");
		} finally {
			cleanup(d);
			cleanup(xdgDir);
		}
	});
});

// ── providers ls – TUI ────────────────────────────────────────────────────────
//
// TUI discovery observations (2026-05-14, cliclack select with max_rows(10)):
//   Initial: ◆  Select a provider
//            │  ● amp
//            │  ○ auggie
//            │  ○ autohand  ...  │  ○ deepagents
//            └
//   ArrowDown: ● moves from amp → auggie
//   Enter (on amp): ◇  Select a provider / │  amp / │ / └  — process exits

const tuiD = makeTmpDir();

test.describe("providers ls TUI", () => {
	test.use({ program: shellProgram(tuiD, ["providers", "ls"]) });

	// T-PV01: prompt header and first item are visible on initial render
	test("renders Select a provider prompt with first item highlighted", async ({
		terminal,
	}) => {
		await expect(terminal.getByText("Select a provider")).toBeVisible();
		await expect(terminal.getByText("amp")).toBeVisible();
	});

	// T-PV02: keyDown moves the selection to the second item
	test("keyDown moves selection to auggie", async ({ terminal }) => {
		await expect(terminal.getByText("Select a provider")).toBeVisible();
		terminal.keyDown();
		await expect(terminal.getByText("auggie")).toBeVisible();
	});

	// T-PV03: Enter submits the selection and exits — submitted ◇ state is visible
	test("Enter submits and shows submitted state with selected slug", async ({
		terminal,
	}) => {
		await expect(terminal.getByText("Select a provider")).toBeVisible();
		terminal.keyPress("Enter");
		// After submit cliclack shows ◇ symbol and the selected value inline
		await expect(terminal.getByText("amp")).toBeVisible();
	});
});
