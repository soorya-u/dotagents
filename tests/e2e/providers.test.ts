import { expect, test } from "@microsoft/tui-test";
import { cleanup, makeTmpDir, run, shellProgram } from "./helpers.js";

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

	// default text output lists providers in Name [slug] (url) format
	test("default text output contains provider slugs in brackets", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode, stdout } = run(["providers", "--ci"], d);
			expect(exitCode).toBe(0);
			expect(stdout).toContain("[claude]");
			expect(stdout).toContain("[amp]");
		} finally {
			cleanup(d);
		}
	});

	// --verbose shows debug line with registry URL
	test("-v shows debug line with registry URL", async () => {
		const d = makeTmpDir();
		try {
			const { stderr } = run(["-v", "providers"], d);
			expect(stderr).toMatch(/Fetching provider registry from https?:\/\//);
		} finally {
			cleanup(d);
		}
	});
});

// ── Providers --quiet / --verbose ─────────────────────────────────────────────

test.describe("providers CLI – --quiet flag", () => {
	// --quiet suppresses provider listing output
	test("--quiet produces empty stdout", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode, stdout } = run(["providers", "--ci", "--quiet"], d);
			expect(exitCode).toBe(0);
			expect(stdout).toBe("");
		} finally {
			cleanup(d);
		}
	});
});

test.describe("providers CLI – --verbose flag", () => {
	// -v adds diagnostic output
	test("-v shows debug output on stderr", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode, stderr } = run(["providers", "--ci", "-v"], d);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/cache|Cache|CACHE|fetch|Fetch/);
		} finally {
			cleanup(d);
		}
	});
});

// ── Providers TUI ─────────────────────────────────────────────────────────────

// TC-PROV-01: TUI select widget renders and is navigable
test.describe("providers TUI – TC-PROV-01 select widget", () => {
	const d = makeTmpDir();
	test.use({
		program: shellProgram(d, ["providers"]),
	});

	test("select widget renders, Enter selects, shows outro", async ({
		terminal,
	}) => {
		await expect(terminal.getByText("Providers")).toBeVisible();
		// First item after alpha sort by slug is "adal"
		await expect(terminal.getByText("adal")).toBeVisible();

		// Press Enter to select the highlighted (first) provider
		terminal.keyPress("Enter");

		// Outro should show with provider name (from registry)
		await expect(terminal.getByText("Adal (https://adal.dev)")).toBeVisible();
	});
});
