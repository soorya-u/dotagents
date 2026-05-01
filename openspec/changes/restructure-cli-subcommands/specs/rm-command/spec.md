## REMOVED Requirements

### Requirement: Remove command deletes a command source file
**Reason**: `dotagents rm command` is removed as part of the CLI restructuring. The top-level `rm` subcommand is deleted. Command deletion moves to `dotagents commands rm`.
**Migration**: Replace `dotagents rm command <name> [flags]` with `dotagents commands rm <name> [flags]`. All flags (`-f`, `--deploy`) are preserved on the new subcommand.

### Requirement: Remove skill deletes the skill directory
**Reason**: `dotagents rm skill` is removed as part of the CLI restructuring. The top-level `rm` subcommand is deleted. Skill directory deletion moves to `dotagents skills rm`.
**Migration**: Replace `dotagents rm skill <name> [flags]` with `dotagents skills rm <name> [flags]`. All flags (`-f`, `--deploy`) are preserved on the new subcommand.

### Requirement: TTY removal prompts for confirmation
**Reason**: Removed with the top-level `rm` command. Confirmation prompt behaviour is preserved on `commands rm` and `skills rm`.
**Migration**: No action needed; TTY confirmation and `--force` behaviour are unchanged on the replacement subcommands.

### Requirement: Remove supports --deploy flag and TTY deploy confirm
**Reason**: Removed with the top-level `rm` command. Deploy support is preserved on `commands rm` and `skills rm`.
**Migration**: No action needed; `--deploy` and TTY deploy confirm behaviour are unchanged on the replacement subcommands.
