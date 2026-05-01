## REMOVED Requirements

### Requirement: Add command creates a new command source file
**Reason**: `dotagents add command` is removed as part of the CLI restructuring. The top-level `add` subcommand is deleted. Command creation moves to `dotagents commands new`.
**Migration**: Replace `dotagents add command <name> [flags]` with `dotagents commands new <name> [flags]`. All flags (`-d`, `-c`, `-t`, `-f`, `--deploy`) are preserved on the new subcommand.

### Requirement: Add skill creates a new skill source directory and file
**Reason**: `dotagents add skill` is removed as part of the CLI restructuring. The top-level `add` subcommand is deleted. Local skill creation moves to `dotagents skills new`.
**Migration**: Replace `dotagents add skill <name> [flags]` with `dotagents skills new <name> [flags]`. All flags (`-d`, `-l`, `--compatibility`, `-f`, `--deploy`) are preserved on the new subcommand.

### Requirement: Add supports --deploy flag and TTY deploy confirm
**Reason**: Removed with the top-level `add` command. Deploy support is preserved on `commands new` and `skills new`.
**Migration**: No action needed; `--deploy` and TTY deploy confirm behaviour are unchanged on the replacement subcommands.

### Requirement: Starter body templates are fixed and name-interpolated
**Reason**: Behaviour is unchanged; this requirement moves to the `commands-subcommand` and `skills-subcommand-extended` specs.
**Migration**: No action needed.
