import { existsSync, readFileSync, writeFileSync } from "node:fs";
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

test.describe("undeploy CLI – basic lifecycle", () => {
	// U01: deploy then undeploy removes deployed files
	test("deploy → undeploy removes all deployed output files", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);
			expect(existsSync(join(d, ".mycode/instructions.md"))).toBe(true);
			expect(existsSync(join(d, ".mycode/commands/hello.md"))).toBe(true);

			const { exitCode } = run(["undeploy", "--no-gitignore"], d);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".mycode/instructions.md"))).toBe(false);
			expect(existsSync(join(d, ".mycode/commands/hello.md"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});

	// U02: undeploy clears cache entries
	test("undeploy clears provider entries in cache.toml", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);
			run(["undeploy", "--no-gitignore"], d);

			const cachePath = join(d, ".dotagents-debug/cache.toml");
			if (existsSync(cachePath)) {
				const content = readFileSync(cachePath, "utf8");
				expect(content).not.toContain("[providers.");
			}
		} finally {
			cleanup(d);
		}
	});

	// U03: undeploy with empty/absent cache exits 0 cleanly
	test("undeploy with no prior deploy exits 0", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			// No deploy run — cache.toml does not exist
			const { exitCode } = run(["undeploy", "--no-gitignore"], d);
			expect(exitCode).toBe(0);
		} finally {
			cleanup(d);
		}
	});

	// U04: undeploy twice is idempotent (second call is a safe no-op)
	test("undeploy twice is safe", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);
			run(["undeploy", "--no-gitignore"], d);
			const { exitCode } = run(["undeploy", "--no-gitignore"], d);
			expect(exitCode).toBe(0);
		} finally {
			cleanup(d);
		}
	});
});

test.describe("undeploy CLI – gitignore handling", () => {
	// U05: --no-gitignore preserves the managed fence after undeploy
	test("--no-gitignore preserves .gitignore fence", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--gitignore"], d);

			const giBefore = readFileSync(join(d, ".gitignore"), "utf8");
			expect(giBefore).toContain("BEGIN dotagents managed");

			run(["undeploy", "--no-gitignore"], d);

			const giAfter = readFileSync(join(d, ".gitignore"), "utf8");
			expect(giAfter).toContain("BEGIN dotagents managed");
		} finally {
			cleanup(d);
		}
	});

	// U06: default undeploy removes the managed fence from .gitignore
	test("undeploy removes .gitignore fence by default", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--gitignore"], d);

			const giBefore = readFileSync(join(d, ".gitignore"), "utf8");
			expect(giBefore).toContain("BEGIN dotagents managed");

			run(["undeploy"], d);

			const giAfter = readFileSync(join(d, ".gitignore"), "utf8");
			expect(giAfter).not.toContain("BEGIN dotagents managed");
		} finally {
			cleanup(d);
		}
	});
});

test.describe("undeploy CLI – --no-cache integration", () => {
	// U07: deploy --no-cache still writes cache.toml so undeploy can read it
	test("deploy --no-cache still enables subsequent undeploy", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--no-cache", "--offline", "--no-gitignore"], d);
			expect(existsSync(join(d, ".mycode/instructions.md"))).toBe(true);

			const { exitCode } = run(["undeploy", "--no-gitignore"], d);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".mycode/instructions.md"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});

	// U08: deploy --no-cache writes cache.toml with provider entries
	test("deploy --no-cache writes cache.toml", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--no-cache", "--offline", "--no-gitignore"], d);

			const cachePath = join(d, ".dotagents-debug/cache.toml");
			expect(existsSync(cachePath)).toBe(true);
			const content = readFileSync(cachePath, "utf8");
			expect(content).toContain("[providers.");
		} finally {
			cleanup(d);
		}
	});
});

test.describe("undeploy CLI – user-edited files", () => {
	// U09: non-TTY skips user-edited files (hash mismatch → warn + keep)
	test("user-edited file is preserved in non-TTY mode", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);

			// Simulate user editing a deployed output
			writeFileSync(
				join(d, ".mycode/instructions.md"),
				"User has manually edited this file.",
			);

			run(["undeploy", "--no-gitignore"], d);

			// Hash mismatch + non-TTY → file skipped, still present
			expect(existsSync(join(d, ".mycode/instructions.md"))).toBe(true);
		} finally {
			cleanup(d);
		}
	});

	// U10: --force deletes user-edited files without prompting
	test("--force deletes user-edited files", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);

			writeFileSync(
				join(d, ".mycode/instructions.md"),
				"User has manually edited this file.",
			);

			const { exitCode } = run(
				["undeploy", "--no-gitignore", "--force"],
				d,
			);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".mycode/instructions.md"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});
});

test.describe("undeploy CLI – empty dir pruning", () => {
	// U11: empty parent dir is removed after all its files are undeployed
	test("empty .mycode/commands/ dir is pruned after undeploy", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);
			expect(existsSync(join(d, ".mycode/commands"))).toBe(true);

			run(["undeploy", "--no-gitignore", "--force"], d);

			expect(existsSync(join(d, ".mycode/commands"))).toBe(false);
		} finally {
			cleanup(d);
		}
	});
});

// ── TUI flows ────────────────────────────────────────────────────────────────
// Each TUI test lives in its own describe block so test.use() applies only to it.

// T-D1: deploy outro prints "N written, M skipped" summary in TTY
test.describe("deploy TUI – T-D1 outro summary", () => {
	const d = makeTmpDir();
	initWithLocalProvider(d);
	test.use({ program: shellProgram(d, ["deploy", "--offline", "--no-gitignore"]) });

	test("deploy prints written/skipped summary to TTY", async ({ terminal }) => {
		try {
			// --offline skips the interactive prompt; outro appears after deploy
			await expect(terminal.getByText("written")).toBeVisible();
		} finally {
			cleanup(d);
		}
	});
});

// T-U1: undeploy TTY confirmation – selecting Yes proceeds and prints summary
test.describe("undeploy TUI – T-U1 confirm Yes removes files", () => {
	const d = makeTmpDir();
	initWithLocalProvider(d);
	run(["deploy", "--offline", "--no-gitignore"], d);
	test.use({ program: shellProgram(d, ["undeploy", "--no-gitignore"]) });

	test("selecting Yes on confirmation prompt removes deployed files", async ({
		terminal,
	}) => {
		try {
			// Confirmation prompt appears
			await expect(terminal.getByText("deployed file")).toBeVisible();
			// Default selection is "No" — move down to "Yes" and confirm
			terminal.keyDown();
			terminal.keyPress("Enter");
			// Summary is printed after successful removal
			await expect(terminal.getByText("removed")).toBeVisible();
		} finally {
			cleanup(d);
		}
	});
});

// T-U2: undeploy TTY confirmation – pressing Enter on default No aborts
test.describe("undeploy TUI – T-U2 confirm No aborts", () => {
	const d = makeTmpDir();
	initWithLocalProvider(d);
	run(["deploy", "--offline", "--no-gitignore"], d);
	test.use({ program: shellProgram(d, ["undeploy", "--no-gitignore"]) });

	test("pressing Enter on default No aborts undeploy", async ({ terminal }) => {
		try {
			// Confirmation prompt appears
			await expect(terminal.getByText("deployed file")).toBeVisible();
			// Accept the default "No" selection
			terminal.keyPress("Enter");
			// Process exits silently; no summary should be printed
		} finally {
			// Files should still exist since undeploy was aborted
			expect(existsSync(join(d, ".mycode/instructions.md"))).toBe(true);
			cleanup(d);
		}
	});
});
