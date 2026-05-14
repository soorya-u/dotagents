import { spawnSync } from "node:child_process";
import {
	mkdirSync,
	mkdtempSync,
	readFileSync,
	rmSync,
	writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

/// Absolute path to the release binary built by `cargo build --release`.
/// Tests are run with cwd=tests/e2e; the binary sits two levels up at <repo>/target/release/.
export const BIN = resolve(process.cwd(), "../../target/release/dotagents");

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
/// Pass `env` to merge extra environment variables into the child process environment.
export function run(
	args: string[],
	cwd: string,
	env?: Record<string, string>,
): { stdout: string; stderr: string; exitCode: number } {
	const result = spawnSync(BIN, args, {
		cwd,
		stdio: ["pipe", "pipe", "pipe"],
		env: env ? { ...process.env, ...env } : undefined,
	});
	return {
		stdout: result.stdout?.toString() ?? "",
		stderr: result.stderr?.toString() ?? "",
		exitCode: result.status ?? 1,
	};
}

/// Path to the local registry.json used for offline test fixtures.
const REGISTRY_JSON_PATH = resolve(
	process.cwd(),
	"../../public/v1/templates/registry.json",
);

/// Create a temp XDG config dir pre-seeded with the local registry.json.
/// Returns the XDG_CONFIG_HOME path; caller is responsible for cleanup().
export function seedRegistryCache(): string {
	const configDir = makeTmpDir();
	const cacheDir = join(configDir, "dotagents", "cache", "templates");
	mkdirSync(cacheDir, { recursive: true });
	writeFileSync(
		join(cacheDir, "registry.json"),
		readFileSync(REGISTRY_JSON_PATH, "utf8"),
	);
	return configDir;
}

/// Canonical setup for deploy tests: initializes a workspace with only the local provider.
/// Uses `init --template with-custom-provider`, which configures local.config.toml to
/// set `targets = []` and define only the mycode provider templates. This avoids
/// CI-unsafe external providers such as gemini (which requires a local cache file).
/// All deploy tests should use this helper unless they explicitly patch out unsafe providers.
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

/// Create two temp dirs — a "cwd" dir and a "workspace" dir containing .dotagents/.
/// Returns both paths; caller must clean up BOTH.
export function makeTwoDirs(): { cwd: string; workspace: string } {
	const cwd = makeTmpDir();
	const workspace = makeTmpDir();
	return { cwd, workspace };
}
