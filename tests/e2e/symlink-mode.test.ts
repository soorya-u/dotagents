import {
	existsSync,
	lstatSync,
	mkdirSync,
	readFileSync,
	readlinkSync,
	writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { expect, test } from "@microsoft/tui-test";
import { cleanup, initWithLocalProvider, makeTmpDir, run } from "./helpers.js";

/// Resolve the root dir name (`.dotagents` for release, `.dotagents-debug` for debug).
function rootDir(dir: string): string {
	return existsSync(join(dir, ".dotagents-debug"))
		? ".dotagents-debug"
		: ".dotagents";
}

/// Append a `[feature-maps.<feature>] mode = "..."` block to local.config.toml.
function setFeatureMode(dir: string, feature: string, mode: string): void {
	const localConfig = join(dir, rootDir(dir), "local.config.toml");
	writeFileSync(
		localConfig,
		readFileSync(localConfig, "utf8") +
			`\n[feature-maps.${feature}]\nmode = "${mode}"\n`,
	);
}

/// Asserts that `path` is a symbolic link (not a regular file or directory).
function expectSymlink(path: string): void {
	expect(existsSync(path)).toBe(true);
	const stat = lstatSync(path);
	expect(stat.isSymbolicLink()).toBe(true);
}

/// Asserts that `path` is a regular file (not a symlink).
function expectRegularFile(path: string): void {
	expect(existsSync(path)).toBe(true);
	const stat = lstatSync(path);
	expect(stat.isFile()).toBe(true);
	expect(stat.isSymbolicLink()).toBe(false);
}

// ── deploy skills with mode=link creates symlinks ────────────────────────────

test.describe("symlink-mode: deploy skills mode=link", () => {
	// skills deploy as symlinks by default (link mode)
	test("deploys skill as a symlink pointing to source", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);

			const target = join(d, ".mycode/skills/hello-skill/SKILL.md");
			expectSymlink(target);

			const source = join(d, `${rootDir(d)}/skills/hello-skill/SKILL.md`);
			const linkTarget = readlinkSync(target);
			expect(linkTarget.toString()).toContain("SKILL.md");
			expect(existsSync(source)).toBe(true);
		} finally {
			cleanup(d);
		}
	});

	// symlinked skill content matches source content
	test("symlinked skill resolves to source content", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);

			const target = join(d, ".mycode/skills/hello-skill/SKILL.md");
			const source = join(d, `${rootDir(d)}/skills/hello-skill/SKILL.md`);
			expect(readFileSync(target, "utf8")).toBe(readFileSync(source, "utf8"));
		} finally {
			cleanup(d);
		}
	});

	// redeploy overwrites an existing symlink without error
	test("redeploy overwrites existing symlink", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);
			const target = join(d, ".mycode/skills/hello-skill/SKILL.md");
			expectSymlink(target);
			// Redeploy should succeed and still produce a symlink
			const { exitCode } = run(["deploy", "--offline", "--no-gitignore"], d);
			expect(exitCode).toBe(0);
			expectSymlink(target);
		} finally {
			cleanup(d);
		}
	});
});

// ── deploy skills with mode=template writes files ────────────────────────────

test.describe("symlink-mode: deploy skills mode=template", () => {
	// skills deploy as regular files when mode=template is set
	test("deploys skill as a regular file when mode=template", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			setFeatureMode(d, "skill", "template");
			run(["deploy", "--offline", "--no-gitignore"], d);

			const target = join(d, ".mycode/skills/hello-skill/SKILL.md");
			expectRegularFile(target);
		} finally {
			cleanup(d);
		}
	});

	// template-mode skill content preserves key metadata and body (YAML roundtrip may change quoting)
	test("template-mode skill content preserves key fields", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			setFeatureMode(d, "skill", "template");
			run(["deploy", "--offline", "--no-gitignore"], d);

			const target = join(d, ".mycode/skills/hello-skill/SKILL.md");
			const content = readFileSync(target, "utf8");
			expect(content).toContain("name: hello-skill");
			expect(content).toContain("Hello Skill");
			expect(content).toContain("## Instructions");
		} finally {
			cleanup(d);
		}
	});

	// template mode with var injection substitutes {{ var.* }} in skill content
	test("template mode injects vars into skill content", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			setFeatureMode(d, "skill", "template");

			// Write a skill with a var placeholder
			const skillDir = join(d, `${rootDir(d)}/skills/var-skill`);
			mkdirSync(skillDir, { recursive: true });
			writeFileSync(
				join(skillDir, "SKILL.md"),
				`---
name: var-skill
description: Var injection test
---

Hello {{ var.agent_name }}`,
			);

			run(["deploy", "--offline", "--no-gitignore"], d);
			const target = join(d, ".mycode/skills/var-skill/SKILL.md");
			expectRegularFile(target);
			const content = readFileSync(target, "utf8");
			expect(content).toContain("Hello Mycode");
			expect(content).not.toContain("{{ var.agent_name }}");
		} finally {
			cleanup(d);
		}
	});
});

// ── skills extra files symlinked ──────────────────────────────────────────────

