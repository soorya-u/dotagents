import { spawnSync } from "node:child_process";
import {
	mkdirSync,
	mkdtempSync,
	readFileSync,
	existsSync,
	rmSync,
	writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

/// Absolute path to the release binary built by `cargo build --release`.
/// Tests are run with cwd=tests/e2e; the binary sits two levels up at <repo>/target/release/.
export const BIN = resolve(process.cwd(), "../../target/release/dotagents");

/// Directories created by makeTmpDir that need cleanup when the worker process exits.
/// tui-test runs each test file in an isolated worker; registering on process exit
/// ensures cleanup runs even if afterAll hooks are lost during suite serialization.
const CLEANUP_DIRS = new Set<string>();

let cleanupRegistered = false;
function ensureCleanupRegistered(): void {
	if (cleanupRegistered) return;
	cleanupRegistered = true;

	// Synchronous cleanup for process exit events (async handlers are ignored).
	process.on("exit", () => {
		for (const dir of CLEANUP_DIRS) {
			try {
				rmSync(dir, { recursive: true, force: true });
			} catch (err) {
				process.stderr.write(`cleanup failed for ${dir}: ${err}\n`);
			}
		}
	});
}

/// Create a fresh isolated temp directory for a single test
export function makeTmpDir(): string {
	ensureCleanupRegistered();
	const dir = mkdtempSync(join(tmpdir(), "dotagents-test-"));
	CLEANUP_DIRS.add(dir);
	return dir;
}

/// Remove a temp directory created by makeTmpDir
export function cleanup(dir: string): void {
	rmSync(dir, { recursive: true, force: true });
	CLEANUP_DIRS.delete(dir);
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
/// Uses `init --template advanced`, which configures local.config.toml to
/// set `targets = []` and define only the mycode provider templates. This avoids
/// CI-unsafe external providers such as gemini (which requires a local cache file).
/// All deploy tests should use this helper unless they explicitly patch out unsafe providers.
export function initWithLocalProvider(dir: string): void {
	run(
		[
			"init",
			"--template",
			"advanced",
			"--features",
			"command,instruction,mcp,skill",
		],
		dir,
	);
	// Enable template mode for Type 2 features so variable/var injection works.
	const rootDir = existsSync(join(dir, ".dotagents-debug"))
		? ".dotagents-debug"
		: ".dotagents";
	const localConfig = join(dir, rootDir, "local.config.toml");
	writeFileSync(
		localConfig,
		readFileSync(localConfig, "utf8") +
			'\n[feature-maps.instruction]\nmode = "template"\n' +
			'[feature-maps.command]\nmode = "template"\n',
	);
}

/// Return a bash invocation suitable for test.use({ program: ... }) that
/// changes to `dir` then execs the binary with `args`. Using `exec` replaces
/// the shell process so the PTY is connected directly to dotagents.
/// Single-quote a shell argument, escaping any embedded single quotes.
/// Pass `env` to set environment variables in the shell invocation.
function quoteShellArg(value: string): string {
	return `'${value.replace(/'/g, "'\\''")}'`;
}

export function shellProgram(
	dir: string,
	args: string[],
	env?: Record<string, string>,
): { file: string; args: string[] } {
	const envPrefix = env
		? `${Object.entries(env)
				.map(([k, v]) => `${k}=${quoteShellArg(v)}`)
				.join(" ")} `
		: "";
	return {
		file: "bash",
		args: [
			"-c",
			`cd ${quoteShellArg(dir)} && ${envPrefix}exec ${quoteShellArg(BIN)} ${args.map(quoteShellArg).join(" ")}`,
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
