# Provider Display in Init and Config

## Purpose

Specifies the display format for provider entries in the multiselect prompts used by the `init` wizard and `config --edit` command.

## Requirements

### Requirement: Provider selection in init and config shows Name [slug] format
The `prompt_targets()` function used by `init` wizard and `config --edit` SHALL display each provider in the multiselect as `Provider Name [provider-slug]` (e.g., "Claude Code [claude]"). When a provider has no name in the registry, the label SHALL fall back to the bare slug.

#### Scenario: Provider with name shows enriched label
- **WHEN** the provider registry contains an entry with slug "claude" and name "Claude Code"
- **THEN** the multiselect item label is "Claude Code [claude]"

#### Scenario: Provider without name shows slug only
- **WHEN** the provider registry contains an entry with slug "roo" and no name
- **THEN** the multiselect item label is "roo"

#### Scenario: config --edit shows same format as init
- **WHEN** user runs `dotagents config global --edit` and the target selection prompt is shown
- **THEN** the provider multiselect uses the same `Name [slug]` format as the init wizard