test.describe("symlink-mode: skills extra files", () => {
	// extra files in skill directory are symlinked alongside SKILL.md
	test("extra files are symlinked in link mode", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);

			// Add extra files to the scaffolded hello-skill
			const skillDir = join(d, `${rootDir(d)}/skills/hello-skill`);
			writeFileSync(join(skillDir, "helper.py"), "print('hi')");
			mkdirSync(join(skillDir, "data"), { recursive: true });
			writeFileSync(join(skillDir, "data", "config.json"), "{}");

			run(["deploy", "--offline", "--no-gitignore"], d);

			// SKILL.md should be a symlink (link mode)
			expectSymlink(join(d, ".mycode/skills/hello-skill/SKILL.md"));
			// Extra files should also be symlinks
			expectSymlink(join(d, ".mycode/skills/hello-skill/helper.py"));
			expectSymlink(join(d, ".mycode/skills/hello-skill/data/config.json"));

			// Extra file content should resolve to source
			expect(
				readFileSync(join(d, ".mycode/skills/hello-skill/helper.py"), "utf8"),
			).toBe("print('hi')");
		} finally {
			cleanup(d);
		}
	});

	// extra files are symlinked even in template mode (only SKILL.md follows mode)
	test("extra files are symlinked in template mode too", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			setFeatureMode(d, "skill", "template");

			const skillDir = join(d, `${rootDir(d)}/skills/hello-skill`);
			writeFileSync(join(skillDir, "helper.py"), "print('hi')");

			run(["deploy", "--offline", "--no-gitignore"], d);

			// SKILL.md should be a regular file (template mode)
			expectRegularFile(join(d, ".mycode/skills/hello-skill/SKILL.md"));
			// Extra file should still be a symlink
			expectSymlink(join(d, ".mycode/skills/hello-skill/helper.py"));
		} finally {
			cleanup(d);
		}
	});

	// skill with only SKILL.md creates no extra symlinks
	test("skill with only SKILL.md creates no extra symlinks", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--no-gitignore"], d);

			// Only SKILL.md should exist in the target directory
			const targetDir = join(d, ".mycode/skills/hello-skill");
			expect(existsSync(join(targetDir, "SKILL.md"))).toBe(true);
			// No other files should exist in the target directory
			const { readdirSync } = await import("node:fs");
			const entries = readdirSync(targetDir);
			expect(entries).toEqual(["SKILL.md"]);
		} finally {
			cleanup(d);
		}
	});

	// deeply nested extra files are symlinked with mirrored structure
	test("deeply nested extra files are symlinked", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);

			const skillDir = join(d, `${rootDir(d)}/skills/hello-skill`);
			mkdirSync(join(skillDir, "a", "b"), { recursive: true });
			writeFileSync(join(skillDir, "a", "b", "deep.txt"), "deep");

			run(["deploy", "--offline", "--no-gitignore"], d);

			expectSymlink(join(d, ".mycode/skills/hello-skill/a/b/deep.txt"));
			expect(
				readFileSync(
					join(d, ".mycode/skills/hello-skill/a/b/deep.txt"),
					"utf8",
				),
			).toBe("deep");
		} finally {
			cleanup(d);
		}
	});
});

// ── dedup works with symlink deploy ──────────────────────────────────────────

test.describe("symlink-mode: dedup", () => {
	// two providers targeting the same skill path: only one symlink is created
	test("dedup creates only one symlink for same target", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);

			// Add a second provider targeting the same skill path
			const localConfig = join(d, rootDir(d), "local.config.toml");
			writeFileSync(
				localConfig,
				readFileSync(localConfig, "utf8") +
					`\n[providers.duplicate.skills]
target = "{{ dir.workspace }}/.mycode/skills/{{ skill.name }}/SKILL.md"
`,
			);

			const { exitCode } = run(["deploy", "--offline", "--no-gitignore"], d);
			expect(exitCode).toBe(0);

			// The symlink should exist (one of the two providers created it)
			const target = join(d, ".mycode/skills/hello-skill/SKILL.md");
			expectSymlink(target);
		} finally {
			cleanup(d);
		}
	});
});

// ── --dry-run reports symlink operations ─────────────────────────────────────

test.describe("symlink-mode: --dry-run", () => {
	// dry-run shows [@] for symlink operations without creating them
	test("dry-run shows [@] for symlinks without creating them", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { stdout, exitCode } = run(["deploy", "--dry-run", "--offline"], d);
			expect(exitCode).toBe(0);
			expect(stdout).toContain("[@]");
			expect(stdout).toContain("symlink");

			// No actual symlinks should exist
			const target = join(d, ".mycode/skills/hello-skill/SKILL.md");
			expect(existsSync(target)).toBe(false);
		} finally {
			cleanup(d);
		}
	});

	// dry-run shows symlink path for skills
	test("dry-run shows skill symlink path", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { stdout, exitCode } = run(["deploy", "--dry-run", "--offline"], d);
			expect(exitCode).toBe(0);
			expect(stdout).toContain(".mycode/skills/hello-skill/SKILL.md");
		} finally {
			cleanup(d);
		}
	});
});

// ── .gitignore fence includes symlinked paths ────────────────────────────────

test.describe("symlink-mode: .gitignore fence", () => {
	// --gitignore flag includes symlinked skill paths in the fence
	test("gitignore fence includes symlinked skill paths", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			run(["deploy", "--offline", "--gitignore"], d);

			const gitignore = readFileSync(join(d, ".gitignore"), "utf8");
			expect(gitignore).toContain(".mycode/");
		} finally {
			cleanup(d);
		}
	});

	// symlinked paths appear in gitignore even without template-rendered files
	test("gitignore fence works when only symlinks are deployed", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			// Remove all features except skill to isolate symlink-only deploy
			const configPath = join(d, rootDir(d), "config.toml");
			writeFileSync(
				configPath,
				readFileSync(configPath, "utf8").replace(
					/features = \[[\s\S]*?\]/,
					'features = [\n    "skill",\n]',
				),
			);

			run(["deploy", "--offline", "--gitignore"], d);

			const gitignore = readFileSync(join(d, ".gitignore"), "utf8");
			expect(gitignore).toContain(".mycode/");
		} finally {
			cleanup(d);
		}
	});
});
