import { existsSync, unlinkSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "@microsoft/tui-test";
import {
	cleanup,
	initWithLocalProvider,
	makeTmpDir,
	makeTwoDirs,
	run,
	shellProgram,
} from "./helpers.js";

// ── CLI flows (non-interactive) ──────────────────────────────────────────────

test.describe("config CLI – no workspace", () => {
	// config outside workspace exits with error referencing init
	test("config without workspace exits non-zero with helpful message", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode, stderr } = run(["config"], d);
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain("dotagents init");
		} finally {
			cleanup(d);
		}
	});

	// config app --edit outside workspace also fails (edit validation fires first)
	test("config app --edit outside workspace fails with cannot edit app message", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode, stderr } = run(["config", "--edit"], d);
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain("cannot be edited directly");
		} finally {
			cleanup(d);
		}
	});
});

test.describe("config CLI – app target", () => {
	// config shows merged app config in a workspace
	test("config in workspace exits 0 and shows features", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode, stdout } = run(["config"], d);
			expect(exitCode).toBe(0);
			// Should mention features
			expect(stdout).toMatch(/features/i);
			// Should mention providers or targets
			expect(stdout).toMatch(/mycode/i);
			expect(stdout).toMatch(/providers/i);
		} finally {
			cleanup(d);
		}
	});

	// config app (explicit) works the same
	test("config app works the same as bare config", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode } = run(["config", "app"], d);
			expect(exitCode).toBe(0);
		} finally {
			cleanup(d);
		}
	});

	// config --json outputs valid JSON with features and providers
	test("config --json outputs valid JSON", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode, stdout } = run(["config", "--json"], d);
			expect(exitCode).toBe(0);
			const parsed = JSON.parse(stdout);
			expect(parsed).toHaveProperty("features");
			expect(parsed).toHaveProperty("targets");
			expect(parsed).toHaveProperty("providers");
		} finally {
			cleanup(d);
		}
	});
});

test.describe("config CLI – global target", () => {
	// config global shows global config
	test("config global exits 0 with config content", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode, stdout } = run(["config", "global"], d);
			expect(exitCode).toBe(0);
			expect(stdout).toMatch(/schema/i);
		} finally {
			cleanup(d);
		}
	});

	// config global --json outputs valid JSON
	test("config global --json outputs valid JSON", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode, stdout } = run(["config", "global", "--json"], d);
			expect(exitCode).toBe(0);
			const parsed = JSON.parse(stdout);
			// GlobalConfig has features, targets, etc.
			expect(parsed).toHaveProperty("features");
		} finally {
			cleanup(d);
		}
	});
});

test.describe("config CLI – local target", () => {
	// config local may or may not exist; depends on init template
	test("config local exists or shows no-local message", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode } = run(["config", "local"], d);
			expect(exitCode).toBe(0);
		} finally {
			cleanup(d);
		}
	});

	// config local --json either outputs JSON or empty object {}
	test("config local --json outputs JSON", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode, stdout } = run(["config", "local", "--json"], d);
			expect(exitCode).toBe(0);
			const parsed = JSON.parse(stdout);
			// advanced template writes features to local config
			expect(parsed).toHaveProperty("features");
		} finally {
			cleanup(d);
		}
	});
});

test.describe("config CLI – --edit validation", () => {
	// config app --edit exits with error about derived config
	test("config app --edit errors with cannot edit app message", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode, stderr } = run(["config", "app", "--edit"], d);
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain("cannot be edited directly");
		} finally {
			cleanup(d);
		}
	});

	// config --edit (defaults to app) errors the same way
	test("config --edit errors (defaults to app)", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode, stderr } = run(["config", "--edit"], d);
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain("cannot be edited directly");
		} finally {
			cleanup(d);
		}
	});

	// config global --edit in non-TTY errors with requires terminal
	test("config global --edit in non-TTY errors with terminal required", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode, stderr } = run(["config", "global", "--edit"], d);
			expect(exitCode).not.toBe(0);
			expect(stderr).toContain("requires a terminal");
		} finally {
			cleanup(d);
		}
	});

	// config --json --edit are mutually exclusive — Clap parse error
	test("config --json --edit exits with conflict error", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			const { exitCode, stderr, stdout } = run(
				["config", "--json", "--edit"],
				d,
			);
			expect(exitCode).toBe(2);
			expect(stderr).toMatch(/cannot be used with/i);
			expect(stdout).toBe("");
		} finally {
			cleanup(d);
		}
	});
});

