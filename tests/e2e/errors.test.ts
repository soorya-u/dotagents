import { expect, test } from "@microsoft/tui-test";
import { cleanup, makeTmpDir, run } from "./helpers.js";

// C31–C34: error flows — binary gives helpful messages for common mistakes

test.describe("error flows – no workspace", () => {
	// C32: commands new outside workspace
	test("commands new without workspace exits non-zero with helpful message", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode, stderr } = run(
				["commands", "new", "hello", "--description", "x"],
				d,
			);
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain("dotagents init");
		} finally {
			cleanup(d);
		}
	});

	// C33: commands ls outside workspace
	test("commands ls without workspace exits non-zero mentioning init", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode, stderr } = run(["commands", "ls"], d);
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain("dotagents init");
		} finally {
			cleanup(d);
		}
	});

	// C31: deploy outside workspace — surfaces error through display_error()
	test("deploy without workspace exits non-zero with formatted error (not panic)", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode, stderr } = run(["deploy"], d);
			expect(exitCode).not.toBe(0);
			// Assert error passes through display_error (log::error! emits [ERROR] in non-TTY)
			expect(stderr).toContain("[ERROR]");
			// Assert no raw panic text
			expect(stderr).not.toContain("panicked at");
		} finally {
			cleanup(d);
		}
	});
});

test.describe("error flows – missing targets", () => {
	// C34: rm non-existent command
	test("rm non-existent command exits non-zero", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const { exitCode, stderr } = run(
				["commands", "rm", "ghost", "--force"],
				d,
			);
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain("ghost");
		} finally {
			cleanup(d);
		}
	});

	// rm non-existent skill
	test("rm non-existent skill exits non-zero", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			const { exitCode, stderr } = run(["skills", "rm", "ghost", "--force"], d);
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain("ghost");
		} finally {
			cleanup(d);
		}
	});

	// duplicate add without --force mentions --force in error
	test("duplicate commands new without --force mentions --force", async () => {
		const d = makeTmpDir();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				d,
			);
			// hello.md already exists from init
			const { exitCode, stderr } = run(
				["commands", "new", "hello", "--description", "x"],
				d,
			);
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain("--force");
		} finally {
			cleanup(d);
		}
	});
});
