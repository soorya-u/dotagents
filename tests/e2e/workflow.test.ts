import { test, expect } from "@microsoft/tui-test";
import { existsSync, readFileSync, writeFileSync } from "fs";
import { join } from "path";
import { cleanup, initWithLocalProvider, makeTmpDir, run, shellProgram } from "./helpers.js";

// ── Journey flows: multi-command user stories ────────────────────────────────

test.describe("J01-J02: init → add → deploy", () => {
  // J01: command deployed with correct content, no frontmatter
  test("init → add command → deploy → output file has no frontmatter", async () => {
    const d = makeTmpDir();
    try {
      initWithLocalProvider(d);
      run(["add", "command", "greet", "--description", "Greet user", "--category", "General", "--tags", "greet"], d);
      const { exitCode } = run(["deploy", "--offline", "--no-gitignore"], d);
      expect(exitCode).toBe(0);

      const output = join(d, ".mycode/commands/greet.md");
      expect(existsSync(output)).toBe(true);
      const content = readFileSync(output, "utf8");
      expect(content).not.toMatch(/^---/);
      expect(content).toContain("greet");
    } finally {
      cleanup(d);
    }
  });

  // J02: skill deployed correctly
  test("init → add skill → deploy → skill output exists", async () => {
    const d = makeTmpDir();
    try {
      initWithLocalProvider(d);
      run(["add", "skill", "my-skill", "--description", "Test skill"], d);
      run(["deploy", "--offline", "--no-gitignore"], d);

      expect(existsSync(join(d, ".mycode/skills/my-skill/SKILLS.md"))).toBe(true);
    } finally {
      cleanup(d);
    }
  });
});

test.describe("J03-J04: CRUD – add, list, remove", () => {
  // J03: full CRUD for commands — uses run() + stderr for ls check
  test("add command → ls shows it → rm → ls no longer shows it", async () => {
    const d = makeTmpDir();
    try {
      run(["init", "--template", "starter"], d);
      run(["add", "command", "greet", "--description", "test"], d);

      // ls shows the new command
      const { stderr } = run(["ls", "--commands"], d);
      expect(stderr).toMatch(/greet/);

      // rm it
      run(["rm", "command", "greet", "--force"], d);
      expect(existsSync(join(d, ".dotagents-debug/commands/greet.md"))).toBe(false);

      // ls no longer shows greet as a command name (hello's description contains "greet" so use stricter pattern)
      const { stderr: afterStderr } = run(["ls", "--commands"], d);
      expect(afterStderr).not.toMatch(/^\[.*INFO.*\]\s+greet\s/m);
    } finally {
      cleanup(d);
    }
  });

  // J04: full CRUD for skills
  test("add skill → rm → skill gone", async () => {
    const d = makeTmpDir();
    try {
      run(["init", "--template", "starter"], d);
      run(["add", "skill", "my-skill", "--description", "test"], d);
      expect(existsSync(join(d, ".dotagents-debug/skills/my-skill"))).toBe(true);

      run(["rm", "skill", "my-skill", "--force"], d);
      expect(existsSync(join(d, ".dotagents-debug/skills/my-skill"))).toBe(false);
    } finally {
      cleanup(d);
    }
  });
});

test.describe("J05-J06: redeploy and idempotency", () => {
  // J05: modifying source updates output on next deploy
  test("editing source is reflected in subsequent deploy", async () => {
    const d = makeTmpDir();
    try {
      initWithLocalProvider(d);
      run(["deploy", "--no-cache", "--offline", "--no-gitignore"], d);

      // edit INSTRUCTIONS.md
      const src = join(d, ".dotagents-debug/INSTRUCTIONS.md");
      writeFileSync(src, readFileSync(src, "utf8") + "\n\nCustom addition.");
      run(["deploy", "--no-cache", "--offline", "--no-gitignore"], d);

      const output = readFileSync(join(d, ".mycode/instructions.md"), "utf8");
      expect(output).toContain("Custom addition.");
    } finally {
      cleanup(d);
    }
  });

  // J06: idempotency — two deploys produce identical output
  test("two deploys without changes produce identical output files", async () => {
    const d = makeTmpDir();
    try {
      initWithLocalProvider(d);
      run(["deploy", "--no-cache", "--offline", "--no-gitignore"], d);
      const first = {
        cmd: readFileSync(join(d, ".mycode/commands/hello.md"), "utf8"),
        instr: readFileSync(join(d, ".mycode/instructions.md"), "utf8"),
        mcp: readFileSync(join(d, ".mycode/mcp.json"), "utf8"),
      };
      run(["deploy", "--no-cache", "--offline", "--no-gitignore"], d);
      const second = {
        cmd: readFileSync(join(d, ".mycode/commands/hello.md"), "utf8"),
        instr: readFileSync(join(d, ".mycode/instructions.md"), "utf8"),
        mcp: readFileSync(join(d, ".mycode/mcp.json"), "utf8"),
      };
      expect(first.cmd).toBe(second.cmd);
      expect(first.instr).toBe(second.instr);
      expect(first.mcp).toBe(second.mcp);
    } finally {
      cleanup(d);
    }
  });
});

// J07: TUI init wizard → patch config → CLI deploy → verify output.
// test.use() is at describe level; workspace starts empty and is populated by the wizard.
test.describe("J07: full interactive journey", () => {
  const d = makeTmpDir();
  test.use({ program: shellProgram(d, ["init"]) });

  test("wizard init → deploy produces output", async ({ terminal }) => {
    try {
      await expect(terminal.getByText("Which features do you want to enable?")).toBeVisible();
      terminal.keyPress("Enter"); // accept all features
      await expect(terminal.getByText("Which starting template?")).toBeVisible();
      // move down to WithCustomProvider
      terminal.keyDown();
      terminal.keyPress("Enter");
      terminal.keyPress("Enter"); // skip providers
      await expect(terminal.getByText("Done! Run")).toBeVisible();
      terminal.kill();

      // patch out gemini target
      const lcPath = join(d, ".dotagents-debug/local.config.toml");
      writeFileSync(lcPath, readFileSync(lcPath, "utf8").replace(/targets\s*=\s*\["gemini"\]/, 'targets = []'));

      // deploy via CLI
      const { exitCode } = run(["deploy", "--offline", "--no-gitignore"], d);
      expect(exitCode).toBe(0);
      expect(existsSync(join(d, ".mycode/commands/hello.md"))).toBe(true);
    } finally {
      cleanup(d);
    }
  });
});

test.describe("J08: full CRUD both types", () => {
  // J08: add command and skill, rm both, ls shows workspace is empty
  test("add command and skill, rm both, ls shows empty workspace", async () => {
    const d = makeTmpDir();
    try {
      run(["init", "--template", "starter"], d);

      // Add a second command and skill beyond the init scaffold
      run(["add", "command", "greet", "--description", "greet"], d);
      run(["add", "skill", "my-skill", "--description", "skill"], d);

      // Remove all commands and skills including scaffold items
      run(["rm", "command", "hello", "--force"], d);
      run(["rm", "command", "greet", "--force"], d);
      run(["rm", "skill", "hello-skill", "--force"], d);
      run(["rm", "skill", "my-skill", "--force"], d);

      // ls shows no items left
      const { exitCode, stderr } = run(["ls"], d);
      expect(exitCode).toBe(0);
      expect(stderr).toMatch(/No skills or commands found/);
    } finally {
      cleanup(d);
    }
  });
});