test.describe("config CLI – empty config", () => {
	// config in an empty-features workspace shows none-configured
	test("config with no features shows none configured", async () => {
		const d = makeTmpDir();
		try {
			run(["init", "--template", "starter"], d);
			const { exitCode, stdout } = run(["config"], d);
			expect(exitCode).toBe(0);
			expect(stdout).toMatch(/none configured/i);
		} finally {
			cleanup(d);
		}
	});
});

// ── Config missing-file handling ──────────────────────────────────────────────

test.describe("config CLI – missing files", () => {
	// TC-CFG-08: missing local.config.toml — text mode shows message, exits 0
	test("config local with missing local.config.toml shows message, exits 0", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			unlinkSync(join(d, ".dotagents/local.config.toml"));
			const { exitCode, stdout } = run(["config", "local", "--ci"], d);
			expect(exitCode).toBe(0);
			expect(stdout).toContain("No local config found");
		} finally {
			cleanup(d);
		}
	});

	// TC-CFG-08: missing local.config.toml — JSON mode returns {}
	test("config local --json with missing local.config.toml returns {}", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			unlinkSync(join(d, ".dotagents/local.config.toml"));
			const { exitCode, stdout } = run(["config", "local", "--json"], d);
			expect(exitCode).toBe(0);
			expect(stdout.trim()).toBe("{}");
		} finally {
			cleanup(d);
		}
	});

	// TC-CFG-09: missing config.toml — text mode exits 1 with "not found"
	test("config global with missing config.toml exits 1 with not found", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			unlinkSync(join(d, ".dotagents/config.toml"));
			const { exitCode, stderr } = run(["config", "global", "--ci"], d);
			expect(exitCode).toBe(1);
			expect(stderr).toContain("not found");
		} finally {
			cleanup(d);
		}
	});

	// TC-CFG-09: missing config.toml — JSON mode also exits 1 with "not found"
	test("config global --json with missing config.toml exits 1 with not found", async () => {
		const d = makeTmpDir();
		try {
			initWithLocalProvider(d);
			unlinkSync(join(d, ".dotagents/config.toml"));
			const { exitCode, stderr } = run(["config", "global", "--json"], d);
			expect(exitCode).toBe(1);
			expect(stderr).toContain("not found");
		} finally {
			cleanup(d);
		}
	});
});

// ── config --cwd ──────────────────────────────────────────────────────────────

test.describe("config --cwd", () => {
	// config --cwd reads config from target workspace
	test("config --cwd reads config from target workspace", async () => {
		const { cwd, workspace } = makeTwoDirs();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				workspace,
			);
			const { exitCode, stdout } = run(["config", "--cwd", workspace], cwd);
			expect(exitCode).toBe(0);
			expect(stdout).toMatch(/features/i);
		} finally {
			cleanup(cwd);
			cleanup(workspace);
		}
	});

	// config --json --cwd outputs JSON for target workspace
	test("config --json --cwd outputs JSON for target workspace", async () => {
		const { cwd, workspace } = makeTwoDirs();
		try {
			run(
				[
					"init",
					"--template",
					"starter",
					"--features",
					"command,instruction,mcp,skill",
				],
				workspace,
			);
			const { exitCode, stdout } = run(
				["config", "--json", "--cwd", workspace],
				cwd,
			);
			expect(exitCode).toBe(0);
			const parsed = JSON.parse(stdout);
			expect(parsed).toHaveProperty("features");
		} finally {
			cleanup(cwd);
			cleanup(workspace);
		}
	});

	// config --cwd without .dotagents/ exits non-zero
	test("config --cwd nonexistent exits non-zero", async () => {
		const d = makeTmpDir();
		try {
			const { exitCode, stderr } = run(
				["config", "--cwd", "/tmp/not-a-dotagents-workspace"],
				d,
			);
			expect(exitCode).not.toBe(0);
			const ok = stderr.includes(".dotagents") || stderr.includes("Failed");
			expect(ok).toBe(true);
		} finally {
			cleanup(d);
		}
	});
});

// ── Config --edit TUI ──────────────────────────────────────────────────────────

