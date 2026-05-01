## ADDED Requirements

### Requirement: skills new, rm, and ls are peers of skills add
The `skills` subcommand group SHALL expose four sub-actions: `add` (existing, registry install), `new` (local scaffold), `rm` (local delete), and `ls` (local listing). The existing `skills add` sub-action SHALL remain unchanged.

#### Scenario: skills add is unaffected by the restructuring
- **WHEN** user runs `dotagents skills add vercel-labs/agent-skills`
- **THEN** the skill is installed from the registry exactly as before this change

#### Scenario: skills subcommand group help lists all four sub-actions
- **WHEN** user runs `dotagents skills --help`
- **THEN** the help output lists `add`, `new`, `rm`, and `ls` as available sub-actions
