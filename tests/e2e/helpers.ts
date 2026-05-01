import { spawnSync } from "child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "fs";
import { join, resolve } from "path";
import { tmpdir } from "os";

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

/// Initialise a workspace in dir using `init --template with-custom-provider`,
/// then patch local.config.toml to drop the `gemini` registry target so deploy
/// works fully offline with only the local mycode templates.
export function initWithLocalProvider(dir: string): void {
  run(["init", "--template", "with-custom-provider"], dir);
  const path = join(dir, ".dotagents-debug/local.config.toml");
  const content = readFileSync(path, "utf8").replace(
    /targets\s*=\s*\["gemini"\]/,
    'targets = []',
  );
  writeFileSync(path, content);
}

/// Return a bash invocation suitable for test.use({ program: ... }) that
/// changes to `dir` then execs the binary with `args`. Using `exec` replaces
/// the shell process so the PTY is connected directly to dotagents.
export function shellProgram(
  dir: string,
  args: string[],
): { file: string; args: string[] } {
  const escaped = [dir, BIN, ...args].map((a) => `'${a}'`).join(" ");
  return {
    file: "bash",
    args: ["-c", `cd ${escaped.split(" ")[0]} && exec ${escaped.split(" ").slice(1).join(" ")}`],
  };
}
