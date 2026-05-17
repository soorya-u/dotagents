## ADDED Requirements

### Requirement: E2e test for undeploy when deployed file is missing on disk
Verify that undeploy handles manually deleted files gracefully.

#### Scenario: Missing deployed file handled gracefully (TC-UNDEPLOY-14)
- **WHEN** a deployed file is manually deleted from disk before running `undeploy --force --no-gitignore`
- **THEN** exit code is 0, remaining deployed files are deleted normally, cache is cleared, no crash or panic

### Requirement: E2e test for old-style gitignore fence cleanup
Verify that undeploy removes legacy `# BEGIN dotagents managed` / `# END dotagents managed` markers. Requires implementation change to recognize old-style markers in the gitignore removal logic.

#### Scenario: Old-style fence markers removed on undeploy (TC-UNDEPLOY-12)
- **WHEN** `.gitignore` contains `# BEGIN dotagents managed` / `# END dotagents managed` markers (legacy format) and `undeploy --force` is run
- **THEN** exit code is 0, the old-style markers and their contents are removed from `.gitignore`, deployed files are deleted, and any user content outside the markers is preserved
