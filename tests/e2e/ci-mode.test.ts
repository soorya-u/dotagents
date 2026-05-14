import { existsSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "@microsoft/tui-test";
import { cleanup, initWithLocalProvider, makeTmpDir, run } from "./helpers.js";

test.describe("CI mode – --ci flag", () => {
	// --ci deploy exits 0 and writes files without interactive prompts
	test("--ci deploy completes without prompting", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode } = run(
				["--ci", "deploy", "--offline", "--no-gitignore"],
				d,
			);
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".mycode/commands/hello.md"))).toBe(true);
		} finally {
			cleanup(d);
		}
	});

	// --ci init completes without wizard and writes config.toml
	test("--ci init completes without wizard", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode } = run(["--ci", "init"], d);
			expect(exitCode).toBe(0);
			// debug builds use .dotagents-debug, release uses .dotagents
			const debugConfig = join(d, ".dotagents-debug/config.toml");
			const releaseConfig = join(d, ".dotagents/config.toml");
			expect(existsSync(debugConfig) || existsSync(releaseConfig)).toBe(true);
		} finally {
			cleanup(d);
		}
	});
});

test.describe("CI mode – DOTAGENTS_CI env var", () => {
	// DOTAGENTS_CI=true deploy behaves identically to --ci
	test("DOTAGENTS_CI=true deploy completes without prompting", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode } = run(["deploy", "--offline", "--no-gitignore"], d, {
				DOTAGENTS_CI: "true",
			});
			expect(exitCode).toBe(0);
			expect(existsSync(join(d, ".mycode/commands/hello.md"))).toBe(true);
		} finally {
			cleanup(d);
		}
	});

	// DOTAGENTS_CI=false leaves behavior unchanged (non-TTY already, exits 0)
	test("DOTAGENTS_CI=false does not break deploy", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode } = run(["deploy", "--offline", "--no-gitignore"], d, {
				DOTAGENTS_CI: "false",
			});
			expect(exitCode).toBe(0);
		} finally {
			cleanup(d);
		}
	});
});