// T-CG01: global --edit — select features then provider multiselect (registry-backed)
test.describe("config TUI – T-CG01 global --edit", () => {
	const d = makeTmpDir();
	initWithLocalProvider(d);
	test.use({ program: shellProgram(d, ["config", "global", "--edit"]) });

	test("selects features and completes edit flow", async ({ terminal }) => {
		try {
			await expect(terminal.getByText("Select active features")).toBeVisible();
			// All features are pre-selected from the existing config; confirm as-is.
			terminal.keyPress("Enter");
			// Provider selection is a registry-backed multiselect; skip with Enter.
			await expect(
				terminal.getByText("Which providers would you like to target?"),
			).toBeVisible();
			terminal.keyPress("Enter");
			await expect(terminal.getByText("Done.")).toBeVisible();
		} finally {
			cleanup(d);
		}
	});
});

// T-CL01: local --edit — select override features then provider multiselect
test.describe("config TUI – T-CL01 local --edit", () => {
	const d = makeTmpDir();
	initWithLocalProvider(d);
	test.use({ program: shellProgram(d, ["config", "local", "--edit"]) });

	test("selects a feature and completes edit flow", async ({ terminal }) => {
		try {
			await expect(
				terminal.getByText("Select override features"),
			).toBeVisible();
			// Select first item then confirm.
			terminal.keyPress("Space");
			terminal.keyPress("Enter");
			// Provider selection is a registry-backed multiselect; skip with Enter.
			await expect(
				terminal.getByText("Which providers would you like to target?"),
			).toBeVisible();
			terminal.keyPress("Enter");
			await expect(terminal.getByText("Done.")).toBeVisible();
			// local config should still exist
			expect(existsSync(join(d, ".dotagents/local.config.toml"))).toBe(true);
		} finally {
			cleanup(d);
		}
	});
});

// ── Config TUI display (non-edit, read-only) ───────────────────────────────────

// T-CA01: bare config (app) in TTY shows Active Features and Providers selects
test.describe("config TUI – T-CA01 app display", () => {
	const d = makeTmpDir();
	initWithLocalProvider(d);
	test.use({ program: shellProgram(d, ["config"]) });

	test("shows Active Features then Providers selects", async ({ terminal }) => {
		try {
			await expect(terminal.getByText("Effective Configuration")).toBeVisible();
			await expect(terminal.getByText("Active Features")).toBeVisible();
			terminal.keyPress("Enter");
			await expect(terminal.getByText("Targets")).toBeVisible();
			terminal.keyPress("Enter");
			await expect(terminal.getByText("Providers")).toBeVisible();
			terminal.keyPress("Enter");
			await expect(terminal.getByText("Variables")).toBeVisible();
			terminal.keyPress("Enter");
			await expect(terminal.getByText("Done.")).toBeVisible();
		} finally {
			cleanup(d);
		}
	});
});

// T-CG02: config global in TTY shows Features select
test.describe("config TUI – T-CG02 global display", () => {
	const d = makeTmpDir();
	initWithLocalProvider(d);
	test.use({ program: shellProgram(d, ["config", "global"]) });

	test("shows Features then Targets selects and completes", async ({
		terminal,
	}) => {
		try {
			await expect(terminal.getByText("Global Configuration")).toBeVisible();
			await expect(terminal.getByText("Features")).toBeVisible();
			terminal.keyPress("Enter");
			await expect(terminal.getByText("Targets")).toBeVisible();
			terminal.keyPress("Enter");
			await expect(terminal.getByText("Variables")).toBeVisible();
			terminal.keyPress("Enter");
			await expect(terminal.getByText("Done.")).toBeVisible();
		} finally {
			cleanup(d);
		}
	});
});

// T-CL02: config local in TTY shows Override Features and Override Providers selects
test.describe("config TUI – T-CL02 local display", () => {
	const d = makeTmpDir();
	initWithLocalProvider(d);
	test.use({ program: shellProgram(d, ["config", "local"]) });

	test("shows Override Features then Override Providers selects", async ({
		terminal,
	}) => {
		try {
			await expect(terminal.getByText("Local Configuration")).toBeVisible();
			await expect(terminal.getByText("Override Features")).toBeVisible();
			terminal.keyPress("Enter");
			await expect(terminal.getByText("Override Targets")).toBeVisible();
			terminal.keyPress("Enter");
			await expect(terminal.getByText("Override Providers")).toBeVisible();
			terminal.keyPress("Enter");
			await expect(terminal.getByText("Override Variables")).toBeVisible();
			terminal.keyPress("Enter");
			await expect(terminal.getByText("Done.")).toBeVisible();
		} finally {
			cleanup(d);
		}
	});
});
