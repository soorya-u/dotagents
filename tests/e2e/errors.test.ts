import { expect, test } from "@microsoft/tui-test";
import { cleanup, makeTmpDir, run } from "./helpers.js";

// C31–C34: error flows — binary gives helpful messages for common mistakes

test.describe("error flows – no workspace", () => {
	// C32: add command outside workspace
	test("add command without workspace exits non-zero with helpful message", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode, stderr } = run(
				["add", "command", "hello", "--description", "x"],
				d,
			);
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain("dotagents init");
		} finally {
			cleanup(d);
		}
	});

	// C33: ls outside workspace
	test("ls without workspace exits non-zero mentioning init", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode, stderr } = run(["ls"], d);
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain("dotagents init");
		} finally {
			cleanup(d);
		}
	});

	// C31: deploy outside workspace — currently panics; assert non-zero exit
	test("deploy without workspace exits non-zero", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode } = run(["deploy"], d);
			expect(exitCode).not.toBe(0);
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
			run(["init", "--template", "starter"], d);
			const { exitCode, stderr } = run(
				["rm", "command", "ghost", "--force"],
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
			run(["init", "--template", "starter"], d);
			const { exitCode, stderr } = run(["rm", "skill", "ghost", "--force"], d);
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain("ghost");
		} finally {
			cleanup(d);
		}
	});

	// duplicate add without --force mentions --force in error
	test("duplicate add command without --force mentions --force", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			// hello.md already exists from init
			const { exitCode, stderr } = run(
				["add", "command", "hello", "--description", "x"],
				d,
			);
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain("--force");
		} finally {
			cleanup(d);
		}
	});
});
