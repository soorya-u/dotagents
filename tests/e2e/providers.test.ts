import { expect, test } from "@microsoft/tui-test";
import {
	cleanup,
	makeTmpDir,
	run,
	seedRegistryCache,
	shellProgram,
} from "./helpers.js";

test.describe("providers ls CLI", () => {
	// --json exits 0 and returns a valid JSON array with slug/name/url fields
	test("--json exits 0 and returns valid JSON array", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode, stdout } = run(["providers", "--json"], d);
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
				["providers", "--json", "--offline"],
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
			const { exitCode, stdout } = run(["providers", "--offline"], d, {
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
			const { exitCode, stderr } = run(["providers", "--offline"], d, {
				XDG_CONFIG_HOME: xdgDir,
			});
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain("cached registry");
			expect(stderr).toContain("dotagents providers");
			expect(stderr).not.toContain("dotagents providers ls");
		} finally {
			cleanup(d);
			cleanup(xdgDir);
		}
	});

	// --verbose with cold cache shows debug line with registry URL before failing
	test("-v with cold cache shows debug line with registry URL", async () => {
		const d = makeTmpDir();
		const xdgDir = makeTmpDir(); // empty — no registry seeded
		try {
			const { stderr } = run(["-v", "providers"], d, {
				XDG_CONFIG_HOME: xdgDir,
			});
			// Should attempt fetch (showing debug URL) then fail with cache error
			expect(stderr).toMatch(/Fetching provider registry from https?:\/\//);
		} finally {
			cleanup(d);
			cleanup(xdgDir);
		}
	});
});

// ── Providers --quiet / --verbose ─────────────────────────────────────────────

test.describe("providers CLI – --quiet flag", () => {
	// TC-PROV-09: --quiet suppresses provider listing output
	test("--quiet --offline with seeded cache produces empty stdout", async () => {
		const d = makeTmpDir();
		const xdgDir = seedRegistryCache();
		try {
			const { exitCode, stdout } = run(
				["providers", "--ci", "--quiet", "--offline"],
				d,
				{ XDG_CONFIG_HOME: xdgDir },
			);
			expect(exitCode).toBe(0);
			expect(stdout).toBe("");
		} finally {
			cleanup(d);
			cleanup(xdgDir);
		}
	});
});

test.describe("providers CLI – --verbose flag", () => {
	// TC-PROV-10: -v adds diagnostic output with seeded cache
	test("-v --offline with seeded cache shows debug output on stderr", async () => {
		const d = makeTmpDir();
		const xdgDir = seedRegistryCache();
		try {
			const { exitCode, stderr } = run(
				["providers", "--ci", "-v", "--offline"],
				d,
				{ XDG_CONFIG_HOME: xdgDir },
			);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/cache|Cache|CACHE|offline|Offline/);
		} finally {
			cleanup(d);
			cleanup(xdgDir);
		}
	});
});

// ── Providers TUI ─────────────────────────────────────────────────────────────

// TC-PROV-01: TUI select widget renders and is navigable
test.describe("providers TUI – TC-PROV-01 select widget", () => {
	const d = makeTmpDir();
	const xdgDir = seedRegistryCache();
	test.use({
		program: shellProgram(d, ["providers", "--offline"], {
			XDG_CONFIG_HOME: xdgDir,
		}),
	});

	test("select widget renders, Enter selects, shows outro", async ({
		terminal,
	}) => {
		try {
			await expect(terminal.getByText("Providers")).toBeVisible();
			// "amp" is the first provider alphabetically by slug
			await expect(terminal.getByText("amp")).toBeVisible();

			// Press Enter to select the highlighted provider
			terminal.keyPress("Enter");

			// Outro should show with provider name
			await expect(terminal.getByText("Amp Code")).toBeVisible();
		} finally {
			cleanup(d);
			cleanup(xdgDir);
		}
	});
});
