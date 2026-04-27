#!/usr/bin/env bash
set -e

changed=$(git diff --name-only HEAD~1 HEAD)

if echo "$changed" | grep -qE "public/v1/templates/|\.github/workflows/generate-registry\.yml|scripts/ci/detect_template_changes\.sh"; then
    echo "changes_detected=true" >> "$GITHUB_OUTPUT"
else
    echo "changes_detected=false" >> "$GITHUB_OUTPUT"
fi
