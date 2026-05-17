import { expect, test } from "@microsoft/tui-test";
import { cleanup, makeTmpDir, run, seedRegistryCache } from "./helpers.js";

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
