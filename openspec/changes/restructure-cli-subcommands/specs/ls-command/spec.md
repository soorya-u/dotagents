## REMOVED Requirements

### Requirement: List skills and commands from source directory
**Reason**: The top-level `dotagents ls` command is removed. Listing splits into two domain-scoped subcommands: `dotagents commands ls` and `dotagents skills ls`.
**Migration**: Replace `dotagents ls` with `dotagents commands ls && dotagents skills ls`. Replace `dotagents ls --commands` with `dotagents commands ls`. Replace `dotagents ls --skills` with `dotagents skills ls`.

### Requirement: Descriptions truncated to terminal width by default
**Reason**: Removed with the top-level `ls` command. Truncation behaviour is preserved on `commands ls` and `skills ls`.
**Migration**: No action needed; truncation and `--full` behaviour are unchanged on the replacement subcommands.

### Requirement: Verbose flag shows full descriptions
**Reason**: Removed with the top-level `ls` command. Additionally, the implicit tie-in between the global `-v`/`--verbose` flag and full-description display is removed entirely as incorrect behaviour. Full descriptions are now shown only via the explicit `--full` flag on `commands ls` and `skills ls`.
**Migration**: Replace `dotagents ls --verbose` with `dotagents commands ls --full` or `dotagents skills ls --full` as appropriate.

### Requirement: No items found exits cleanly
**Reason**: Removed with the top-level `ls` command. Empty-state handling is preserved on `commands ls` and `skills ls`.
**Migration**: No action needed.

### Requirement: Workspace not found produces actionable error
**Reason**: Removed with the top-level `ls` command. Workspace-not-found error handling is preserved on `commands ls` and `skills ls`.
**Migration**: No action needed.
