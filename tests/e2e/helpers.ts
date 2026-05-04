import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

/// Absolute path to the debug binary built by `cargo build`.
/// Tests are run with cwd=tests/e2e; the binary sits two levels up at <repo>/target/debug/.
export const BIN = resolve(process.cwd(), "../../target/debug/dotagents");

/// Create a fresh isolated temp directory for a single test
export function makeTmpDir(): string {
	return mkdtempSync(join(tmpdir(), "dotagents-test-"));
}

/// Remove a temp directory created by makeTmpDir
export function cleanup(dir: string): void {
	rmSync(dir, { recursive: true, force: true });
}

/// Run the binary with the given args in the given working directory (non-TTY).
/// Always captures both stdout and stderr so callers can inspect both streams.
export function run(
	args: string[],
	cwd: string,
): { stdout: string; stderr: string; exitCode: number } {
	const result = spawnSync(BIN, args, {
		cwd,
		stdio: ["pipe", "pipe", "pipe"],
	});
	return {
		stdout: result.stdout?.toString() ?? "",
		stderr: result.stderr?.toString() ?? "",
		exitCode: result.status ?? 1,
	};
}

/// Initialise a workspace in dir using `init --template with-custom-provider`.
/// The generated local.config.toml already has `targets = []` and the mycode
/// provider sections, so deploy works fully offline with only the local templates.
export function initWithLocalProvider(dir: string): void {
	run(["init", "--template", "with-custom-provider"], dir);
}

/// Return a bash invocation suitable for test.use({ program: ... }) that
/// changes to `dir` then execs the binary with `args`. Using `exec` replaces
/// the shell process so the PTY is connected directly to dotagents.
/// Single-quote a shell argument, escaping any embedded single quotes.
function quoteShellArg(value: string): string {
	return `'${value.replace(/'/g, "'\\''")}'`;
}

export function shellProgram(
	dir: string,
	args: string[],
): { file: string; args: string[] } {
	return {
		file: "bash",
		args: [
			"-c",
			`cd ${quoteShellArg(dir)} && exec ${quoteShellArg(BIN)} ${args.map(quoteShellArg).join(" ")}`,
		],
	};
}
