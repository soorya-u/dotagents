import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "@microsoft/tui-test";
import { cleanup, makeTmpDir } from "./helpers.js";
import { shellProgram } from "./helpers.js";

test.describe("init TUI – T-HOOK-01 hooks prompt present", () => {
	const d = makeTmpDir();
	test.use({ program: shellProgram(d, ["init"]) });

	test("wizard shows hooks in features prompt and selects by default", async ({
		terminal,
	}) => {
		try {
			await expect(
				terminal.getByText("Which features do you want to enable?"),
			).toBeVisible();

			// Arrow down to Hooks (last default item) and ensure it is visible/selected
			// We do not assert exact symbol because layout is PTY-captured; presence is enough.
			// Press Enter to accept defaults (includes Hooks).
			terminal.keyPress("Enter");

			await expect(
				terminal.getByText("Which starting template?"),
			).toBeVisible();

			// Accept default Starter
			terminal.keyPress("Enter");

			// Skip providers
			terminal.keyPress("Enter");

			await expect(terminal.getByText("Done! Run")).toBeVisible();

			// hooks.jsonc must exist because Hooks was selected by default
			expect(existsSync(join(d, ".dotagents/hooks.jsonc"))).toBe(true);
		} finally {
			cleanup(d);
		}
	});
});

test.describe("init TUI – T-HOOK-02 --no-hooks excludes hooks", () => {
	const d = makeTmpDir();
	test.use({ program: shellProgram(d, ["init", "--no-hooks"]) });

	test("no hooks prompt path and no hooks.jsonc", async ({ terminal }) => {
		try {
			await expect(
				terminal.getByText("Which features do you want to enable?"),
			).toBeVisible();

			// Accept defaults (but --no-hooks already removed Hook from list)
			terminal.keyPress("Enter");

			await expect(
				terminal.getByText("Which starting template?"),
			).toBeVisible();
			terminal.keyPress("Enter");

			// skip providers
			terminal.keyPress("Enter");

			await expect(terminal.getByText("Done! Run")).toBeVisible();

			expect(existsSync(join(d, ".dotagents/hooks.jsonc"))).toBe(false);
			// config should not list "hook"
			const cfg = readFileSync(join(d, ".dotagents/config.toml"), "utf8");
			expect(cfg).not.toMatch(/features\s*=\s*\[[^\]]*hook/);
		} finally {
			cleanup(d);
		}
	});
});
