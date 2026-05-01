import { expect, test } from "@microsoft/tui-test";
import { cleanup, makeTmpDir, run } from "./helpers.js";

// ── CLI flows ────────────────────────────────────────────────────────────────
// Note: dotagents ls writes all output to stderr (via simplelog/cliclack).
// All tests check the `stderr` field from run() rather than stdout.

test.describe("ls CLI – output content", () => {
	// C20: ls shows both sections
	test("shows Commands and Skills sections", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode, stderr } = run(["ls"], d);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/Skills/);
			expect(stderr).toMatch(/Commands/);
		} finally {
			cleanup(d);
		}
	});

	// shows sample hello command and hello-skill
	test("shows hello command and hello-skill from init scaffold", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode, stderr } = run(["ls"], d);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/hello/);
			expect(stderr).toMatch(/hello-skill/);
		} finally {
			cleanup(d);
		}
	});

	// shows count summary
	test("shows count summary line", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode, stderr } = run(["ls"], d);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/skill\(s\)/);
			expect(stderr).toMatch(/command\(s\)/);
		} finally {
			cleanup(d);
		}
	});
});

test.describe("ls CLI – filter flags", () => {
	// C21: --commands shows only Commands section
	test("--commands hides Skills section", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode, stderr } = run(["ls", "--commands"], d);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/Commands/);
			expect(stderr).not.toMatch(/Skills/);
		} finally {
			cleanup(d);
		}
	});

	// C22: --skills shows only Skills section
	test("--skills hides Commands section", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode, stderr } = run(["ls", "--skills"], d);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/Skills/);
			expect(stderr).not.toMatch(/Commands/);
		} finally {
			cleanup(d);
		}
	});

	// C23: both flags together shows both sections
	test("--commands --skills shows both sections", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode, stderr } = run(["ls", "--commands", "--skills"], d);
			expect(exitCode).toBe(0);
			expect(stderr).toMatch(/Commands/);
			expect(stderr).toMatch(/Skills/);
		} finally {
			cleanup(d);
		}
	});
});

test.describe("ls CLI – verbose/full mode", () => {
	// C24: --full flag succeeds
	test("--full flag exits zero", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode } = run(["ls", "--full"], d);
			expect(exitCode).toBe(0);
		} finally {
			cleanup(d);
		}
	});

	// C25: -v flag enables full mode
	test("-v flag exits zero", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode } = run(["-v", "ls"], d);
			expect(exitCode).toBe(0);
		} finally {
			cleanup(d);
		}
	});
});
