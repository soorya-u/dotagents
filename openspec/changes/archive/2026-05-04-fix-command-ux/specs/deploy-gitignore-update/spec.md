## MODIFIED Requirements

### Requirement: Update workspace root .gitignore with fenced section
When the gitignore update step runs, it SHALL write workspace-relative target paths into a dotagents-managed fenced section in the workspace root `.gitignore`. Lines outside the fenced section SHALL NOT be modified. The fenced section SHALL use `#region dotagents` as the opening marker and `#endregion dotagents` as the closing marker.

#### Scenario: .gitignore does not exist — create it
- **WHEN** no `.gitignore` exists at the workspace root
- **THEN** a new `.gitignore` is created containing only the dotagents fenced section with the collected paths, opened with `#region dotagents` and closed with `#endregion dotagents`

#### Scenario: .gitignore exists without fenced section — append section
- **WHEN** `.gitignore` exists with user content but no dotagents fence
- **THEN** the fenced section is appended at the end with `#region dotagents` / `#endregion dotagents` markers; existing content is preserved verbatim

#### Scenario: .gitignore exists with fenced section — add new paths only
- **WHEN** the `#region dotagents` / `#endregion dotagents` section already contains some paths and deploy wrote additional new paths
- **THEN** only the new paths are appended inside the fence; existing entries and user content outside the fence are unchanged

#### Scenario: All paths already present — no write
- **WHEN** every collected target path is already listed inside the `#region dotagents` section
- **THEN** `.gitignore` is not modified

#### Scenario: User content outside fence is preserved
- **WHEN** `.gitignore` contains user entries before and after the dotagents fenced section
- **THEN** after update, those entries remain exactly as they were
