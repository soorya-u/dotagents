import { test, expect } from "@microsoft/tui-test";
import { existsSync, mkdirSync, readFileSync, statSync } from "fs";
import { join } from "path";
import { BIN, cleanup, makeTmpDir, run } from "./helpers.js";

test.describe("gen-completions CLI", () => {
  // C28: bash completions
  test("generates non-empty dotagents.bash for bash", async () => {
    const d = makeTmpDir();
    try {
      mkdirSync(join(d, "out"));
      const { exitCode } = run(["gen-completions", "--shell", "bash", "--to", "./out"], d);
      expect(exitCode).toBe(0);
      const file = join(d, "out/dotagents.bash");
      expect(existsSync(file)).toBe(true);
      expect(statSync(file).size).toBeGreaterThan(0);
    } finally {
      cleanup(d);
    }
  });

  // C29: zsh completions
  test("generates non-empty _dotagents for zsh", async () => {
    const d = makeTmpDir();
    try {
      mkdirSync(join(d, "out"));
      const { exitCode } = run(["gen-completions", "--shell", "zsh", "--to", "./out"], d);
      expect(exitCode).toBe(0);
      const file = join(d, "out/_dotagents");
      expect(existsSync(file)).toBe(true);
      expect(statSync(file).size).toBeGreaterThan(0);
    } finally {
      cleanup(d);
    }
  });

  // C30: fish completions
  test("generates non-empty dotagents.fish for fish", async () => {
    const d = makeTmpDir();
    try {
      mkdirSync(join(d, "out"));
      const { exitCode } = run(["gen-completions", "--shell", "fish", "--to", "./out"], d);
      expect(exitCode).toBe(0);
      const file = join(d, "out/dotagents.fish");
      expect(existsSync(file)).toBe(true);
      expect(statSync(file).size).toBeGreaterThan(0);
    } finally {
      cleanup(d);
    }
  });

  // completions file contains binary name
  test("bash completion file references the binary name", async () => {
    const d = makeTmpDir();
    try {
      mkdirSync(join(d, "out"));
      run(["gen-completions", "--shell", "bash", "--to", "./out"], d);
      const content = readFileSync(join(d, "out/dotagents.bash"), "utf8");
      expect(content).toContain("dotagents");
    } finally {
      cleanup(d);
    }
  });

  // gen-completions does not require an init'd workspace
  test("succeeds without an initialised workspace", async () => {
    const d = makeTmpDir();
    try {
      mkdirSync(join(d, "out"));
      const { exitCode } = run(["gen-completions", "--shell", "bash", "--to", "./out"], d);
      expect(exitCode).toBe(0);
    } finally {
      cleanup(d);
    }
  });
});
