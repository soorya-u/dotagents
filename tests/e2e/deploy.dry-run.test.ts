import { existsSync, readFileSync, unlinkSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "@microsoft/tui-test";
import { cleanup, initWithLocalProvider, makeTmpDir, run } from "./helpers.js";

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
			const cachePath = join(d, ".dotagents-debug/cache.toml");
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
				join(d, ".dotagents-debug/commands/hello.md"),
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
			unlinkSync(join(d, ".dotagents-debug/templates/mycode/instructions.hbs"));
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
