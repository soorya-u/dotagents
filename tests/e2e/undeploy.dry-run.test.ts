import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "@microsoft/tui-test";
import { cleanup, initWithLocalProvider, makeTmpDir, run } from "./helpers.js";

// ── undeploy --dry-run ────────────────────────────────────────────────────────

test.describe("undeploy --dry-run – no side effects", () => {
	// U-DR01: --dry-run deletes no files
	test("deployed files remain on disk after dry run", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);
			expect(existsSync(join(d, ".mycode/commands/hello.md"))).toBe(true);

			const { exitCode } = run(["undeploy", "--dry-run"], d);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".mycode/commands/hello.md"))).toBe(true);
			expect(existsSync(join(d, ".mycode/instructions.md"))).toBe(true);
		} finally {
			cleanup(d);
		}
	});

	// U-DR02: --dry-run does not clear cache.toml
	test("cache.toml retains entries after dry run", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);
			const cacheBefore = readFileSync(
				join(d, ".dotagents-debug/cache.toml"),
				"utf8",
			);
			expect(cacheBefore).toContain("[providers.");

			run(["undeploy", "--dry-run"], d);

			const cacheAfter = readFileSync(
				join(d, ".dotagents-debug/cache.toml"),
				"utf8",
			);
			expect(cacheAfter).toBe(cacheBefore);
		} finally {
			cleanup(d);
		}
	});

	// U-DR03: --dry-run does not remove .gitignore fence
	test(".gitignore fence is preserved after dry run", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--gitignore"], d);

			const giBefore = readFileSync(join(d, ".gitignore"), "utf8");
			expect(giBefore).toContain("#region dotagents");

			run(["undeploy", "--dry-run"], d);

			const giAfter = readFileSync(join(d, ".gitignore"), "utf8");
			expect(giAfter).toContain("#region dotagents");
		} finally {
			cleanup(d);
		}
	});
});

test.describe("undeploy --dry-run – output format", () => {
	// U-DR04: stdout shows [-] for unmodified cached files
	test("stdout contains [-] for each unmodified deployed file", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);
			const { stdout, exitCode } = run(["undeploy", "--dry-run"], d);
			expect(exitCode).toBe(0);
			expect(stdout).toContain("[-]");
		} finally {
			cleanup(d);
		}
	});

	// U-DR05: stdout shows [x] for user-edited files
	test("stdout contains [x] for user-edited deployed file", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);
			// Simulate user editing a deployed file
			writeFileSync(
				join(d, ".mycode/instructions.md"),
				"user has manually edited this",
			);
			const { stdout, exitCode } = run(["undeploy", "--dry-run"], d);
			expect(exitCode).toBe(0);
			expect(stdout).toContain("[x]");
			expect(stdout).toContain("(edited)");
		} finally {
			cleanup(d);
		}
	});

	// U-DR06: stdout contains summary line
	test("stdout contains 'files would be affected'", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);
			const { stdout } = run(["undeploy", "--dry-run"], d);
			expect(stdout).toContain("files would be affected");
		} finally {
			cleanup(d);
		}
	});

	// U-DR07: stdout contains dry run header
	test("stdout contains dry run header", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);
			const { stdout } = run(["undeploy", "--dry-run"], d);
			expect(stdout).toContain("Dry run");
		} finally {
			cleanup(d);
		}
	});
});

test.describe("undeploy --dry-run – empty cache", () => {
	// U-DR08: empty cache exits 0 with 0 files would be affected
	test("empty cache exits 0 with 0 files would be affected", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			// No deploy run — cache.toml does not exist
			const { exitCode, stdout } = run(["undeploy", "--dry-run"], d);
			expect(exitCode).toBe(0);
			expect(stdout).toContain("0 files would be affected");
		} finally {
			cleanup(d);
		}
	});
});
